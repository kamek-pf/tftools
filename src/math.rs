//! Math helpers.
use std::ops::Sub;

use crc::crc32;

/// Feature scaling helper - normalization (also called min-max scaling)
pub fn normalize<T>(value: T, min: T, max: T) -> f64
where
    T: Copy + Sub<Output = T> + Into<f64>,
{
    (value - min).into() / (max - min).into()
}

/// Given some data and a ratio, for instance the bytes of an image and a ratio of 20%,
/// this function computes whether or not or not the data should be retained.
/// You can use it as a predicate to split a dataset between a training set and a testing set.
/// This method is deteministic, applying it to a collection should result in datasets matching the
/// specified ratio closely, but not perfectly.
///
/// ## Example
/// ```rust
/// let input = "anything that can be converted to a slice of bytes";
/// let ratio = 20; // For 20%
/// assert!(retain(input, ratio));
/// ```
pub fn retain<T>(input: T, ratio: u8) -> bool
where
    T: AsRef<[u8]>,
{
    let ratio = if ratio > 100 {
        100f64
    } else {
        ratio as f64 / 100f64
    };

    // Compute a hash of the input, if the hash is below ratio% of the maximum
    // hash value, it is retained.
    let threshold = (u32::max_value() as f64 * ratio).round() as u32;
    let bytes = input.as_ref();
    let crc = crc32::checksum_ieee(&bytes);

    crc < threshold
}

#[test]
fn test_normalize() {
    assert_eq!(normalize(50, 0, 100), 0.5);
    assert_eq!(normalize(50.0, 0.0, 100.0), 0.5);
    assert_eq!(normalize(10u32, 0u32, 100u32), 0.1);
}

#[test]
fn test_split() {
    let input = vec![
        vec![1],
        vec![2],
        vec![3],
        vec![4],
        vec![5],
        vec![6],
        vec![7],
        vec![8],
        vec![9],
        vec![10],
    ];

    let retained = input
        .clone()
        .iter()
        .filter(|element| retain(element, 50))
        .count();
    assert_eq!(retained, 5);

    let retained = input
        .clone()
        .iter()
        .filter(|element| retain(element, 20))
        .count();

    // It SHOULD retain 2 elements.
    // The CRC method provides a deterministic way to split the data
    // and the ratio should be close enough to what we ask, but it's not perfect
    assert_eq!(retained, 1);
}

#[test]
fn test_split_dataset() {
    let input = vec![
        include_bytes!("../dataset/1.jpg").to_vec(),
        include_bytes!("../dataset/2.jpg").to_vec(),
        include_bytes!("../dataset/3.jpg").to_vec(),
        include_bytes!("../dataset/4.jpg").to_vec(),
        include_bytes!("../dataset/5.jpg").to_vec(),
        include_bytes!("../dataset/6.jpg").to_vec(),
    ];

    // Same comment, 20% is about 1.2 here it appears to be rounded to 2
    let retained = input
        .clone()
        .iter()
        .filter(|element| retain(element, 20))
        .count();
    assert_eq!(retained, 2);

    let input = vec![
        include_str!("../dataset/1.xml"),
        include_str!("../dataset/2.xml"),
        include_str!("../dataset/3.xml"),
        include_str!("../dataset/4.xml"),
        include_str!("../dataset/5.xml"),
        include_str!("../dataset/6.xml"),
    ];

    // ... and here to one
    let retained = input
        .clone()
        .iter()
        .filter(|element| retain(element, 20))
        .count();
    assert_eq!(retained, 1);
}
