use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LabelMap {
    index: i64,
    map: HashMap<String, i64>,
}

impl LabelMap {
    pub fn new() -> Self {
        LabelMap {
            index: 1,
            ..Default::default()
        }
    }

    pub fn add(&mut self, label: &str) -> Option<i64> {
        let current = self.index;
        if self.map.get(label).is_none() {
            self.map.insert(label.to_owned(), current);
            self.index += 1;
            Some(current)
        } else {
            None
        }
    }

    pub fn get(&self, label: &str) -> Option<i64> {
        self.map.get(label).copied()
    }
}
