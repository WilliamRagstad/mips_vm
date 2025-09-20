//! MIPS Instruction Set Architecture definitions
//!
//! This crate provides opcodes, decoder functions, and disassembly tables
//! for the MIPS architecture.

pub mod decoder;
pub mod disasm;
pub mod instructions;
pub mod opcodes;

pub use decoder::*;
pub use disasm::*;
pub use instructions::*;
pub use opcodes::*;

/// MIPS instruction formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    R, // Register format
    I, // Immediate format
    J, // Jump format
}

/// Represents a decoded MIPS instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: u32,
    pub format: InstructionFormat,
    pub raw: u32,
}

/// Represents a 32-bit word in MIPS
pub type Word = u32;

/// Represents a 16-bit immediate value
pub type Immediate = u16;
