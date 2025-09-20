pub mod memory;
pub mod vm;
pub mod program;
pub mod address;
pub mod register_file;  // New efficient register file
pub mod symbol_table;   // New fast symbol table

// Re-export commonly used types from other crates
pub use mips_isa::{InstructionKind, InstructionFormat, Word, Immediate};
pub use mips_util::{bitfields, endianness, logging, Register};

// Re-export new improved types
pub use register_file::*;
pub use symbol_table::*;

// Legacy modules for compatibility - these will be phased out
pub mod assembler;
pub mod compiler;
pub mod parser;
pub mod registers;
pub mod transpilers;
