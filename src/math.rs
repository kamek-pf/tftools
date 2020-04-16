//! Math helpers.
use std::ops::Sub;

/// Feature scaling helper - normalization (also called min-max scaling)
pub fn normalize<T>(value: T, min: T, max: T) -> f64
where
    T: Copy + Sub<Output = T> + Into<f64>,
{
    (value - min).into() / (max - min).into()
}

#[test]
fn test_normalize() {
    assert_eq!(normalize(50, 0, 100), 0.5);
    assert_eq!(normalize(50.0, 0.0, 100.0), 0.5);
    assert_eq!(normalize(10u32, 0u32, 100u32), 0.1);
}
