//! MIPS assembler frontend

pub mod assembler;
pub mod output;
pub mod parser;

pub use assembler::*;
pub use output::*;
pub use parser::*;

use mips_elf::ElfType;

/// Assembler configuration
#[derive(Debug, Clone)]
pub struct AssemblerConfig {
    pub output_type: ElfType,
    pub debug_info: bool,
    pub optimize: bool,
}

impl Default for AssemblerConfig {
    fn default() -> Self {
        Self {
            output_type: ElfType::Executable,
            debug_info: false,
            optimize: false,
        }
    }
}
