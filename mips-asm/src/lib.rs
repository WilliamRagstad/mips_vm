//! MIPS assembler frontend following blueprint design
//!
//! Implements the lex→parse→expand→symbols→layout→emit pipeline:
//! 1. Lex: Tokenize assembly source using logos
//! 2. Parse: Build AST using nom parser combinators
//! 3. Expand: Process macros and pseudo-instructions
//! 4. Symbols: Resolve labels and build symbol table
//! 5. Layout: Assign addresses to sections and symbols
//! 6. Emit: Generate ELF object file with relocations

pub mod lexer;
pub mod parser;
pub mod expander;
pub mod symbols;
pub mod layout;
pub mod emitter;
pub mod assembler;
pub mod ast;
pub mod errors;

// Legacy modules for compatibility during migration
pub mod output;

pub use assembler::*;
pub use ast::*;
pub use errors::*;
pub use lexer::*;
pub use parser::*;
pub use expander::*;
pub use symbols::*;
pub use layout::*;
pub use emitter::*;
pub use output::*;

use mips_elf::ElfType;

/// Assembler configuration
#[derive(Debug, Clone)]
pub struct AssemblerConfig {
    pub output_type: ElfType,
    pub debug_info: bool,
    pub optimize: bool,
    pub target_endian: mips_elf::writer::Endianness,
}

impl Default for AssemblerConfig {
    fn default() -> Self {
        Self {
            output_type: ElfType::Relocatable,
            debug_info: false,
            optimize: false,
            target_endian: mips_elf::writer::Endianness::Little,
        }
    }
}
