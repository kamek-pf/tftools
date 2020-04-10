use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LabelMap {
    map: HashMap<String, i64>,
}

impl LabelMap {
    pub fn set(&mut self, label: String, value: i64) {
        if !self.map.get(&label).is_none() {
            self.map.insert(label, value);
        }
    }

    pub fn get(&self, label: &str) -> Option<i64> {
        self.map.get(label).copied()
    }
}
