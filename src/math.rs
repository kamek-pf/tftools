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

/// Split a collection in two. The ratio determines the amount of data
/// that should be held by the second member of the tuple.
/// The method used splits the data deteministically and the resulting sizes should
/// match the specified ratio closely, but not perfectly.
pub fn split<T>(input: Vec<T>, right_ratio: u8) -> (Vec<T>, Vec<T>)
where
    T: AsRef<[u8]>,
{
    let right_ratio = if right_ratio > 100 {
        100f64
    } else {
        right_ratio as f64 / 100f64
    };

    // Right capacity is right_ratio% of the collection size
    let right_capacity = ((input.len() as f64) * right_ratio).round() as usize;
    let mut right = Vec::with_capacity(right_capacity);

    // Left capacity is the rest of it
    let left_capacity = input.len() - right_capacity;
    let mut left = Vec::with_capacity(left_capacity);

    // Compute a hash of each element, if the hash is below right_ratio% of maximum
    // hash value, it goes in the right collection. Otherwise, it goes to the left.
    let threshold = (u32::max_value() as f64 * right_ratio).round() as u32;
    input.into_iter().for_each(|element| {
        let bytes = element.as_ref();
        let crc = crc32::checksum_ieee(&bytes);

        if crc >= threshold {
            left.push(element)
        } else {
            right.push(element)
        }
    });

    (left, right)
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

    let (left, right) = split(input.clone(), 50);
    assert_eq!(left.len(), 5);
    assert_eq!(right.len(), 5);

    let (left, right) = split(input, 20);
    // Lengths here SHOULD be 8 and 2, respectively
    // The CRC method provides a deterministic way to split the data
    // and the ratio whould be close enough to what we ask, but it's not perfect
    assert_eq!(left.len(), 9);
    assert_eq!(right.len(), 1);
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
    let (left, right) = split(input, 20);
    assert_eq!(right.len(), 2);
    assert_eq!(left.len(), 4);

    let input = vec![
        include_str!("../dataset/1.xml"),
        include_str!("../dataset/2.xml"),
        include_str!("../dataset/3.xml"),
        include_str!("../dataset/4.xml"),
        include_str!("../dataset/5.xml"),
        include_str!("../dataset/6.xml"),
    ];

    // ... and here to one
    let (left, right) = split(input, 20);
    assert_eq!(right.len(), 1);
    assert_eq!(left.len(), 5);
}
