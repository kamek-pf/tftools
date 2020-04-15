//! This module contains generated protobuf structures used to work with TensorFlow.
//! For convenience, some From implementations were added to the Feature type.
pub mod example;
pub mod feature;
pub mod string_int_label_map;

use protobuf::RepeatedField;

use feature::{BytesList, Feature, FloatList, Int64List};

impl From<Vec<i64>> for Feature {
    fn from(input: Vec<i64>) -> Feature {
        let mut values = Feature::new();
        let mut list = Int64List::new();

        list.set_value(input);
        values.set_int64_list(list);

        values
    }
}

impl From<i64> for Feature {
    fn from(input: i64) -> Feature {
        Feature::from(vec![input])
    }
}

impl From<Vec<f32>> for Feature {
    fn from(input: Vec<f32>) -> Feature {
        let mut values = Feature::new();
        let mut list = FloatList::new();

        list.set_value(input);
        values.set_float_list(list);

        values
    }
}

impl From<f32> for Feature {
    fn from(input: f32) -> Feature {
        Feature::from(vec![input])
    }
}

impl From<Vec<String>> for Feature {
    fn from(input: Vec<String>) -> Feature {
        let mut values = Feature::new();
        let mut list = BytesList::new();
        let mut repeated = RepeatedField::new();

        input.into_iter().for_each(|string| {
            repeated.push(string.into_bytes());
        });

        list.set_value(repeated);
        values.set_bytes_list(list);

        values
    }
}

impl From<String> for Feature {
    fn from(input: String) -> Feature {
        Feature::from(vec![input])
    }
}

impl From<Vec<Vec<u8>>> for Feature {
    fn from(input: Vec<Vec<u8>>) -> Feature {
        let mut values = Feature::new();
        let mut list = BytesList::new();
        let mut repeated = RepeatedField::new();

        input.into_iter().for_each(|bytes| {
            repeated.push(bytes);
        });

        list.set_value(repeated);
        values.set_bytes_list(list);

        values
    }
}

impl From<Vec<u8>> for Feature {
    fn from(input: Vec<u8>) -> Feature {
        Feature::from(vec![input])
    }
}
