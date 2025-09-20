//! MIPS opcodes and instruction definitions

use bitvec::prelude::*;
use phf::{phf_map, Map};

/// MIPS R-type instruction function codes
pub mod r_type {
    pub const SLL: u32 = 0x00;
    pub const SRL: u32 = 0x02;
    pub const SRA: u32 = 0x03;
    pub const SLLV: u32 = 0x04;
    pub const SRLV: u32 = 0x06;
    pub const SRAV: u32 = 0x07;
    pub const JR: u32 = 0x08;
    pub const JALR: u32 = 0x09;
    pub const SYSCALL: u32 = 0x0C;
    pub const MFHI: u32 = 0x10;
    pub const MTHI: u32 = 0x11;
    pub const MFLO: u32 = 0x12;
    pub const MTLO: u32 = 0x13;
    pub const MULT: u32 = 0x18;
    pub const MULTU: u32 = 0x19;
    pub const DIV: u32 = 0x1A;
    pub const DIVU: u32 = 0x1B;
    pub const ADD: u32 = 0x20;
    pub const ADDU: u32 = 0x21;
    pub const SUB: u32 = 0x22;
    pub const SUBU: u32 = 0x23;
    pub const AND: u32 = 0x24;
    pub const OR: u32 = 0x25;
    pub const XOR: u32 = 0x26;
    pub const NOR: u32 = 0x27;
    pub const SLT: u32 = 0x2A;
    pub const SLTU: u32 = 0x2B;
}

/// MIPS I-type instruction opcodes
pub mod i_type {
    pub const BLTZ: u32 = 0x01; // Also BGEZ
    pub const J: u32 = 0x02;
    pub const JAL: u32 = 0x03;
    pub const BEQ: u32 = 0x04;
    pub const BNE: u32 = 0x05;
    pub const BLEZ: u32 = 0x06;
    pub const BGTZ: u32 = 0x07;
    pub const ADDI: u32 = 0x08;
    pub const ADDIU: u32 = 0x09;
    pub const SLTI: u32 = 0x0A;
    pub const SLTIU: u32 = 0x0B;
    pub const ANDI: u32 = 0x0C;
    pub const ORI: u32 = 0x0D;
    pub const XORI: u32 = 0x0E;
    pub const LUI: u32 = 0x0F;
    pub const LB: u32 = 0x20;
    pub const LH: u32 = 0x21;
    pub const LW: u32 = 0x23;
    pub const LBU: u32 = 0x24;
    pub const LHU: u32 = 0x25;
    pub const SB: u32 = 0x28;
    pub const SH: u32 = 0x29;
    pub const SW: u32 = 0x2B;
}

/// MIPS J-type instruction opcodes
pub mod j_type {
    pub const J: u32 = 0x02;
    pub const JAL: u32 = 0x03;
}

/// Special opcode for R-type instructions
pub const R_TYPE_OPCODE: u32 = 0x00;

/// Perfect hash map for R-type function code to mnemonic mappings
static R_TYPE_MNEMONICS: Map<u32, &'static str> = phf_map! {
    0x00u32 => "sll",
    0x02u32 => "srl",
    0x03u32 => "sra",
    0x04u32 => "sllv",
    0x06u32 => "srlv",
    0x07u32 => "srav",
    0x08u32 => "jr",
    0x09u32 => "jalr",
    0x0Cu32 => "syscall",
    0x10u32 => "mfhi",
    0x11u32 => "mthi",
    0x12u32 => "mflo",
    0x13u32 => "mtlo",
    0x18u32 => "mult",
    0x19u32 => "multu",
    0x1Au32 => "div",
    0x1Bu32 => "divu",
    0x20u32 => "add",
    0x21u32 => "addu",
    0x22u32 => "sub",
    0x23u32 => "subu",
    0x24u32 => "and",
    0x25u32 => "or",
    0x26u32 => "xor",
    0x27u32 => "nor",
    0x2Au32 => "slt",
    0x2Bu32 => "sltu",
};

/// Perfect hash map for I-type opcode to mnemonic mappings  
static I_TYPE_MNEMONICS: Map<u32, &'static str> = phf_map! {
    0x01u32 => "bltz", // Also bgez - would need rt field to disambiguate
    0x02u32 => "j",
    0x03u32 => "jal",
    0x04u32 => "beq",
    0x05u32 => "bne",
    0x06u32 => "blez",
    0x07u32 => "bgtz",
    0x08u32 => "addi",
    0x09u32 => "addiu",
    0x0Au32 => "slti",
    0x0Bu32 => "sltiu",
    0x0Cu32 => "andi",
    0x0Du32 => "ori",
    0x0Eu32 => "xori",
    0x0Fu32 => "lui",
    0x20u32 => "lb",
    0x21u32 => "lh",
    0x23u32 => "lw",
    0x24u32 => "lbu",
    0x25u32 => "lhu",
    0x28u32 => "sb",
    0x29u32 => "sh",
    0x2Bu32 => "sw",
};

/// Get mnemonic for R-type instruction function code
pub fn get_r_type_mnemonic(function: u32) -> Option<&'static str> {
    R_TYPE_MNEMONICS.get(&function).copied()
}

/// Get mnemonic for I-type instruction opcode
pub fn get_i_type_mnemonic(opcode: u32) -> Option<&'static str> {
    I_TYPE_MNEMONICS.get(&opcode).copied()
}

/// Get opcode from instruction word using bitvec
pub fn get_opcode(instruction: u32) -> u32 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[0..6].load_be::<u32>()
}

/// Get function code from R-type instruction using bitvec
pub fn get_function(instruction: u32) -> u32 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[26..32].load_be::<u32>()
}

/// Get rs field from instruction using bitvec
pub fn get_rs(instruction: u32) -> u32 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[6..11].load_be::<u32>()
}

/// Get rt field from instruction using bitvec
pub fn get_rt(instruction: u32) -> u32 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[11..16].load_be::<u32>()
}

/// Get rd field from R-type instruction using bitvec
pub fn get_rd(instruction: u32) -> u32 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[16..21].load_be::<u32>()
}

/// Get shamt field from R-type instruction using bitvec
pub fn get_shamt(instruction: u32) -> u32 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[21..26].load_be::<u32>()
}

/// Get immediate field from I-type instruction using bitvec
pub fn get_immediate(instruction: u32) -> u16 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[16..32].load_be::<u16>()
}

/// Get address field from J-type instruction using bitvec
pub fn get_address(instruction: u32) -> u32 {
    let bits = instruction.view_bits::<bitvec::order::Msb0>();
    bits[6..32].load_be::<u32>()
}
