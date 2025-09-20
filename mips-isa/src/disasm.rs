//! MIPS disassembly tables and functions

use crate::Instruction;

/// Disassemble a MIPS instruction to human-readable format
pub fn disassemble(instruction: &Instruction) -> String {
    match instruction.format {
        crate::InstructionFormat::R => disassemble_r_type(instruction),
        crate::InstructionFormat::I => disassemble_i_type(instruction),
        crate::InstructionFormat::J => disassemble_j_type(instruction),
    }
}

fn disassemble_r_type(instruction: &Instruction) -> String {
    // Extract fields from R-type instruction
    let rs = (instruction.raw >> 21) & 0x1F;
    let rt = (instruction.raw >> 16) & 0x1F;
    let rd = (instruction.raw >> 11) & 0x1F;
    let shamt = (instruction.raw >> 6) & 0x1F;
    let funct = instruction.raw & 0x3F;

    format!(
        "r-type rs:{} rt:{} rd:{} shamt:{} funct:{}",
        rs, rt, rd, shamt, funct
    )
}

fn disassemble_i_type(instruction: &Instruction) -> String {
    // Extract fields from I-type instruction
    let rs = (instruction.raw >> 21) & 0x1F;
    let rt = (instruction.raw >> 16) & 0x1F;
    let immediate = instruction.raw & 0xFFFF;

    format!("i-type rs:{} rt:{} imm:{}", rs, rt, immediate)
}

fn disassemble_j_type(instruction: &Instruction) -> String {
    // Extract fields from J-type instruction
    let address = instruction.raw & 0x3FFFFFF;

    format!("j-type addr:{}", address)
}
