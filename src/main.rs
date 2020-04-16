mod cli;
mod math;
mod pascal_voc;
mod tensorflow_protos;

use std::convert::TryFrom;
use std::error::Error;

use structopt::StructOpt;

use cli::{Command, PascalVoc};
use pascal_voc::PrepareOpts;

fn main() -> Result<(), Box<dyn Error>> {
    match Command::from_args() {
        // PASCAL-VOC commands
        Command::PascalVoc(pv_cmd) => match pv_cmd {
            // Prepare subcommand
            PascalVoc::Prepare(opts) => {
                let opts = PrepareOpts::try_from(opts)?;
                println!("{:?}", opts);
                Ok(())
            }
        },
    }
}
