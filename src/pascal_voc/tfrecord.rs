use std::ffi::OsStr;
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
            .and_then(|ext| match ext {
                "png" | "PNG" | "jpg" | "jpeg" | "JPEG" => Some(ext),
                _ => None,
            });

        match (ext, fs::read(&example.path)) {
            (Some(ext), Ok(bytes)) => {
                let width = example.size.width;
                let height = example.size.height;

                // Map labels first, keep track of failures and bail
                let (labels_ok, mapped_labels) = map_labels(&example, &self.label_map);
                if labels_ok {
                    self.classes.push(mapped_labels);
                } else {
                    self.ignored.push(example.path.to_string_lossy().into());
                    return;
                }

                // Add metadata and recorder state
                self.height.push(height as i64);
                self.height.push(width as i64);
                self.filename.push(example.filename.into_bytes());
                self.current_size += bytes.len();
                self.encoded_image_data.push(bytes);
                self.image_format.push(ext.as_bytes().to_owned());

                // Add coordinates
                self.xmins.push(
                    example
                        .objects
                        .iter()
                        .map(|o| math::normalize(o.bndbox.xmin, 0, width))
                        .collect(),
                );
                self.xmaxs.push(
                    example
                        .objects
                        .iter()
                        .map(|o| math::normalize(o.bndbox.xmax, 0, width))
                        .collect(),
                );
                self.ymins.push(
                    example
                        .objects
                        .iter()
                        .map(|o| math::normalize(o.bndbox.ymin, 0, height))
                        .collect(),
                );
                self.ymaxs.push(
                    example
                        .objects
                        .iter()
                        .map(|o| math::normalize(o.bndbox.ymax, 0, height))
                        .collect(),
                );

                // Add labels
                self.classes_text.push(
                    example
                        .objects
                        .iter()
                        .map(|o| o.name.clone().into_bytes())
                        .collect(),
                );
            }
            _ => {
                self.ignored.push(example.path.to_string_lossy().into());
            }
        };
    }
}

// Map labels to integers, bool describes whether or not all labels were mapped successfully
fn map_labels(input: &Annotation, map: &LabelMap) -> (bool, Vec<i64>) {
    unimplemented!();
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
