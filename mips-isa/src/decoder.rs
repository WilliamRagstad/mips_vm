//! MIPS instruction decoder

use crate::{opcodes, Instruction, InstructionFormat};

/// Decode a 32-bit MIPS instruction
pub fn decode_instruction(raw: u32) -> Result<Instruction, DecodeError> {
    let opcode = opcodes::get_opcode(raw);

    let format = match opcode {
        0x00 => InstructionFormat::R,        // R-type instructions use opcode 0
        0x02 | 0x03 => InstructionFormat::J, // J and JAL
        _ => InstructionFormat::I,           // Most other instructions are I-type
    };

    Ok(Instruction {
        opcode,
        format,
        raw,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Invalid instruction: {0:08x}")]
    InvalidInstruction(u32),
}
