//! MIPS instruction definitions following the blueprint design

use crate::{InstructionFormat, RegisterId};
use bitvec::prelude::*;

/// Comprehensive MIPS instruction enum supporting MIPS32 Release 2
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    // Arithmetic operations
    Add {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Addi {
        rt: RegisterId,
        rs: RegisterId,
        imm: i16,
    },
    Addiu {
        rt: RegisterId,
        rs: RegisterId,
        imm: i16,
    },
    Addu {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Sub {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Subu {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },

    // Multiplication and Division
    Mult {
        rs: RegisterId,
        rt: RegisterId,
    },
    Multu {
        rs: RegisterId,
        rt: RegisterId,
    },
    Div {
        rs: RegisterId,
        rt: RegisterId,
    },
    Divu {
        rs: RegisterId,
        rt: RegisterId,
    },
    Mfhi {
        rd: RegisterId,
    },
    Mthi {
        rs: RegisterId,
    },
    Mflo {
        rd: RegisterId,
    },
    Mtlo {
        rs: RegisterId,
    },

    // Logical operations
    And {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Andi {
        rt: RegisterId,
        rs: RegisterId,
        imm: u16,
    },
    Or {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Ori {
        rt: RegisterId,
        rs: RegisterId,
        imm: u16,
    },
    Xor {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Xori {
        rt: RegisterId,
        rs: RegisterId,
        imm: u16,
    },
    Nor {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Lui {
        rt: RegisterId,
        imm: u16,
    },

    // Shift operations
    Sll {
        rd: RegisterId,
        rt: RegisterId,
        shamt: u8,
    },
    Srl {
        rd: RegisterId,
        rt: RegisterId,
        shamt: u8,
    },
    Sra {
        rd: RegisterId,
        rt: RegisterId,
        shamt: u8,
    },
    Sllv {
        rd: RegisterId,
        rt: RegisterId,
        rs: RegisterId,
    },
    Srlv {
        rd: RegisterId,
        rt: RegisterId,
        rs: RegisterId,
    },
    Srav {
        rd: RegisterId,
        rt: RegisterId,
        rs: RegisterId,
    },

    // Comparison operations
    Slt {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Sltu {
        rd: RegisterId,
        rs: RegisterId,
        rt: RegisterId,
    },
    Slti {
        rt: RegisterId,
        rs: RegisterId,
        imm: i16,
    },
    Sltiu {
        rt: RegisterId,
        rs: RegisterId,
        imm: i16,
    },

    // Branch operations
    Beq {
        rs: RegisterId,
        rt: RegisterId,
        offset: i16,
    },
    Bne {
        rs: RegisterId,
        rt: RegisterId,
        offset: i16,
    },
    Blez {
        rs: RegisterId,
        offset: i16,
    },
    Bgtz {
        rs: RegisterId,
        offset: i16,
    },
    Bltz {
        rs: RegisterId,
        offset: i16,
    },
    Bgez {
        rs: RegisterId,
        offset: i16,
    },
    Bltzal {
        rs: RegisterId,
        offset: i16,
    },
    Bgezal {
        rs: RegisterId,
        offset: i16,
    },

    // Jump operations
    J {
        target: u32,
    },
    Jal {
        target: u32,
    },
    Jr {
        rs: RegisterId,
    },
    Jalr {
        rd: RegisterId,
        rs: RegisterId,
    },

    // Load operations
    Lb {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Lbu {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Lh {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Lhu {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Lw {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Lwl {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Lwr {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },

    // Store operations
    Sb {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Sh {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Sw {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Swl {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },
    Swr {
        rt: RegisterId,
        offset: i16,
        base: RegisterId,
    },

    // System operations
    Syscall,
    Break {
        code: u32,
    },

    // Coprocessor operations (basic)
    Mfc0 {
        rt: RegisterId,
        rd: RegisterId,
    },
    Mtc0 {
        rt: RegisterId,
        rd: RegisterId,
    },

    // Pseudo instructions
    Li {
        rt: RegisterId,
        imm: i32,
    },
    La {
        rt: RegisterId,
        addr: u32,
    },
    Move {
        rd: RegisterId,
        rs: RegisterId,
    },
    Nop,
}

impl Instr {
    /// Get the mnemonic string for this instruction
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Instr::Add { .. } => "add",
            Instr::Addi { .. } => "addi",
            Instr::Addiu { .. } => "addiu",
            Instr::Addu { .. } => "addu",
            Instr::Sub { .. } => "sub",
            Instr::Subu { .. } => "subu",
            Instr::Mult { .. } => "mult",
            Instr::Multu { .. } => "multu",
            Instr::Div { .. } => "div",
            Instr::Divu { .. } => "divu",
            Instr::Mfhi { .. } => "mfhi",
            Instr::Mthi { .. } => "mthi",
            Instr::Mflo { .. } => "mflo",
            Instr::Mtlo { .. } => "mtlo",
            Instr::And { .. } => "and",
            Instr::Andi { .. } => "andi",
            Instr::Or { .. } => "or",
            Instr::Ori { .. } => "ori",
            Instr::Xor { .. } => "xor",
            Instr::Xori { .. } => "xori",
            Instr::Nor { .. } => "nor",
            Instr::Lui { .. } => "lui",
            Instr::Sll { .. } => "sll",
            Instr::Srl { .. } => "srl",
            Instr::Sra { .. } => "sra",
            Instr::Sllv { .. } => "sllv",
            Instr::Srlv { .. } => "srlv",
            Instr::Srav { .. } => "srav",
            Instr::Slt { .. } => "slt",
            Instr::Sltu { .. } => "sltu",
            Instr::Slti { .. } => "slti",
            Instr::Sltiu { .. } => "sltiu",
            Instr::Beq { .. } => "beq",
            Instr::Bne { .. } => "bne",
            Instr::Blez { .. } => "blez",
            Instr::Bgtz { .. } => "bgtz",
            Instr::Bltz { .. } => "bltz",
            Instr::Bgez { .. } => "bgez",
            Instr::Bltzal { .. } => "bltzal",
            Instr::Bgezal { .. } => "bgezal",
            Instr::J { .. } => "j",
            Instr::Jal { .. } => "jal",
            Instr::Jr { .. } => "jr",
            Instr::Jalr { .. } => "jalr",
            Instr::Lb { .. } => "lb",
            Instr::Lbu { .. } => "lbu",
            Instr::Lh { .. } => "lh",
            Instr::Lhu { .. } => "lhu",
            Instr::Lw { .. } => "lw",
            Instr::Lwl { .. } => "lwl",
            Instr::Lwr { .. } => "lwr",
            Instr::Sb { .. } => "sb",
            Instr::Sh { .. } => "sh",
            Instr::Sw { .. } => "sw",
            Instr::Swl { .. } => "swl",
            Instr::Swr { .. } => "swr",
            Instr::Syscall => "syscall",
            Instr::Break { .. } => "break",
            Instr::Mfc0 { .. } => "mfc0",
            Instr::Mtc0 { .. } => "mtc0",
            Instr::Li { .. } => "li",
            Instr::La { .. } => "la",
            Instr::Move { .. } => "move",
            Instr::Nop => "nop",
        }
    }

    /// Check if this is a pseudo instruction
    pub fn is_pseudo(&self) -> bool {
        matches!(
            self,
            Instr::Li { .. } | Instr::La { .. } | Instr::Move { .. } | Instr::Nop
        )
    }

    /// Get the instruction format
    pub fn format(&self) -> InstructionFormat {
        match self {
            // R-type instructions
            Instr::Add { .. }
            | Instr::Addu { .. }
            | Instr::Sub { .. }
            | Instr::Subu { .. }
            | Instr::And { .. }
            | Instr::Or { .. }
            | Instr::Xor { .. }
            | Instr::Nor { .. }
            | Instr::Slt { .. }
            | Instr::Sltu { .. }
            | Instr::Sll { .. }
            | Instr::Srl { .. }
            | Instr::Sra { .. }
            | Instr::Sllv { .. }
            | Instr::Srlv { .. }
            | Instr::Srav { .. }
            | Instr::Jr { .. }
            | Instr::Jalr { .. }
            | Instr::Mult { .. }
            | Instr::Multu { .. }
            | Instr::Div { .. }
            | Instr::Divu { .. }
            | Instr::Mfhi { .. }
            | Instr::Mthi { .. }
            | Instr::Mflo { .. }
            | Instr::Mtlo { .. }
            | Instr::Syscall
            | Instr::Break { .. } => InstructionFormat::R,

            // J-type instructions
            Instr::J { .. } | Instr::Jal { .. } => InstructionFormat::J,

            // I-type instructions
            Instr::Addi { .. }
            | Instr::Addiu { .. }
            | Instr::Andi { .. }
            | Instr::Ori { .. }
            | Instr::Xori { .. }
            | Instr::Lui { .. }
            | Instr::Slti { .. }
            | Instr::Sltiu { .. }
            | Instr::Beq { .. }
            | Instr::Bne { .. }
            | Instr::Blez { .. }
            | Instr::Bgtz { .. }
            | Instr::Bltz { .. }
            | Instr::Bgez { .. }
            | Instr::Bltzal { .. }
            | Instr::Bgezal { .. }
            | Instr::Lb { .. }
            | Instr::Lbu { .. }
            | Instr::Lh { .. }
            | Instr::Lhu { .. }
            | Instr::Lw { .. }
            | Instr::Lwl { .. }
            | Instr::Lwr { .. }
            | Instr::Sb { .. }
            | Instr::Sh { .. }
            | Instr::Sw { .. }
            | Instr::Swl { .. }
            | Instr::Swr { .. }
            | Instr::Mfc0 { .. }
            | Instr::Mtc0 { .. } => InstructionFormat::I,

            // Pseudo instructions are handled differently
            Instr::Li { .. } | Instr::La { .. } | Instr::Move { .. } | Instr::Nop => {
                InstructionFormat::Pseudo
            }
        }
    }

    /// Encode this instruction to a 32-bit word
    pub fn encode(&self) -> Result<u32, &'static str> {
        match self {
            Instr::Add { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x20),
            Instr::Addi { rt, rs, imm } => encode_i_type(0x08, *rs, *rt, *imm as u16),
            Instr::Addiu { rt, rs, imm } => encode_i_type(0x09, *rs, *rt, *imm as u16),
            Instr::Addu { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x21),
            Instr::Sub { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x22),
            Instr::Subu { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x23),

            Instr::Mult { rs, rt } => encode_r_type(0x00, *rs, *rt, 0, 0, 0x18),
            Instr::Multu { rs, rt } => encode_r_type(0x00, *rs, *rt, 0, 0, 0x19),
            Instr::Div { rs, rt } => encode_r_type(0x00, *rs, *rt, 0, 0, 0x1A),
            Instr::Divu { rs, rt } => encode_r_type(0x00, *rs, *rt, 0, 0, 0x1B),
            Instr::Mfhi { rd } => encode_r_type(0x00, 0, 0, *rd, 0, 0x10),
            Instr::Mthi { rs } => encode_r_type(0x00, *rs, 0, 0, 0, 0x11),
            Instr::Mflo { rd } => encode_r_type(0x00, 0, 0, *rd, 0, 0x12),
            Instr::Mtlo { rs } => encode_r_type(0x00, *rs, 0, 0, 0, 0x13),

            Instr::And { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x24),
            Instr::Andi { rt, rs, imm } => encode_i_type(0x0C, *rs, *rt, *imm),
            Instr::Or { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x25),
            Instr::Ori { rt, rs, imm } => encode_i_type(0x0D, *rs, *rt, *imm),
            Instr::Xor { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x26),
            Instr::Xori { rt, rs, imm } => encode_i_type(0x0E, *rs, *rt, *imm),
            Instr::Nor { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x27),
            Instr::Lui { rt, imm } => encode_i_type(0x0F, 0, *rt, *imm),

            Instr::Sll { rd, rt, shamt } => {
                encode_r_type(0x00, 0, *rt, *rd, *shamt as RegisterId, 0x00)
            }
            Instr::Srl { rd, rt, shamt } => {
                encode_r_type(0x00, 0, *rt, *rd, *shamt as RegisterId, 0x02)
            }
            Instr::Sra { rd, rt, shamt } => {
                encode_r_type(0x00, 0, *rt, *rd, *shamt as RegisterId, 0x03)
            }
            Instr::Sllv { rd, rt, rs } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x04),
            Instr::Srlv { rd, rt, rs } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x06),
            Instr::Srav { rd, rt, rs } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x07),

            Instr::Slt { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x2A),
            Instr::Sltu { rd, rs, rt } => encode_r_type(0x00, *rs, *rt, *rd, 0, 0x2B),
            Instr::Slti { rt, rs, imm } => encode_i_type(0x0A, *rs, *rt, *imm as u16),
            Instr::Sltiu { rt, rs, imm } => encode_i_type(0x0B, *rs, *rt, *imm as u16),

            Instr::Beq { rs, rt, offset } => encode_i_type(0x04, *rs, *rt, *offset as u16),
            Instr::Bne { rs, rt, offset } => encode_i_type(0x05, *rs, *rt, *offset as u16),
            Instr::Blez { rs, offset } => encode_i_type(0x06, *rs, 0, *offset as u16),
            Instr::Bgtz { rs, offset } => encode_i_type(0x07, *rs, 0, *offset as u16),
            Instr::Bltz { rs, offset } => encode_i_type(0x01, *rs, 0x00, *offset as u16),
            Instr::Bgez { rs, offset } => encode_i_type(0x01, *rs, 0x01, *offset as u16),
            Instr::Bltzal { rs, offset } => encode_i_type(0x01, *rs, 0x10, *offset as u16),
            Instr::Bgezal { rs, offset } => encode_i_type(0x01, *rs, 0x11, *offset as u16),

            Instr::J { target } => encode_j_type(0x02, *target),
            Instr::Jal { target } => encode_j_type(0x03, *target),
            Instr::Jr { rs } => encode_r_type(0x00, *rs, 0, 0, 0, 0x08),
            Instr::Jalr { rd, rs } => encode_r_type(0x00, *rs, 0, *rd, 0, 0x09),

            Instr::Lb { rt, offset, base } => encode_i_type(0x20, *base, *rt, *offset as u16),
            Instr::Lbu { rt, offset, base } => encode_i_type(0x24, *base, *rt, *offset as u16),
            Instr::Lh { rt, offset, base } => encode_i_type(0x21, *base, *rt, *offset as u16),
            Instr::Lhu { rt, offset, base } => encode_i_type(0x25, *base, *rt, *offset as u16),
            Instr::Lw { rt, offset, base } => encode_i_type(0x23, *base, *rt, *offset as u16),
            Instr::Lwl { rt, offset, base } => encode_i_type(0x22, *base, *rt, *offset as u16),
            Instr::Lwr { rt, offset, base } => encode_i_type(0x26, *base, *rt, *offset as u16),

            Instr::Sb { rt, offset, base } => encode_i_type(0x28, *base, *rt, *offset as u16),
            Instr::Sh { rt, offset, base } => encode_i_type(0x29, *base, *rt, *offset as u16),
            Instr::Sw { rt, offset, base } => encode_i_type(0x2B, *base, *rt, *offset as u16),
            Instr::Swl { rt, offset, base } => encode_i_type(0x2A, *base, *rt, *offset as u16),
            Instr::Swr { rt, offset, base } => encode_i_type(0x2E, *base, *rt, *offset as u16),

            Instr::Syscall => encode_r_type(0x00, 0, 0, 0, 0, 0x0C),
            Instr::Break { code } => encode_r_type(
                0x00,
                (*code >> 15) & 0x1F,
                (*code >> 10) & 0x1F,
                (*code >> 5) & 0x1F,
                *code & 0x1F,
                0x0D,
            ),

            Instr::Mfc0 { rt, rd } => encode_r_type(0x10, 0x00, *rt, *rd, 0, 0x00),
            Instr::Mtc0 { rt, rd } => encode_r_type(0x10, 0x04, *rt, *rd, 0, 0x00),

            // Pseudo instructions cannot be directly encoded
            Instr::Li { .. } | Instr::La { .. } | Instr::Move { .. } | Instr::Nop => {
                Err("Pseudo instruction cannot be directly encoded")
            }
        }
    }

    /// Decode a 32-bit instruction word
    pub fn decode(word: u32) -> Result<Self, &'static str> {
        let opcode = extract_bits(word, 26, 6);

        match opcode {
            0x00 => decode_r_type(word),
            0x01 => decode_regimm(word),
            0x02 => Ok(Instr::J {
                target: extract_bits(word, 0, 26),
            }),
            0x03 => Ok(Instr::Jal {
                target: extract_bits(word, 0, 26),
            }),
            0x04 => decode_beq(word),
            0x05 => decode_bne(word),
            0x06 => decode_blez(word),
            0x07 => decode_bgtz(word),
            0x08 => decode_addi(word),
            0x09 => decode_addiu(word),
            0x0A => decode_slti(word),
            0x0B => decode_sltiu(word),
            0x0C => decode_andi(word),
            0x0D => decode_ori(word),
            0x0E => decode_xori(word),
            0x0F => decode_lui(word),
            0x10 => decode_cop0(word),
            0x20 => decode_lb(word),
            0x21 => decode_lh(word),
            0x22 => decode_lwl(word),
            0x23 => decode_lw(word),
            0x24 => decode_lbu(word),
            0x25 => decode_lhu(word),
            0x26 => decode_lwr(word),
            0x28 => decode_sb(word),
            0x29 => decode_sh(word),
            0x2A => decode_swl(word),
            0x2B => decode_sw(word),
            0x2E => decode_swr(word),
            _ => Err("Unknown instruction"),
        }
    }
}

/// Extract bits from a 32-bit word (bit 0 is LSB)
fn extract_bits(word: u32, start: usize, len: usize) -> u32 {
    let bits = word.view_bits::<bitvec::order::Lsb0>();
    bits[start..start + len].load_le::<u32>()
}

/// Encode R-type instruction
fn encode_r_type(
    opcode: u32,
    rs: RegisterId,
    rt: RegisterId,
    rd: RegisterId,
    shamt: RegisterId,
    function: u32,
) -> Result<u32, &'static str> {
    if opcode > 0x3F || rs > 31 || rt > 31 || rd > 31 || shamt > 31 || function > 0x3F {
        return Err("Field value out of range");
    }

    Ok((opcode << 26)
        | ((rs as u32) << 21)
        | ((rt as u32) << 16)
        | ((rd as u32) << 11)
        | ((shamt as u32) << 6)
        | function)
}

/// Encode I-type instruction
fn encode_i_type(
    opcode: u32,
    rs: RegisterId,
    rt: RegisterId,
    immediate: u16,
) -> Result<u32, &'static str> {
    if opcode > 0x3F || rs > 31 || rt > 31 {
        return Err("Field value out of range");
    }

    Ok((opcode << 26) | ((rs as u32) << 21) | ((rt as u32) << 16) | (immediate as u32))
}

/// Encode J-type instruction  
fn encode_j_type(opcode: u32, target: u32) -> Result<u32, &'static str> {
    if opcode > 0x3F || target > 0x3FFFFFF {
        return Err("Field value out of range");
    }

    Ok((opcode << 26) | target)
}

// Decode helper functions
fn decode_r_type(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let rd = extract_bits(word, 11, 5);
    let shamt = extract_bits(word, 6, 5);
    let function = extract_bits(word, 0, 6);

    match function {
        0x00 => Ok(Instr::Sll {
            rd,
            rt,
            shamt: shamt as u8,
        }),
        0x02 => Ok(Instr::Srl {
            rd,
            rt,
            shamt: shamt as u8,
        }),
        0x03 => Ok(Instr::Sra {
            rd,
            rt,
            shamt: shamt as u8,
        }),
        0x04 => Ok(Instr::Sllv { rd, rt, rs }),
        0x06 => Ok(Instr::Srlv { rd, rt, rs }),
        0x07 => Ok(Instr::Srav { rd, rt, rs }),
        0x08 => Ok(Instr::Jr { rs }),
        0x09 => Ok(Instr::Jalr { rd, rs }),
        0x0C => Ok(Instr::Syscall),
        0x0D => Ok(Instr::Break {
            code: (rs << 15) | (rt << 10) | (rd << 5) | shamt,
        }),
        0x10 => Ok(Instr::Mfhi { rd }),
        0x11 => Ok(Instr::Mthi { rs }),
        0x12 => Ok(Instr::Mflo { rd }),
        0x13 => Ok(Instr::Mtlo { rs }),
        0x18 => Ok(Instr::Mult { rs, rt }),
        0x19 => Ok(Instr::Multu { rs, rt }),
        0x1A => Ok(Instr::Div { rs, rt }),
        0x1B => Ok(Instr::Divu { rs, rt }),
        0x20 => Ok(Instr::Add { rd, rs, rt }),
        0x21 => Ok(Instr::Addu { rd, rs, rt }),
        0x22 => Ok(Instr::Sub { rd, rs, rt }),
        0x23 => Ok(Instr::Subu { rd, rs, rt }),
        0x24 => Ok(Instr::And { rd, rs, rt }),
        0x25 => Ok(Instr::Or { rd, rs, rt }),
        0x26 => Ok(Instr::Xor { rd, rs, rt }),
        0x27 => Ok(Instr::Nor { rd, rs, rt }),
        0x2A => Ok(Instr::Slt { rd, rs, rt }),
        0x2B => Ok(Instr::Sltu { rd, rs, rt }),
        _ => Err("Unknown R-type function"),
    }
}

fn decode_regimm(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;

    match rt {
        0x00 => Ok(Instr::Bltz { rs, offset }),
        0x01 => Ok(Instr::Bgez { rs, offset }),
        0x10 => Ok(Instr::Bltzal { rs, offset }),
        0x11 => Ok(Instr::Bgezal { rs, offset }),
        _ => Err("Unknown REGIMM instruction"),
    }
}

fn decode_beq(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Beq { rs, rt, offset })
}

fn decode_bne(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Bne { rs, rt, offset })
}

fn decode_blez(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Blez { rs, offset })
}

fn decode_bgtz(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Bgtz { rs, offset })
}

fn decode_addi(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Addi { rt, rs, imm })
}

fn decode_addiu(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Addiu { rt, rs, imm })
}

fn decode_slti(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Slti { rt, rs, imm })
}

fn decode_sltiu(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Sltiu { rt, rs, imm })
}

fn decode_andi(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as u16;
    Ok(Instr::Andi { rt, rs, imm })
}

fn decode_ori(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as u16;
    Ok(Instr::Ori { rt, rs, imm })
}

fn decode_xori(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as u16;
    Ok(Instr::Xori { rt, rs, imm })
}

fn decode_lui(word: u32) -> Result<Instr, &'static str> {
    let rt = extract_bits(word, 16, 5);
    let imm = extract_bits(word, 0, 16) as u16;
    Ok(Instr::Lui { rt, imm })
}

fn decode_cop0(word: u32) -> Result<Instr, &'static str> {
    let rs = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let rd = extract_bits(word, 11, 5);

    match rs {
        0x00 => Ok(Instr::Mfc0 { rt, rd }),
        0x04 => Ok(Instr::Mtc0 { rt, rd }),
        _ => Err("Unknown COP0 instruction"),
    }
}

fn decode_lb(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Lb { rt, offset, base })
}

fn decode_lh(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Lh { rt, offset, base })
}

fn decode_lwl(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Lwl { rt, offset, base })
}

fn decode_lw(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Lw { rt, offset, base })
}

fn decode_lbu(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Lbu { rt, offset, base })
}

fn decode_lhu(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Lhu { rt, offset, base })
}

fn decode_lwr(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Lwr { rt, offset, base })
}

fn decode_sb(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Sb { rt, offset, base })
}

fn decode_sh(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Sh { rt, offset, base })
}

fn decode_swl(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Swl { rt, offset, base })
}

fn decode_sw(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Sw { rt, offset, base })
}

fn decode_swr(word: u32) -> Result<Instr, &'static str> {
    let base = extract_bits(word, 21, 5);
    let rt = extract_bits(word, 16, 5);
    let offset = extract_bits(word, 0, 16) as i16;
    Ok(Instr::Swr { rt, offset, base })
}
