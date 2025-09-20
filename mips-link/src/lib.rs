//! Static linker for MIPS object files

pub mod linker;
pub mod relocations;
pub mod symbols;

pub use linker::*;
pub use relocations::*;
pub use symbols::*;

/// Linker configuration
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    pub entry_point: Option<String>,
    pub base_address: u32,
    pub output_format: OutputFormat,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            entry_point: Some("_start".to_string()),
            base_address: 0x10000000,
            output_format: OutputFormat::Executable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Executable,
    SharedLibrary,
}
