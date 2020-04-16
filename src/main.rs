mod math;
mod pascal_voc;
mod tensorflow_protos;

use std::path::PathBuf;

fn main() {
    let input = PathBuf::from("dataset");
    let output = PathBuf::from("output");
    let opts = pascal_voc::PrepareOpts { input, output };

    pascal_voc::prepare(opts);
}
