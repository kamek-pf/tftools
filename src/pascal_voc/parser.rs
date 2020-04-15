//! This module implements deserializers for the PASCAL VOC data format
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

use quick_xml::DeError as DeserializeError;
use serde::Deserialize;
use thiserror::Error;

/// A PASCAL-VOC XML annotation, this is the main object type
#[derive(Debug, Deserialize, Clone)]
pub struct Annotation {
    /// Original name of the folder containing the target image.
    /// Might change if files are moved.
    pub folder: String,
    /// Original name the target image. Might change if files are moved.
    pub filename: String,
    /// Original name of the folder containing the target image.
    /// Might change if files are moved.
    pub path: PathBuf,
    /// Generated field. PASCAL-VOC files contain the absolute path to the original file.
    /// That path will be valid on the machine of the person who labeled the original image.
    /// Since this tool assumes the original image and its PASCAL-VOC file are collocated,
    /// we can build the correct path.
    #[serde(skip)]
    pub system_path: PathBuf,
    /// Source database (might be missing/irrelevant).
    pub source: Source,
    /// Dimensions of the image.
    pub size: Size,
    pub segmented: bool,
    /// Objects labled in the image.
    #[serde(rename = "object", default)]
    pub objects: Vec<Object>,
}

impl Annotation {
    /// Deserialize the content of a file into an Annotation
    pub fn from_file(path: &Path) -> Result<Annotation, PascalVocError> {
        let content: String = fs::read_to_string(path)?;
        let mut example: Annotation = quick_xml::de::from_str(&content)?;
        let mut system_path = path.to_owned();
        system_path.set_file_name(&example.filename);
        example.system_path = system_path;

        Ok(example)
    }
}

/// The <source> top level field
#[derive(Debug, Deserialize, Clone)]
pub struct Source {
    pub database: Option<String>,
    pub annotation: Option<String>,
    pub image: Option<String>,
}

/// The <size> top level field
#[derive(Debug, Deserialize, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
    pub depth: u8,
}

/// The <object> top level field
#[derive(Debug, Deserialize, Clone)]
pub struct Object {
    pub name: String,
    pub pose: String,
    pub truncated: bool,
    pub difficult: bool,
    pub bndbox: BndBox,
}

/// Coordinates of the bounding box, under the <object> field
#[derive(Debug, Deserialize, Clone)]
pub struct BndBox {
    pub xmin: u32,
    pub ymin: u32,
    pub xmax: u32,
    pub ymax: u32,
}

/// Error types you might encounter while working with PASCAL VOC files
#[derive(Debug, Error)]
pub enum PascalVocError {
    #[error("Io error while attempting to read the example")]
    Io(#[from] IoError),

    #[error("Failed to deserialize the example")]
    Deserialize(#[from] DeserializeError),
}

#[test]
fn deserialize_pascal_voc() {
    let first_path = PathBuf::from("./dataset/1.xml");
    let first = Annotation::from_file(&first_path).unwrap();
    assert_eq!(first.filename, "1.jpg");
    assert_eq!(first.objects.len(), 2);
    assert_eq!(first.segmented, false);
    assert_eq!(first.size.width, 480);
    assert_eq!(first.size.height, 360);
    assert_eq!(first.size.depth, 3);
    assert_eq!(first.objects[0].name, "dog");
    assert_eq!(first.objects[0].bndbox.xmin, 85);
    assert_eq!(first.objects[0].bndbox.ymin, 1);
    assert_eq!(first.objects[0].bndbox.xmax, 381);
    assert_eq!(first.objects[0].bndbox.ymax, 244);
    assert_eq!(first.objects[0].truncated, true);
    assert_eq!(first.objects[0].difficult, false);

    let fifth_path = PathBuf::from("./dataset/5.xml");
    let fifth = Annotation::from_file(&fifth_path).unwrap();
    assert_eq!(fifth.filename, "5.jpg");
    assert_eq!(fifth.objects.len(), 1);
    assert_eq!(fifth.segmented, false);
    assert_eq!(fifth.size.width, 1000);
    assert_eq!(fifth.size.height, 667);
    assert_eq!(fifth.size.depth, 3);
    assert_eq!(fifth.objects[0].name, "hotdog");
    assert_eq!(fifth.objects[0].truncated, false);
    assert_eq!(fifth.objects[0].difficult, false);
}
