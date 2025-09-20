//! MIPS opcodes and instruction definitions

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

/// Get opcode from instruction word
pub fn get_opcode(instruction: u32) -> u32 {
    (instruction >> 26) & 0x3F
}

/// Get function code from R-type instruction
pub fn get_function(instruction: u32) -> u32 {
    instruction & 0x3F
}

/// Get rs field from instruction
pub fn get_rs(instruction: u32) -> u32 {
    (instruction >> 21) & 0x1F
}

/// Get rt field from instruction  
pub fn get_rt(instruction: u32) -> u32 {
    (instruction >> 16) & 0x1F
}

/// Get rd field from R-type instruction
pub fn get_rd(instruction: u32) -> u32 {
    (instruction >> 11) & 0x1F
}

/// Get shamt field from R-type instruction
pub fn get_shamt(instruction: u32) -> u32 {
    (instruction >> 6) & 0x1F
}

/// Get immediate field from I-type instruction
pub fn get_immediate(instruction: u32) -> u16 {
    (instruction & 0xFFFF) as u16
}

/// Get address field from J-type instruction
pub fn get_address(instruction: u32) -> u32 {
    instruction & 0x3FFFFFF
}
