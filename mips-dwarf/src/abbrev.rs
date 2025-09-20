//! DWARF abbreviation table generation

/// Abbreviation table builder
pub struct AbbrevTable {
    entries: Vec<AbbrevEntry>,
}

impl AbbrevTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_abbrev(&mut self, code: u32, tag: u32, has_children: bool) {
        self.entries.push(AbbrevEntry {
            code,
            tag,
            has_children,
            attributes: Vec::new(),
        });
    }

    pub fn generate(&self) -> Vec<u8> {
        // TODO: Generate actual abbreviation table
        Vec::new()
    }
}

impl Default for AbbrevTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AbbrevEntry {
    pub code: u32,
    pub tag: u32,
    pub has_children: bool,
    pub attributes: Vec<(u32, u32)>, // (name, form) pairs
}
