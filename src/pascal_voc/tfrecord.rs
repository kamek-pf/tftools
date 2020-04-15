use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Error as IoError};
use std::mem;
use std::path::Path;

use protobuf::Message;
use tensorflow::io::RecordWriter;
use thiserror::Error;

use super::label_map::LabelMap;
use super::parser::Annotation;
use crate::math;
use crate::tensorflow_protos::example::Example;
use crate::tensorflow_protos::feature::{Feature, Features};

/// Allows building tfrecord files by adding PASCAL VOC annotated examples
#[derive(Debug, Default)]
pub struct RecordBuilder {
    // Map labels to integers
    label_map: LabelMap,
    // Max sized allowed for each output file
    max_size: usize,
    // Current estimate of the output file size
    // @TODO: currently unused, update when record splitting is implemented
    current_size: usize,
    // Current chunk
    // @TODO: currently unused, update when record splitting is implemented
    current_chunk: u64,
    // Examples that should be part of the output tfrecord file
    examples: Vec<ExampleImage>,
}

// Flat representation of an example
#[derive(Debug, Default)]
struct ExampleImage {
    height: i64,
    width: i64,
    filename: String,
    image_bytes: Vec<u8>,
    image_format: String,
    xmins: Vec<f32>, // List of normalized left x coordinates in bounding box (1 per box)
    xmaxs: Vec<f32>, // List of normalized right x coordinates in bounding box # (1 per box)
    ymins: Vec<f32>, // List of normalized top y coordinates in bounding box (1 per box)
    ymaxs: Vec<f32>, // List of normalized bottom y coordinates in bounding box # (1 per box)
    classes: Vec<i64>, // List of integer class id of bounding box (1 per box)
    classes_text: Vec<String>, // List of string class name of bounding box (1 per box)
}

impl RecordBuilder {
    /// Initialize a new RecordBuilder
    pub fn new(max_size: usize, label_map: LabelMap) -> RecordBuilder {
        RecordBuilder {
            label_map,
            max_size,
            ..Default::default()
        }
    }

    /// Add an example to the to the set
    pub fn add_example(&mut self, example: Annotation) {
        let ext = example
            .path
            .extension()
            .and_then(|s| s.to_str())
            .and_then(|ext| match ext.to_lowercase().as_ref() {
                "png" | "jpg" | "jpeg" => Some(ext),
                _ => None,
            });

        if let (Some(ext), Ok(bytes)) = (ext, fs::read(&example.system_path)) {
            // First, map labels to their id and bail on error
            let classes = if let Some(classes) = map_labels(&example, &self.label_map) {
                classes
            } else {
                return;
            };

            self.current_size += bytes.len();
            let (xmins, xmaxs, ymins, ymaxs) = get_normalized_coordinates(&example);

            let input = ExampleImage {
                height: example.size.height as i64,
                width: example.size.width as i64,
                filename: example.filename.clone(),
                image_bytes: bytes,
                image_format: ext.to_owned(),
                xmins,
                xmaxs,
                ymins,
                ymaxs,
                classes,
                classes_text: example.objects.iter().map(|o| o.name.clone()).collect(),
            };

            self.examples.push(input);
        }
    }

    /// Write examples added to the builder to a tfrecord file
    pub fn write_tfrecord(&mut self, path: &Path) -> Result<(), TfRecordError> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let buffered_writer = BufWriter::new(file);
        let mut record_writer = RecordWriter::new(buffered_writer);

        mem::take(&mut self.examples)
            .into_iter()
            .for_each(|example| {
                let protobuf = Example::from(example);

                protobuf
                    .write_to_bytes()
                    .ok()
                    .and_then(|bytes| record_writer.write_record(&bytes).ok());
            });

        Ok(())
    }
}

// Map labels to integers. Option is Some if all operations succeed
fn map_labels(input: &Annotation, label_map: &LabelMap) -> Option<Vec<i64>> {
    input
        .objects
        .iter()
        .map(|object| label_map.get(&object.name))
        .collect()
}

// Outputs vectors of normalized coordinates, tuple structure is (xmins, xmaxs, ymins, ymaxs)
fn get_normalized_coordinates(input: &Annotation) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let labels_count = input.objects.len();
    let width = input.size.width;
    let height = input.size.height;
    let mut xmins = Vec::with_capacity(labels_count);
    let mut xmaxs = Vec::with_capacity(labels_count);
    let mut ymins = Vec::with_capacity(labels_count);
    let mut ymaxs = Vec::with_capacity(labels_count);

    input.objects.iter().for_each(|object| {
        xmins.push(math::normalize(object.bndbox.xmin, 0, width) as f32);
        xmaxs.push(math::normalize(object.bndbox.xmax, 0, width) as f32);
        ymins.push(math::normalize(object.bndbox.ymin, 0, height) as f32);
        ymaxs.push(math::normalize(object.bndbox.ymax, 0, height) as f32);
    });

    (xmins, xmaxs, ymins, ymaxs)
}

// Map our internal representation of an example to the generic version used by TensorFlow
impl From<ExampleImage> for Example {
    fn from(input: ExampleImage) -> Example {
        let mut output = Example::new();
        let mut features = Features::new();
        let mut features_map = HashMap::new();

        insert_feature(&mut features_map, "image/height", input.height);
        insert_feature(&mut features_map, "image/width", input.width);

        // According to the docs, "image/filename" and "image/source_id"
        // are both based on file name, see python sample code:
        // https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/using_your_own_dataset.md
        let source_id = input.filename.clone();
        insert_feature(&mut features_map, "image/filename", input.filename);
        insert_feature(&mut features_map, "image/source_id", source_id);
        insert_feature(&mut features_map, "image/encoded", input.image_bytes);
        insert_feature(&mut features_map, "image/format", input.image_format);
        insert_feature(&mut features_map, "image/object/bbox/xmin", input.xmins);
        insert_feature(&mut features_map, "image/object/bbox/xmax", input.xmaxs);
        insert_feature(&mut features_map, "image/object/bbox/ymin", input.ymins);
        insert_feature(&mut features_map, "image/object/bbox/ymax", input.ymaxs);
        insert_feature(&mut features_map, "image/object/class/text", input.classes);
        insert_feature(
            &mut features_map,
            "image/object/class/label",
            input.classes_text,
        );

        features.set_feature(features_map);
        output.set_features(features);

        output
    }
}

// Helper function, converts a list of values into a TensorFlow Feature and insert it into a map
fn insert_feature<V: Into<Feature>>(map: &mut HashMap<String, Feature>, attr: &str, values: V) {
    let attr = String::from(attr);
    map.insert(attr, values.into());
}

/// Error types you might encounter while working with tfrecord files
#[derive(Debug, Error)]
pub enum TfRecordError {
    #[error("Io error while attempting to write tfrecord file")]
    Io(#[from] IoError),
}
