use std::fs;

use super::label_map::LabelMap;
use super::parser::Annotation;
use crate::math;

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
    features: BuilderFeatures,
}

#[derive(Debug, Default)]
struct BuilderFeatures {
    height: Vec<i64>,                 // image heights
    width: Vec<i64>,                  // image width
    filename: Vec<Vec<u8>>,           // File names byte strings
    encoded_image_data: Vec<Vec<u8>>, // Image as bytes
    image_format: Vec<Vec<u8>>,       // Image extension as byte strings, 'b'jpg  or 'b'png,

    xmins: Vec<Vec<f64>>, // List of normalized left x coordinates in bounding box (1 per box)
    xmaxs: Vec<Vec<f64>>, // List of normalized right x coordinates in bounding box # (1 per box)
    ymins: Vec<Vec<f64>>, // List of normalized top y coordinates in bounding box (1 per box)
    ymaxs: Vec<Vec<f64>>, // List of normalized bottom y coordinates in bounding box # (1 per box)
    classes: Vec<Vec<i64>>, // List of integer class id of bounding box (1 per box)
    classes_text: Vec<Vec<Vec<u8>>>, // List of string class name of bounding box (1 per box)
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
                features.classes_text.push(
                    example
                        .objects
                        .iter()
                        .map(|o| o.name.clone().into_bytes())
                        .collect(),
                );

                // Add metadata and update recorder state
                self.current_size += bytes.len();
                features.height.push(height as i64);
                features.height.push(width as i64);
                features.filename.push(example.filename.into_bytes());
                features.encoded_image_data.push(bytes);
                features.image_format.push(ext.as_bytes().to_owned());
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
