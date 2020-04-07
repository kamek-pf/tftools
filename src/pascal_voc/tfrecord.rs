pub struct RecordBuilder {
    // Max sized allowed for each output file
    max_size: u64,

    // Current estimate of the output file size
    current_size: u64,

    // Current chunk
    current_chunk: u64,
}

// Structure: example <- features <- feature

// A single feature is made of :

// height   # Image height
// width    # Image width
// filename # Filename of the image. Empty if image is not from file
// encoded_image_data   # Encoded image bytes
// image_format         # b'jpeg' or b'png'

// xmins    # List of normalized left x coordinates in bounding box (1 per box)
// xmaxs    # List of normalized right x coordinates in bounding box # (1 per box)
// ymins    # List of normalized top y coordinates in bounding box (1 per box)
// ymaxs    # List of normalized bottom y coordinates in bounding box # (1 per box)
// classes  # List of integer class id of bounding box (1 per box)
// classes_text  # List of string class name of bounding box (1 per box)

// @TODO:
// - Push VOC into record builder
// - If estimate > max_size:
//      - Build Example from RecordBuilder
//      - Serialize Example as protobuf text
//      - Write to file with extension based on current_chunk
//      - Update state and keep going
// - If finish method is called:
//      - current_chunk > 0 ? write file with extension : write file without extension
