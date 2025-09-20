//! DWARF line number program generation

/// Line number program builder
pub struct LineProgram {
    entries: Vec<LineEntry>,
}

impl LineProgram {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_line(&mut self, address: u32, file: u32, line: u32, column: u32) {
        self.entries.push(LineEntry {
            address,
            file,
            line,
            column,
        });
    }

    pub fn generate(&self) -> Vec<u8> {
        // TODO: Generate actual DWARF line number program
        Vec::new()
    }
}

impl Default for LineProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LineEntry {
    pub address: u32,
    pub file: u32,
    pub line: u32,
    pub column: u32,
}
