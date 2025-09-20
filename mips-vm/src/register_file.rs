//! Improved register file using smallvec for performance

use smallvec::{SmallVec, smallvec};
use std::fmt;
use thiserror::Error;

/// MIPS register file with 32 general-purpose registers
/// Uses SmallVec for stack allocation optimization
pub struct RegisterFile {
    /// General-purpose registers $0-$31
    /// $0 is always zero, others are read-write
    registers: SmallVec<[u32; 32]>,
    /// HI register for multiplication/division results
    hi: u32,
    /// LO register for multiplication/division results  
    lo: u32,
    /// Program counter
    pc: u32,
}

impl RegisterFile {
    /// Create a new register file with all registers initialized to 0
    pub fn new() -> Self {
        let mut registers = smallvec![0u32; 32];
        registers[0] = 0; // $zero is always 0
        
        Self {
            registers,
            hi: 0,
            lo: 0,
            pc: 0,
        }
    }

    /// Get the value of a general-purpose register
    pub fn get_gpr(&self, reg: u8) -> u32 {
        if reg >= 32 {
            0 // Invalid register number returns 0
        } else {
            self.registers[reg as usize]
        }
    }

    /// Set the value of a general-purpose register
    /// Note: Setting $zero has no effect
    pub fn set_gpr(&mut self, reg: u8, value: u32) {
        if reg > 0 && reg < 32 {
            self.registers[reg as usize] = value;
        }
        // $zero ($0) cannot be changed
    }

    /// Get the HI register value
    pub fn get_hi(&self) -> u32 {
        self.hi
    }

    /// Set the HI register value
    pub fn set_hi(&mut self, value: u32) {
        self.hi = value;
    }

    /// Get the LO register value  
    pub fn get_lo(&self) -> u32 {
        self.lo
    }

    /// Set the LO register value
    pub fn set_lo(&mut self, value: u32) {
        self.lo = value;
    }

    /// Get the program counter
    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    /// Set the program counter
    pub fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }

    /// Increment the program counter by 4 (next instruction)
    pub fn advance_pc(&mut self) {
        self.pc = self.pc.wrapping_add(4);
    }

    /// Get register name from number
    pub fn register_name(reg: u8) -> &'static str {
        match reg {
            0 => "$zero", 1 => "$at", 2 => "$v0", 3 => "$v1",
            4 => "$a0", 5 => "$a1", 6 => "$a2", 7 => "$a3",
            8 => "$t0", 9 => "$t1", 10 => "$t2", 11 => "$t3",
            12 => "$t4", 13 => "$t5", 14 => "$t6", 15 => "$t7",
            16 => "$s0", 17 => "$s1", 18 => "$s2", 19 => "$s3",
            20 => "$s4", 21 => "$s5", 22 => "$s6", 23 => "$s7",
            24 => "$t8", 25 => "$t9", 26 => "$k0", 27 => "$k1",
            28 => "$gp", 29 => "$sp", 30 => "$fp", 31 => "$ra",
            _ => "$invalid",
        }
    }

    /// Reset all registers to initial state
    pub fn reset(&mut self) {
        for i in 0..32 {
            self.registers[i] = 0;
        }
        self.hi = 0;
        self.lo = 0;
        self.pc = 0;
    }

    /// Get a slice of all general-purpose registers for debugging
    pub fn gpr_slice(&self) -> &[u32] {
        &self.registers
    }
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "RegisterFile {{")?;
        for (i, &value) in self.registers.iter().enumerate() {
            if value != 0 || i == 0 { // Show non-zero registers and $zero
                writeln!(f, "  {}: 0x{:08x} ({})", Self::register_name(i as u8), value, value)?;
            }
        }
        writeln!(f, "  HI: 0x{:08x} ({})", self.hi, self.hi)?;
        writeln!(f, "  LO: 0x{:08x} ({})", self.lo, self.lo)?;
        writeln!(f, "  PC: 0x{:08x} ({})", self.pc, self.pc)?;
        write!(f, "}}")
    }
}

impl fmt::Display for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Registers:")?;
        for i in 0..32 {
            if i % 4 == 0 {
                write!(f, "  ")?;
            }
            let value = self.registers[i];
            write!(f, "{}: {:08x}", Self::register_name(i as u8), value)?;
            if i % 4 == 3 {
                writeln!(f)?;
            } else {
                write!(f, "  ")?;
            }
        }
        writeln!(f, "  HI: {:08x}  LO: {:08x}  PC: {:08x}", self.hi, self.lo, self.pc)
    }
}

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("Invalid register number: {0} (must be 0-31)")]
    InvalidRegister(u8),
    #[error("Cannot write to $zero register")]
    WriteToZero,
}

/// Instruction buffer for pipelined execution
/// Uses SmallVec to store a small number of instructions efficiently
pub type InstructionBuffer = SmallVec<[u32; 4]>; // Typical pipeline depth

/// Branch target buffer for branch prediction
/// Uses SmallVec for frequently accessed branch targets
pub type BranchTargetBuffer = SmallVec<[(u32, u32); 8]>; // (pc, target) pairs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_file_creation() {
        let rf = RegisterFile::new();
        assert_eq!(rf.get_gpr(0), 0); // $zero
        assert_eq!(rf.get_gpr(1), 0); // $at
        assert_eq!(rf.get_hi(), 0);
        assert_eq!(rf.get_lo(), 0);
        assert_eq!(rf.get_pc(), 0);
    }

    #[test]
    fn test_zero_register_immutable() {
        let mut rf = RegisterFile::new();
        rf.set_gpr(0, 0xdeadbeef);
        assert_eq!(rf.get_gpr(0), 0); // Should still be 0
    }

    #[test]
    fn test_general_registers() {
        let mut rf = RegisterFile::new();
        rf.set_gpr(1, 0x12345678);
        rf.set_gpr(31, 0x87654321);
        assert_eq!(rf.get_gpr(1), 0x12345678);
        assert_eq!(rf.get_gpr(31), 0x87654321);
    }

    #[test]
    fn test_hi_lo_registers() {
        let mut rf = RegisterFile::new();
        rf.set_hi(0xabcdef00);
        rf.set_lo(0x12345678);
        assert_eq!(rf.get_hi(), 0xabcdef00);
        assert_eq!(rf.get_lo(), 0x12345678);
    }

    #[test]
    fn test_program_counter() {
        let mut rf = RegisterFile::new();
        rf.set_pc(0x00400000);
        assert_eq!(rf.get_pc(), 0x00400000);
        rf.advance_pc();
        assert_eq!(rf.get_pc(), 0x00400004);
    }

    #[test]
    fn test_invalid_register() {
        let rf = RegisterFile::new();
        assert_eq!(rf.get_gpr(32), 0); // Invalid register returns 0
        assert_eq!(rf.get_gpr(255), 0);
    }

    #[test]
    fn test_register_names() {
        assert_eq!(RegisterFile::register_name(0), "$zero");
        assert_eq!(RegisterFile::register_name(29), "$sp");
        assert_eq!(RegisterFile::register_name(31), "$ra");
        assert_eq!(RegisterFile::register_name(32), "$invalid");
    }
}
