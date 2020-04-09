use std::collections::HashMap;
use std::fs;

use protobuf::RepeatedField;

use super::label_map::LabelMap;
use super::parser::Annotation;
use crate::math;
use crate::tensorflow_protos::example::Example;
use crate::tensorflow_protos::feature::{BytesList, Feature, Features, FloatList, Int64List};

#[derive(Debug, Default)]
pub struct RecordBuilder {
    // Map labels to integers
    label_map: LabelMap,
    // Max sized allowed for each output file
    max_size: usize,
    // Current estimate of the output file size
    current_size: usize,
    // Current chunk
    current_chunk: u64,
    // Examples that failed to load or are otherwise invalid
    ignored: Vec<String>,
    // Features used by the neural net
    features: Vec<BuilderFeatures>,
}

#[derive(Debug, Default)]
struct BuilderFeatures {
    height: i64,
    width: i64,
    filename: String,
    image_bytes: Vec<u8>,
    image_format: String,

    xmins: Vec<f64>, // List of normalized left x coordinates in bounding box (1 per box)
    xmaxs: Vec<f64>, // List of normalized right x coordinates in bounding box # (1 per box)
    ymins: Vec<f64>, // List of normalized top y coordinates in bounding box (1 per box)
    ymaxs: Vec<f64>, // List of normalized bottom y coordinates in bounding box # (1 per box)
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

    pub fn add_example(&mut self, example: Annotation) {
        let ext = example
            .path
            .extension()
            .and_then(|s| s.to_str())
            .and_then(|ext| match ext.to_lowercase().as_ref() {
                "png" | "jpg" | "jpeg" => Some(ext),
                _ => None,
            });

        match (ext, fs::read(&example.path)) {
            (Some(ext), Ok(bytes)) => {
                let width = example.size.width;
                let height = example.size.height;
                let features = &mut self.features;

                // Map labels first, keep track of failures and bail
                if let Some(mapped_labels) = map_labels(&example, &self.label_map) {
                    features.classes.push(mapped_labels);
                } else {
                    self.ignored.push(example.path.to_string_lossy().into());
                    return;
                }

                // Add coordinates
                let (xmins, xmaxs, ymins, ymaxs) = get_normalized_coordinates(&example);
                features.xmins.push(xmins);
                features.xmaxs.push(xmaxs);
                features.ymins.push(ymins);
                features.ymaxs.push(ymaxs);

                // Add labels
                features
                    .classes_text
                    .push(example.objects.iter().map(|o| o.name.clone()).collect());

                // Add metadata and update recorder state
                self.current_size += bytes.len();
                features.height.push(height as i64);
                features.height.push(width as i64);
                features.filename.push(example.filename.clone());
                features.image_bytes.push(bytes);
                features.image_format.push(ext.to_owned());
            }
            _ => {
                self.ignored.push(example.path.to_string_lossy().into());
            }
        };
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

// Outputs vectors of normalized coordinates, tuple struct is (xmins, xmaxs, ymins, ymaxs)
fn get_normalized_coordinates(input: &Annotation) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let labels_count = input.objects.len();
    let width = input.size.width;
    let height = input.size.height;
    let mut xmins = Vec::with_capacity(labels_count);
    let mut xmaxs = Vec::with_capacity(labels_count);
    let mut ymins = Vec::with_capacity(labels_count);
    let mut ymaxs = Vec::with_capacity(labels_count);

    input.objects.iter().for_each(|object| {
        xmins.push(math::normalize(object.bndbox.xmin, 0, width));
        xmaxs.push(math::normalize(object.bndbox.xmax, 0, width));
        ymins.push(math::normalize(object.bndbox.ymin, 0, height));
        ymaxs.push(math::normalize(object.bndbox.ymax, 0, height));
    });

    (xmins, xmaxs, ymins, ymaxs)
}

// Map our features to TensorFlow's Example, which can be serialized into a tfrecord file
impl From<BuilderFeatures> for Example {
    fn from(builder: BuilderFeatures) -> Example {
        let mut example = Example::new();
        let mut features = Features::new();
        let mut features_map = HashMap::new();

        insert_feature(&mut features_map, "image/height", builder.height);
        insert_feature(&mut features_map, "image/width", builder.width);

        // According to the docs, "image/filename" and "image/source_id"
        // are both based on file name, see python sample code:
        // https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/using_your_own_dataset.md
        let source_id = builder.filename.clone();
        insert_feature(&mut features_map, "image/filename", builder.filename);
        insert_feature(&mut features_map, "image/source_id", source_id);

        insert_feature(&mut features_map, "image/encoded", builder.image_bytes);
        insert_feature(&mut features_map, "image/format", builder.image_format);
        insert_feature(&mut features_map, "image/object/bbox/xmin", builder.xmins);

        features.set_feature(features_map);
        example.set_features(features);

        example
    }
}

// Helper function, converts a list of values into a TensorFlow Feature and insert it into a map
fn insert_feature<V: Into<Feature>>(map: &mut HashMap<String, Feature>, attr: &str, values: V) {
    let attr = String::from(attr);
    map.insert(attr, values.into());
}

impl From<Vec<i64>> for Feature {
    fn from(input: Vec<i64>) -> Feature {
        let mut values = Feature::new();
        let mut list = Int64List::new();

        list.set_value(input);
        values.set_int64_list(list);

        values
    }
}

impl From<Vec<f32>> for Feature {
    fn from(input: Vec<f32>) -> Feature {
        let mut values = Feature::new();
        let mut list = FloatList::new();

        list.set_value(input);
        values.set_float_list(list);

        values
    }
}

impl From<Vec<String>> for Feature {
    fn from(input: Vec<String>) -> Feature {
        let mut values = Feature::new();
        let mut list = BytesList::new();
        let mut repeated = RepeatedField::new();

        input.into_iter().for_each(|string| {
            repeated.push(string.into_bytes());
        });

        list.set_value(repeated);
        values.set_bytes_list(list);

        values
    }
}

impl From<Vec<Vec<u8>>> for Feature {
    fn from(input: Vec<Vec<u8>>) -> Feature {
        let mut values = Feature::new();
        let mut list = BytesList::new();
        let mut repeated = RepeatedField::new();

        input.into_iter().for_each(|bytes| {
            repeated.push(bytes);
        });

        list.set_value(repeated);
        values.set_bytes_list(list);

        values
    }
}

// Structure: example <- features <- feature

// @TODO:
// - Push VOC into record builder
// - If estimate > max_size:
//      - Build Example from RecordBuilder
//      - Serialize Example as protobuf text
//      - Write to file with extension based on current_chunk
//      - Update state and keep going
// - If finish method is called:
//      - current_chunk > 0 ? write file with extension : write file without extension
