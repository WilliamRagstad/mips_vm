//! MIPS assembler core - Enhanced with existing mips-vm logic

use crate::{ParsedProgram, ParsedInstruction, Operand, AssemblerConfig};
use mips_isa::{InstructionKind, InstructionFormat, r_type, i_type, j_type, R_TYPE_OPCODE};
use mips_util::bitfields::{insert_bits, extract_bits};
use std::collections::HashMap;

/// MIPS assembler
pub struct Assembler {
    config: AssemblerConfig,
    labels: HashMap<String, u32>,
}

impl Assembler {
    pub fn new(config: AssemblerConfig) -> Self {
        Self { 
            config,
            labels: HashMap::new(),
        }
    }

    /// Assemble a parsed program into machine code
    pub fn assemble(&mut self, program: &ParsedProgram) -> Result<AssembledProgram, AssemblerError> {
        let mut assembled = AssembledProgram::new();
        
        // First pass: collect labels and their addresses
        let mut address = 0u32;
        for label in &program.labels {
            self.labels.insert(label.name.clone(), address);
        }
        
        for instruction in &program.instructions {
            address += 4; // Each instruction is 4 bytes
        }
        
        // Second pass: generate machine code
        address = 0;
        for instruction in &program.instructions {
            let machine_code = self.assemble_instruction(instruction, address)?;
            assembled.code.extend_from_slice(&machine_code.to_le_bytes());
            address += 4;
        }
        
        // Add symbols
        for (name, addr) in &self.labels {
            assembled.symbols.push(Symbol {
                name: name.clone(),
                address: *addr,
                size: 4,
            });
        }
        
        Ok(assembled)
    }

    fn assemble_instruction(&self, instruction: &ParsedInstruction, pc: u32) -> Result<u32, AssemblerError> {
        let kind = InstructionKind::from_mnemonic(&instruction.mnemonic)
            .ok_or_else(|| AssemblerError::UnknownInstruction(instruction.mnemonic.clone()))?;

        match kind {
            // R-type instructions
            InstructionKind::Add => self.encode_r_type(R_TYPE_OPCODE, r_type::ADD, &instruction.operands),
            InstructionKind::Addu => self.encode_r_type(R_TYPE_OPCODE, r_type::ADDU, &instruction.operands),
            InstructionKind::Sub => self.encode_r_type(R_TYPE_OPCODE, r_type::SUB, &instruction.operands),
            InstructionKind::Subu => self.encode_r_type(R_TYPE_OPCODE, r_type::SUBU, &instruction.operands),
            InstructionKind::And => self.encode_r_type(R_TYPE_OPCODE, r_type::AND, &instruction.operands),
            InstructionKind::Or => self.encode_r_type(R_TYPE_OPCODE, r_type::OR, &instruction.operands),
            InstructionKind::Xor => self.encode_r_type(R_TYPE_OPCODE, r_type::XOR, &instruction.operands),
            InstructionKind::Nor => self.encode_r_type(R_TYPE_OPCODE, r_type::NOR, &instruction.operands),
            InstructionKind::Slt => self.encode_r_type(R_TYPE_OPCODE, r_type::SLT, &instruction.operands),
            InstructionKind::Sltu => self.encode_r_type(R_TYPE_OPCODE, r_type::SLTU, &instruction.operands),
            InstructionKind::Sll => self.encode_r_shift(R_TYPE_OPCODE, r_type::SLL, &instruction.operands),
            InstructionKind::Srl => self.encode_r_shift(R_TYPE_OPCODE, r_type::SRL, &instruction.operands),
            InstructionKind::Sra => self.encode_r_shift(R_TYPE_OPCODE, r_type::SRA, &instruction.operands),
            InstructionKind::Jr => self.encode_r_jump(R_TYPE_OPCODE, r_type::JR, &instruction.operands),
            
            // I-type instructions
            InstructionKind::Addi => self.encode_i_type(i_type::ADDI, &instruction.operands),
            InstructionKind::Addiu => self.encode_i_type(i_type::ADDIU, &instruction.operands),
            InstructionKind::Andi => self.encode_i_type(i_type::ANDI, &instruction.operands),
            InstructionKind::Ori => self.encode_i_type(i_type::ORI, &instruction.operands),
            InstructionKind::Xori => self.encode_i_type(i_type::XORI, &instruction.operands),
            InstructionKind::Slti => self.encode_i_type(i_type::SLTI, &instruction.operands),
            InstructionKind::Sltiu => self.encode_i_type(i_type::SLTIU, &instruction.operands),
            InstructionKind::Lui => self.encode_i_lui(&instruction.operands),
            InstructionKind::Lw => self.encode_i_memory(i_type::LW, &instruction.operands),
            InstructionKind::Sw => self.encode_i_memory(i_type::SW, &instruction.operands),
            InstructionKind::Lb => self.encode_i_memory(i_type::LB, &instruction.operands),
            InstructionKind::Sb => self.encode_i_memory(i_type::SB, &instruction.operands),
            InstructionKind::Beq => self.encode_i_branch(i_type::BEQ, &instruction.operands, pc),
            InstructionKind::Bne => self.encode_i_branch(i_type::BNE, &instruction.operands, pc),
            
            // J-type instructions
            InstructionKind::J => self.encode_j_type(j_type::J, &instruction.operands),
            InstructionKind::Jal => self.encode_j_type(j_type::JAL, &instruction.operands),
            
            // Pseudo instructions
            InstructionKind::Li => self.encode_pseudo_li(&instruction.operands),
            InstructionKind::Move => self.encode_pseudo_move(&instruction.operands),
            InstructionKind::Nop => Ok(0x00000000), // NOP is encoded as SLL $0, $0, 0
            
            _ => Err(AssemblerError::UnknownInstruction(instruction.mnemonic.clone())),
        }
    }

    fn encode_r_type(&self, opcode: u32, funct: u32, operands: &[Operand]) -> Result<u32, AssemblerError> {
        if operands.len() != 3 {
            return Err(AssemblerError::InvalidOperand("R-type instruction requires 3 operands".to_string()));
        }

        let rd = self.parse_register(&operands[0])?;
        let rs = self.parse_register(&operands[1])?;
        let rt = self.parse_register(&operands[2])?;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, opcode, 26, 6);
        instruction = insert_bits(instruction, rs, 21, 5);
        instruction = insert_bits(instruction, rt, 16, 5);
        instruction = insert_bits(instruction, rd, 11, 5);
        instruction = insert_bits(instruction, 0, 6, 5); // shamt = 0
        instruction = insert_bits(instruction, funct, 0, 6);

        Ok(instruction)
    }

    fn encode_r_shift(&self, opcode: u32, funct: u32, operands: &[Operand]) -> Result<u32, AssemblerError> {
        if operands.len() != 3 {
            return Err(AssemblerError::InvalidOperand("Shift instruction requires 3 operands".to_string()));
        }

        let rd = self.parse_register(&operands[0])?;
        let rt = self.parse_register(&operands[1])?;
        let shamt = self.parse_immediate(&operands[2])? as u32;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, opcode, 26, 6);
        instruction = insert_bits(instruction, 0, 21, 5); // rs = 0
        instruction = insert_bits(instruction, rt, 16, 5);
        instruction = insert_bits(instruction, rd, 11, 5);
        instruction = insert_bits(instruction, shamt, 6, 5);
        instruction = insert_bits(instruction, funct, 0, 6);

        Ok(instruction)
    }

    fn encode_r_jump(&self, opcode: u32, funct: u32, operands: &[Operand]) -> Result<u32, AssemblerError> {
        if operands.len() != 1 {
            return Err(AssemblerError::InvalidOperand("Jump register instruction requires 1 operand".to_string()));
        }

        let rs = self.parse_register(&operands[0])?;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, opcode, 26, 6);
        instruction = insert_bits(instruction, rs, 21, 5);
        instruction = insert_bits(instruction, 0, 16, 5); // rt = 0
        instruction = insert_bits(instruction, 0, 11, 5); // rd = 0
        instruction = insert_bits(instruction, 0, 6, 5);  // shamt = 0
        instruction = insert_bits(instruction, funct, 0, 6);

        Ok(instruction)
    }

    fn encode_i_type(&self, opcode: u32, operands: &[Operand]) -> Result<u32, AssemblerError> {
        if operands.len() != 3 {
            return Err(AssemblerError::InvalidOperand("I-type instruction requires 3 operands".to_string()));
        }

        let rt = self.parse_register(&operands[0])?;
        let rs = self.parse_register(&operands[1])?;
        let immediate = self.parse_immediate(&operands[2])? as u16;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, opcode, 26, 6);
        instruction = insert_bits(instruction, rs, 21, 5);
        instruction = insert_bits(instruction, rt, 16, 5);
        instruction = insert_bits(instruction, immediate as u32, 0, 16);

        Ok(instruction)
    }

    fn encode_i_lui(&self, operands: &[Operand]) -> Result<u32, AssemblerError> {
        if operands.len() != 2 {
            return Err(AssemblerError::InvalidOperand("LUI instruction requires 2 operands".to_string()));
        }

        let rt = self.parse_register(&operands[0])?;
        let immediate = self.parse_immediate(&operands[1])? as u16;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, i_type::LUI, 26, 6);
        instruction = insert_bits(instruction, 0, 21, 5); // rs = 0
        instruction = insert_bits(instruction, rt, 16, 5);
        instruction = insert_bits(instruction, immediate as u32, 0, 16);

        Ok(instruction)
    }

    fn encode_i_memory(&self, opcode: u32, operands: &[Operand]) -> Result<u32, AssemblerError> {
        if operands.len() != 2 {
            return Err(AssemblerError::InvalidOperand("Memory instruction requires 2 operands".to_string()));
        }

        let rt = self.parse_register(&operands[0])?;
        let (offset, rs) = self.parse_memory_reference(&operands[1])?;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, opcode, 26, 6);
        instruction = insert_bits(instruction, rs, 21, 5);
        instruction = insert_bits(instruction, rt, 16, 5);
        instruction = insert_bits(instruction, offset as u32, 0, 16);

        Ok(instruction)
    }

    fn encode_i_branch(&self, opcode: u32, operands: &[Operand], pc: u32) -> Result<u32, AssemblerError> {
        if operands.len() != 3 {
            return Err(AssemblerError::InvalidOperand("Branch instruction requires 3 operands".to_string()));
        }

        let rs = self.parse_register(&operands[0])?;
        let rt = self.parse_register(&operands[1])?;
        let target = self.parse_label(&operands[2])?;
        
        // Calculate relative offset
        let offset = ((target as i32 - (pc + 4) as i32) / 4) as i16;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, opcode, 26, 6);
        instruction = insert_bits(instruction, rs, 21, 5);
        instruction = insert_bits(instruction, rt, 16, 5);
        instruction = insert_bits(instruction, offset as u32, 0, 16);

        Ok(instruction)
    }

    fn encode_j_type(&self, opcode: u32, operands: &[Operand]) -> Result<u32, AssemblerError> {
        if operands.len() != 1 {
            return Err(AssemblerError::InvalidOperand("Jump instruction requires 1 operand".to_string()));
        }

        let address = self.parse_label(&operands[0])? >> 2; // Address is word-aligned

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, opcode, 26, 6);
        instruction = insert_bits(instruction, address, 0, 26);

        Ok(instruction)
    }

    fn encode_pseudo_li(&self, operands: &[Operand]) -> Result<u32, AssemblerError> {
        // LI is implemented as ORI $rt, $0, immediate for small values
        // or LUI + ORI for larger values
        if operands.len() != 2 {
            return Err(AssemblerError::InvalidOperand("LI instruction requires 2 operands".to_string()));
        }

        let rt = self.parse_register(&operands[0])?;
        let immediate = self.parse_immediate(&operands[1])?;

        if immediate >= 0 && immediate <= 0xFFFF {
            // Use ORI $rt, $0, immediate
            let mut instruction = 0u32;
            instruction = insert_bits(instruction, i_type::ORI, 26, 6);
            instruction = insert_bits(instruction, 0, 21, 5); // rs = $0
            instruction = insert_bits(instruction, rt, 16, 5);
            instruction = insert_bits(instruction, immediate as u32, 0, 16);
            Ok(instruction)
        } else {
            // For now, just use the lower 16 bits - full implementation would need two instructions
            let mut instruction = 0u32;
            instruction = insert_bits(instruction, i_type::ORI, 26, 6);
            instruction = insert_bits(instruction, 0, 21, 5); // rs = $0
            instruction = insert_bits(instruction, rt, 16, 5);
            instruction = insert_bits(instruction, (immediate & 0xFFFF) as u32, 0, 16);
            Ok(instruction)
        }
    }

    fn encode_pseudo_move(&self, operands: &[Operand]) -> Result<u32, AssemblerError> {
        // MOVE is implemented as OR $rd, $rs, $0
        if operands.len() != 2 {
            return Err(AssemblerError::InvalidOperand("MOVE instruction requires 2 operands".to_string()));
        }

        let rd = self.parse_register(&operands[0])?;
        let rs = self.parse_register(&operands[1])?;

        let mut instruction = 0u32;
        instruction = insert_bits(instruction, R_TYPE_OPCODE, 26, 6);
        instruction = insert_bits(instruction, rs, 21, 5);
        instruction = insert_bits(instruction, 0, 16, 5); // rt = $0
        instruction = insert_bits(instruction, rd, 11, 5);
        instruction = insert_bits(instruction, 0, 6, 5);  // shamt = 0
        instruction = insert_bits(instruction, r_type::OR, 0, 6);

        Ok(instruction)
    }

    fn parse_register(&self, operand: &Operand) -> Result<u32, AssemblerError> {
        match operand {
            Operand::Register(reg_str) => {
                let reg_str = reg_str.strip_prefix('$')
                    .ok_or_else(|| AssemblerError::InvalidOperand(format!("Invalid register: {}", reg_str)))?;
                
                match reg_str {
                    "zero" | "0" => Ok(0),
                    "at" | "1" => Ok(1),
                    "v0" | "2" => Ok(2),
                    "v1" | "3" => Ok(3),
                    "a0" | "4" => Ok(4),
                    "a1" | "5" => Ok(5),
                    "a2" | "6" => Ok(6),
                    "a3" | "7" => Ok(7),
                    "t0" | "8" => Ok(8),
                    "t1" | "9" => Ok(9),
                    "t2" | "10" => Ok(10),
                    "t3" | "11" => Ok(11),
                    "t4" | "12" => Ok(12),
                    "t5" | "13" => Ok(13),
                    "t6" | "14" => Ok(14),
                    "t7" | "15" => Ok(15),
                    "s0" | "16" => Ok(16),
                    "s1" | "17" => Ok(17),
                    "s2" | "18" => Ok(18),
                    "s3" | "19" => Ok(19),
                    "s4" | "20" => Ok(20),
                    "s5" | "21" => Ok(21),
                    "s6" | "22" => Ok(22),
                    "s7" | "23" => Ok(23),
                    "t8" | "24" => Ok(24),
                    "t9" | "25" => Ok(25),
                    "k0" | "26" => Ok(26),
                    "k1" | "27" => Ok(27),
                    "gp" | "28" => Ok(28),
                    "sp" | "29" => Ok(29),
                    "fp" | "30" => Ok(30),
                    "ra" | "31" => Ok(31),
                    _ => {
                        if let Ok(num) = reg_str.parse::<u32>() {
                            if num <= 31 {
                                Ok(num)
                            } else {
                                Err(AssemblerError::InvalidOperand(format!("Invalid register number: {}", num)))
                            }
                        } else {
                            Err(AssemblerError::InvalidOperand(format!("Invalid register: {}", reg_str)))
                        }
                    }
                }
            }
            _ => Err(AssemblerError::InvalidOperand("Expected register".to_string())),
        }
    }

    fn parse_immediate(&self, operand: &Operand) -> Result<i32, AssemblerError> {
        match operand {
            Operand::Immediate(value) => Ok(*value),
            _ => Err(AssemblerError::InvalidOperand("Expected immediate value".to_string())),
        }
    }

    fn parse_memory_reference(&self, operand: &Operand) -> Result<(i32, u32), AssemblerError> {
        match operand {
            Operand::MemoryReference { offset, register } => {
                let reg = self.parse_register(&Operand::Register(register.clone()))?;
                Ok((*offset, reg))
            }
            _ => Err(AssemblerError::InvalidOperand("Expected memory reference".to_string())),
        }
    }

    fn parse_label(&self, operand: &Operand) -> Result<u32, AssemblerError> {
        match operand {
            Operand::Label(label) => {
                self.labels.get(label)
                    .copied()
                    .ok_or_else(|| AssemblerError::UndefinedLabel(label.clone()))
            }
            _ => Err(AssemblerError::InvalidOperand("Expected label".to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssembledProgram {
    pub code: Vec<u8>,
    pub entry_point: u32,
    pub symbols: Vec<Symbol>,
}

impl AssembledProgram {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            entry_point: 0,
            symbols: Vec::new(),
        }
    }
}

impl Default for AssembledProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub address: u32,
    pub size: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum AssemblerError {
    #[error("Unknown instruction: {0}")]
    UnknownInstruction(String),
    #[error("Invalid operand: {0}")]
    InvalidOperand(String),
    #[error("Undefined label: {0}")]
    UndefinedLabel(String),
}
