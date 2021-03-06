use protoc_rust::{self, Args, Customize};

// When the project is built, this file will generate Rust files from
// TensorFlow's ProtoBuf definitions.
// Generated files are NOT commited, and written to src/tensorflow_protos.

fn main() {
    protoc_rust::run(Args {
        out_dir: "src/tensorflow_protos",
        input: &[
            "models/research/object_detection/protos/string_int_label_map.proto",
            "tensorflow/tensorflow/core/example/example.proto",
            "tensorflow/tensorflow/core/example/feature.proto",
        ],
        includes: &["tensorflow/", "models/"],
        customize: Customize {
            ..Default::default()
        },
    })
    .expect("protoc");
}
