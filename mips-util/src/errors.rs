//! Common error types for the MIPS toolchain

use thiserror::Error;
use std::fmt;

/// Common result type for MIPS operations
pub type MipsResult<T> = Result<T, MipsError>;

/// Top-level error enum for the MIPS toolchain
#[derive(Debug, Error)]
pub enum MipsError {
    /// ISA-related errors (instruction decoding, encoding, etc.)
    #[error("ISA error: {0}")]
    Isa(#[from] IsaError),

    /// ELF file format errors
    #[error("ELF error: {0}")]
    Elf(#[from] ElfError),

    /// Assembly/parsing errors
    #[error("Assembly error: {0}")]
    Assembly(#[from] AssemblyError),

    /// VM execution errors
    #[error("VM error: {0}")]
    Vm(#[from] VmError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

/// ISA-related errors
#[derive(Debug, Error)]
pub enum IsaError {
    #[error("Invalid instruction: 0x{instruction:08x}")]
    InvalidInstruction { instruction: u32 },

    #[error("Unsupported instruction: {mnemonic}")]
    UnsupportedInstruction { mnemonic: String },

    #[error("Invalid register number: {register} (must be 0-31)")]
    InvalidRegister { register: u8 },

    #[error("Invalid immediate value: {value} (range: {min}..={max})")]
    InvalidImmediate { value: i32, min: i32, max: i32 },

    #[error("Bitfield operation failed: {message}")]
    BitfieldError { message: String },

    #[error("Opcode lookup failed for: {opcode:02x}")]
    OpcodeNotFound { opcode: u32 },
}

/// ELF file format errors
#[derive(Debug, Error)]
pub enum ElfError {
    #[error("Invalid ELF magic number")]
    InvalidMagic,

    #[error("Unsupported ELF class: {class}")]
    UnsupportedClass { class: u8 },

    #[error("Unsupported machine type: {machine}")]
    UnsupportedMachine { machine: u16 },

    #[error("Section not found: {name}")]
    SectionNotFound { name: String },

    #[error("Symbol not found: {name}")]
    SymbolNotFound { name: String },

    #[error("Invalid section header: {reason}")]
    InvalidSectionHeader { reason: String },

    #[error("Invalid symbol table entry: {reason}")]
    InvalidSymbol { reason: String },

    #[error("Relocation error: {message}")]
    RelocationError { message: String },
}

/// Assembly and parsing errors
#[derive(Debug, Error)]
pub enum AssemblyError {
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError { line: usize, column: usize, message: String },

    #[error("Lexer error at position {position}: {message}")]
    LexerError { position: usize, message: String },

    #[error("Undefined label: {label}")]
    UndefinedLabel { label: String },

    #[error("Duplicate label: {label}")]
    DuplicateLabel { label: String },

    #[error("Invalid directive: {directive}")]
    InvalidDirective { directive: String },

    #[error("Invalid operand: {operand} for instruction {instruction}")]
    InvalidOperand { operand: String, instruction: String },

    #[error("Address out of range: 0x{address:08x}")]
    AddressOutOfRange { address: u32 },

    #[error("Symbol table error: {message}")]
    SymbolTableError { message: String },
}

/// VM execution errors
#[derive(Debug, Error)]
pub enum VmError {
    #[error("Memory access violation at address 0x{address:08x}")]
    MemoryViolation { address: u32 },

    #[error("Illegal instruction at PC 0x{pc:08x}: 0x{instruction:08x}")]
    IllegalInstruction { pc: u32, instruction: u32 },

    #[error("System call error: syscall {number} with error: {message}")]
    SystemCallError { number: u32, message: String },

    #[error("Stack overflow at address 0x{address:08x}")]
    StackOverflow { address: u32 },

    #[error("Stack underflow at address 0x{address:08x}")]
    StackUnderflow { address: u32 },

    #[error("Division by zero at PC 0x{pc:08x}")]
    DivisionByZero { pc: u32 },

    #[error("Arithmetic overflow at PC 0x{pc:08x}")]
    ArithmeticOverflow { pc: u32 },

    #[error("Unaligned memory access: address 0x{address:08x}, alignment {alignment}")]
    UnalignedAccess { address: u32, alignment: u8 },

    #[error("Breakpoint hit at PC 0x{pc:08x}")]
    BreakpointHit { pc: u32 },

    #[error("Execution halted: {reason}")]
    ExecutionHalted { reason: String },
}

/// DWARF debug information errors
#[derive(Debug, Error)]
pub enum DwarfError {
    #[error("DWARF parsing error: {message}")]
    ParseError { message: String },

    #[error("Unsupported DWARF version: {version}")]
    UnsupportedVersion { version: u16 },

    #[error("Invalid DIE at offset 0x{offset:x}")]
    InvalidDie { offset: u64 },

    #[error("Line number program error: {message}")]
    LineNumberError { message: String },

    #[error("Abbreviation table error: {message}")]
    AbbreviationError { message: String },
}

/// Linking errors
#[derive(Debug, Error)]
pub enum LinkError {
    #[error("Undefined symbol: {symbol}")]
    UndefinedSymbol { symbol: String },

    #[error("Multiple definition of symbol: {symbol}")]
    MultipleDefinition { symbol: String },

    #[error("Relocation overflow: {relocation_type} at address 0x{address:08x}")]
    RelocationOverflow { relocation_type: String, address: u32 },

    #[error("Unsupported relocation type: {relocation_type}")]
    UnsupportedRelocation { relocation_type: u32 },

    #[error("Section alignment error: section {section} requires {required} byte alignment")]
    AlignmentError { section: String, required: u32 },
}

// Convenience constructors for common error patterns
impl MipsError {
    pub fn invalid_instruction(instruction: u32) -> Self {
        MipsError::Isa(IsaError::InvalidInstruction { instruction })
    }

    pub fn invalid_register(register: u8) -> Self {
        MipsError::Isa(IsaError::InvalidRegister { register })
    }

    pub fn memory_violation(address: u32) -> Self {
        MipsError::Vm(VmError::MemoryViolation { address })
    }

    pub fn syntax_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        MipsError::Assembly(AssemblyError::SyntaxError {
            line,
            column,
            message: message.into(),
        })
    }

    pub fn undefined_label(label: impl Into<String>) -> Self {
        MipsError::Assembly(AssemblyError::UndefinedLabel {
            label: label.into(),
        })
    }
}

// Helper trait for converting strings to errors
pub trait IntoMipsError<T> {
    fn into_mips_error(self) -> MipsResult<T>;
}

impl<T> IntoMipsError<T> for Result<T, String> {
    fn into_mips_error(self) -> MipsResult<T> {
        self.map_err(|s| MipsError::Generic(s))
    }
}

// Context extension for better error messages
pub trait MipsErrorContext<T> {
    fn with_context(self, context: impl Into<String>) -> MipsResult<T>;
}

impl<T, E> MipsErrorContext<T> for Result<T, E>
where
    E: Into<MipsError>,
{
    fn with_context(self, context: impl Into<String>) -> MipsResult<T> {
        self.map_err(|e| {
            let base_error = e.into();
            MipsError::Generic(format!("{}: {}", context.into(), base_error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = MipsError::invalid_instruction(0xdeadbeef);
        assert!(error.to_string().contains("Invalid instruction: 0xdeadbeef"));
    }

    #[test]
    fn test_error_chain() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let mips_error = MipsError::from(io_error);
        assert!(mips_error.to_string().contains("I/O error"));
    }

    #[test]
    fn test_context() {
        let result: Result<(), String> = Err("something went wrong".to_string());
        let mips_result = result.into_mips_error().with_context("During parsing");
        
        match mips_result {
            Err(MipsError::Generic(msg)) => {
                assert!(msg.contains("During parsing"));
                assert!(msg.contains("something went wrong"));
            }
            _ => panic!("Expected Generic error"),
        }
    }
}
