//! DWARF debug info generation

/// Debug info builder
pub struct DebugInfo {
    entries: Vec<u8>,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_compile_unit(&mut self, name: &str, producer: &str) {
        // TODO: Generate compile unit DIE
    }

    pub fn generate(&self) -> Vec<u8> {
        self.entries.clone()
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self::new()
    }
}
