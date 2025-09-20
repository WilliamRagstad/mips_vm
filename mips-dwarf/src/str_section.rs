//! DWARF string section generation

use std::collections::HashMap;

/// String section builder
pub struct StringSection {
    strings: HashMap<String, u32>,
    data: Vec<u8>,
}

impl StringSection {
    pub fn new() -> Self {
        let mut section = Self {
            strings: HashMap::new(),
            data: Vec::new(),
        };
        // First byte is always null
        section.data.push(0);
        section
    }

    pub fn add_string(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.strings.get(s) {
            return offset;
        }

        let offset = self.data.len() as u32;
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0); // null terminator
        self.strings.insert(s.to_string(), offset);
        offset
    }

    pub fn generate(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl Default for StringSection {
    fn default() -> Self {
        Self::new()
    }
}
