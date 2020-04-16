use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufWriter, Error as IoError, Write};
use std::path::Path;

use thiserror::Error;

use crate::tensorflow_protos::string_int_label_map::{StringIntLabelMap, StringIntLabelMapItem};

#[derive(Debug, Clone, Default)]
pub struct LabelMap {
    index: i64,
    map: HashMap<String, i64>,
}

impl LabelMap {
    /// Create a new label mapper
    pub fn new() -> Self {
        LabelMap {
            index: 1,
            ..Default::default()
        }
    }

    /// Add a label to the collection. It's safe to call this function repeatedly with the same label.
    /// Always returns the correct ID for a given label.
    pub fn add(&mut self, label: &str) -> i64 {
        let current = self.index;
        if self.map.get(label).is_none() {
            self.map.insert(label.to_owned(), current);
            self.index += 1;
        }
        current
    }

    /// Get the ID for a label
    pub fn get(&self, label: &str) -> Option<i64> {
        self.map.get(label).copied()
    }

    /// Write examples added to the builder to a tfrecord file
    pub fn write_to_file(self, path: &Path) -> Result<(), LabelMapError> {
        let protobuf = StringIntLabelMap::from(self);
        let pbtxt = format!("{:#?}", protobuf);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let mut buffered_writer = BufWriter::new(file);
        buffered_writer.write_all(pbtxt.as_bytes())?;

        Ok(())
    }
}

impl From<LabelMap> for StringIntLabelMap {
    fn from(input: LabelMap) -> StringIntLabelMap {
        let content = input
            .map
            .into_iter()
            .map(|(label, id)| {
                let mut item = StringIntLabelMapItem::new();
                item.set_name(label);
                item.set_id(id as i32);
                item
            })
            .collect();

        let mut protobuf = StringIntLabelMap::new();
        protobuf.set_item(content);

        protobuf
    }
}

/// Error types you might encounter while working with label maps
#[derive(Debug, Error)]
pub enum LabelMapError {
    #[error("Io error while attempting to write label map")]
    Io(#[from] IoError),
}
