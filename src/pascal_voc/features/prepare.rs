//! This module implements the logic necessary to prepare a PASCAL-VOC dataset.
//! - Parse PASCAL-VOC files
//! - Generate the label_map.txt file required by TensorFlow
//! - Split the data into a training set and a test set
//! - Generate tfrecord files for each set
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

use log::warn;
use thiserror::Error;
use walkdir::WalkDir;

use crate::math;
use crate::pascal_voc::label_map::{LabelMap, LabelMapError};
use crate::pascal_voc::parser::{Annotation, PascalVocError};
use crate::pascal_voc::tfrecord::{RecordBuilder, TfRecordError};

/// Configuration options for preparing TensorFlow input files
/// from PASCAL-VOC annotated images
#[derive(Debug)]
pub struct PrepareOpts {
    /// Input directory, where your data set is. Will be searched recursively.
    pub input: PathBuf,
    /// Output directory, where the TensorFlow configuration files will be written.
    pub output: PathBuf,
    /// Percentage of data that should be placed in the test set.
    pub test_set_ratio: u8,
}

// Takes a directory as a input, will recursively search for PASCAL-VOC files
// and generate tfrecord files in the output directory
pub fn prepare(opts: PrepareOpts) -> Result<Report, PrepareError> {
    // Make sure output directory exist, create it otherwise
    fs::create_dir_all(&opts.output)?;

    // Report information while processing the dataset
    let mut report = Report::default();

    // Collect all annotations
    let mut input_examples = Vec::new();
    get_xml_paths(&opts.input)
        .iter()
        .for_each(|path| match Annotation::from_file(path) {
            Ok(annotation) => {
                input_examples.push(annotation);
                report.valid_annotations += 1;
            }
            Err(e) => report.invalid_annotations.push((path.to_owned(), e)),
        });

    // Build and write label map
    let label_map = gen_label_map(&opts, &input_examples)?;
    report.labels = true;

    // Split dataset in two and generate tfrecord file for each set
    let (test, train) = split_dataset(input_examples, opts.test_set_ratio);
    if gen_tfrecord(&opts, test, &label_map, "test.tfrecord")?.is_some() {
        report.test = true;
    }

    if gen_tfrecord(&opts, train, &label_map, "train.tfrecord")?.is_some() {
        report.train = true;
    }

    Ok(report)
}

// Split the dataset in two, a training set and a testing set.
// Tuple structure is (test, train)
fn split_dataset(input: Vec<Annotation>, ratio: u8) -> (Vec<Annotation>, Vec<Annotation>) {
    input.into_iter().partition(|annotation| {
        fs::read(&annotation.system_path)
            .map(|bytes| math::retain(bytes, ratio))
            .unwrap_or(false)
    })
}

// Generate the label map and write it to a file
fn gen_label_map(opts: &PrepareOpts, examples: &[Annotation]) -> Result<LabelMap, PrepareError> {
    // Generate label map
    let mut label_map = LabelMap::new();
    examples
        .iter()
        .flat_map(|e| e.objects.iter())
        .for_each(|o| {
            label_map.add(&o.name);
        });

    // Write label map to file
    let mut label_output: PathBuf = opts.output.clone();
    label_output.push("label_map.txt");
    label_map.clone().write_to_file(&label_output)?;

    Ok(label_map)
}

// Generate the label map and write it to a file
fn gen_tfrecord(
    opts: &PrepareOpts,
    examples: Vec<Annotation>,
    label_map: &LabelMap,
    filename: &str,
) -> Result<Option<()>, PrepareError> {
    if examples.is_empty() {
        warn!("{} dataset is empty, tfrecord won't be generated", filename);
        return Ok(None);
    }

    let mut record = RecordBuilder::new(0, label_map.clone());
    examples.into_iter().for_each(|e| record.add_example(e));

    // Write tfrecord
    let mut record_output: PathBuf = opts.output.clone();
    record_output.push(filename);
    record.write_tfrecord(&record_output)?;

    Ok(Some(()))
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
    pub valid_annotations: usize,
    pub invalid_annotations: Vec<(PathBuf, PascalVocError)>,
    pub train: bool,
    pub test: bool,
    pub labels: bool,
}

impl Report {
    pub fn total_examples(&self) -> usize {
        self.valid_annotations + self.invalid_annotations.len()
    }

    pub fn print_report(&self) {
        match (self.test, self.train) {
            (true, true) => println!(
                "Done, 3 files were written, found {} examples.",
                self.total_examples()
            ),
            (_, true) | (true, _) => println!(
                "Done, 2 files were written, one dataset was empty, found {} examples.",
                self.total_examples()
            ),
            _ => (),
        }

        if !self.invalid_annotations.is_empty() {
            println!(
                "{} example(s) could not be processed:",
                self.invalid_annotations.len(),
            );
            self.invalid_annotations.iter().for_each(|e| {
                println!("   - In {:?} - {}", e.0, e.1);
            })
        }
    }
}

#[derive(Debug, Error)]
pub enum PrepareError {
    #[error("An IO error occured while preparing the dataset")]
    IoError(#[from] IoError),

    #[error("Something went wrong while generating label map file")]
    LabelMap(#[from] LabelMapError),

    #[error("Something went wrong while generating tfrecord file")]
    TfRecord(#[from] TfRecordError),
}
