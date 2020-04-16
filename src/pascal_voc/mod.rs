pub mod label_map;
pub mod parser;
pub mod tfrecord;

use std::path::{Path, PathBuf};

use thiserror::Error;
use walkdir::WalkDir;

use label_map::{LabelMap, LabelMapError};
use parser::{Annotation, PascalVocError};
use tfrecord::{RecordBuilder, TfRecordError};

pub struct PrepareOpts {
    pub input: PathBuf,
    pub output: PathBuf,
}

// Takes a directory as a input, will recursively search for PASCAL-VOC files
// and generate tfrecord files in the output directory
pub fn prepare(opts: PrepareOpts) -> Result<Report, PrepareError> {
    // Report information while processing the dataset
    let mut report = Report::default();

    // Collect all annotations
    let mut input_examples = Vec::new();
    get_xml_paths(&opts.input)
        .iter()
        .for_each(|path| match Annotation::from_file(path) {
            Ok(annotation) => input_examples.push(annotation),
            Err(e) => report.invalid_annotations.push((path.to_owned(), e)),
        });

    // Generate label map
    let mut label_map = LabelMap::new();
    input_examples
        .iter()
        .flat_map(|e| e.objects.iter())
        .for_each(|o| {
            label_map.add(&o.name);
        });

    // Write label map to file
    let mut label_output: PathBuf = opts.output.clone().into();
    label_output.push("label_map.txt");
    label_map.clone().write_to_file(&label_output)?;

    // Generate tfrecord
    let mut record = RecordBuilder::new(0, label_map.clone());
    input_examples
        .into_iter()
        .for_each(|e| record.add_example(e));

    // Write tfrecord
    let mut record_output: PathBuf = opts.output.into();
    record_output.push("out.tfrecord");
    record.write_tfrecord(&record_output)?;

    Ok(report)
}

// Recursively walk the specified root directory and return XML paths
fn get_xml_paths(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .and_then(|ext| match ext.to_lowercase().as_ref() {
                    "xml" => Some(e.path().to_owned()),
                    _ => None,
                })
        })
        .collect()
}

#[derive(Debug, Default)]
pub struct Report {
    invalid_annotations: Vec<(PathBuf, PascalVocError)>,
}

#[derive(Debug, Error)]
pub enum PrepareError {
    #[error("Something went wrong while generating label map file")]
    LabelMap(#[from] LabelMapError),

    #[error("Something went wrong while generating tfrecord file")]
    TfRecord(#[from] TfRecordError),
}
