use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LabelMap {
    index: i64,
    map: HashMap<String, i64>,
}

impl LabelMap {
    /// Create a new label mapper
    pub fn new() -> Self {
        LabelMap {
            index: 1,
            ..Default::default()
        }
    }

    /// Add a label to the collection. It's safe to call this function repeatedly with the same label.
    /// Always returns the correct ID for a given label.
    pub fn add(&mut self, label: &str) -> i64 {
        let current = self.index;
        if self.map.get(label).is_none() {
            self.map.insert(label.to_owned(), current);
            self.index += 1;
        }
        current
    }

    /// Get the ID for a label
    pub fn get(&self, label: &str) -> Option<i64> {
        self.map.get(label).copied()
    }
}
