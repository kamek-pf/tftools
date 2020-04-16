//! Implements the CLI interface.
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;
use thiserror::Error;

use crate::pascal_voc::PrepareOpts;

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Use a PASCAL-VOC dataset
    PascalVoc(PascalVoc),
}

#[derive(StructOpt, Debug)]
pub enum PascalVoc {
    /// Prepare a PASCAL-VOC dataset for tensorflow
    /// This operations generates the label map and two tfrecord files: a training set and a test set
    Prepare(PrepareCliOpts),
}

#[derive(StructOpt, Debug)]
pub struct PrepareCliOpts {
    /// Input directory, where your dataset is. Will be searched recursively
    #[structopt(short = "i", long = "input")]
    pub input: PathBuf,
    /// Output directory, where the TensorFlow configuration files will be written
    #[structopt(short = "o", long = "output")]
    pub output: PathBuf,
    /// Percentage of data that should be retained and placed in the test set
    #[structopt(long = "retain", default_value = "20%")]
    pub retain: String,
}

// Convert the CLI structure for the prepare operation into out internal representation
impl TryFrom<PrepareCliOpts> for PrepareOpts {
    type Error = CliError;

    fn try_from(cli: PrepareCliOpts) -> Result<PrepareOpts, CliError> {
        let retain = if cli.retain.contains('/') {
            cli.retain.split('/').next().unwrap_or("").to_owned()
        } else if cli.retain.contains('%') {
            cli.retain.split('%').next().unwrap_or("").to_owned()
        } else {
            cli.retain
        };

        let opts = PrepareOpts {
            input: cli.input,
            output: cli.output,
            test_set_ratio: u8::from_str(&retain)?,
        };

        Ok(opts)
    }
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Could not parse integer value")]
    Integer(#[from] ParseIntError),
}
