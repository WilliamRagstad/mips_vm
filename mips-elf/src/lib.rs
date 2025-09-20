//! Minimal ELF writer and reader for MIPS binaries

pub mod reader;
pub mod types;
pub mod writer;

pub use reader::*;
pub use types::*;
pub use writer::*;

/// ELF file types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfType {
    None = 0,
    Relocatable = 1,  // ET_REL
    Executable = 2,   // ET_EXEC
    SharedObject = 3, // ET_DYN
    Core = 4,         // ET_CORE
}

/// MIPS machine type
pub const EM_MIPS: u16 = 8;
