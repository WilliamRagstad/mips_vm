//! MIPS instruction definitions - Adapted from existing mips-vm logic

/// Represents the different kinds of MIPS instructions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstructionKind {
    // Arithmetic instructions
    Add, Addi, Addiu, Addu,
    Sub, Subu,
    Mult, Multu, Div, Divu,
    
    // Logical instructions
    And, Andi,
    Or, Ori,
    Xor, Xori,
    Nor,
    
    // Comparison instructions
    Slt, Sltu, Slti, Sltiu,
    
    // Shift instructions
    Sll, Sllv,
    Srl, Srlv,
    Sra, Srav,
    
    // Branch instructions
    Beq, Bne,
    Blez, Bgtz,
    
    // Jump instructions
    J, Jal, Jr, Jalr,
    
    // Load instructions
    Lb, Lbu, Lh, Lhu, Lw,
    Lui,
    
    // Store instructions
    Sb, Sh, Sw,
    
    // Pseudo instructions
    Li, La, Move, Nop,
    
    // System instructions
    Syscall,
}

impl InstructionKind {
    /// Get the string representation of the instruction
    pub fn mnemonic(&self) -> &'static str {
        match self {
            InstructionKind::Add => "add",
            InstructionKind::Addi => "addi",
            InstructionKind::Addiu => "addiu",
            InstructionKind::Addu => "addu",
            InstructionKind::Sub => "sub",
            InstructionKind::Subu => "subu",
            InstructionKind::Mult => "mult",
            InstructionKind::Multu => "multu",
            InstructionKind::Div => "div",
            InstructionKind::Divu => "divu",
            InstructionKind::And => "and",
            InstructionKind::Andi => "andi",
            InstructionKind::Or => "or",
            InstructionKind::Ori => "ori",
            InstructionKind::Xor => "xor",
            InstructionKind::Xori => "xori",
            InstructionKind::Nor => "nor",
            InstructionKind::Slt => "slt",
            InstructionKind::Sltu => "sltu",
            InstructionKind::Slti => "slti",
            InstructionKind::Sltiu => "sltiu",
            InstructionKind::Sll => "sll",
            InstructionKind::Sllv => "sllv",
            InstructionKind::Srl => "srl",
            InstructionKind::Srlv => "srlv",
            InstructionKind::Sra => "sra",
            InstructionKind::Srav => "srav",
            InstructionKind::Beq => "beq",
            InstructionKind::Bne => "bne",
            InstructionKind::Blez => "blez",
            InstructionKind::Bgtz => "bgtz",
            InstructionKind::J => "j",
            InstructionKind::Jal => "jal",
            InstructionKind::Jr => "jr",
            InstructionKind::Jalr => "jalr",
            InstructionKind::Lb => "lb",
            InstructionKind::Lbu => "lbu",
            InstructionKind::Lh => "lh",
            InstructionKind::Lhu => "lhu",
            InstructionKind::Lw => "lw",
            InstructionKind::Lui => "lui",
            InstructionKind::Sb => "sb",
            InstructionKind::Sh => "sh",
            InstructionKind::Sw => "sw",
            InstructionKind::Li => "li",
            InstructionKind::La => "la",
            InstructionKind::Move => "move",
            InstructionKind::Nop => "nop",
            InstructionKind::Syscall => "syscall",
        }
    }

    /// Parse instruction kind from mnemonic string
    pub fn from_mnemonic(mnemonic: &str) -> Option<Self> {
        match mnemonic.to_lowercase().as_str() {
            "add" => Some(InstructionKind::Add),
            "addi" => Some(InstructionKind::Addi),
            "addiu" => Some(InstructionKind::Addiu),
            "addu" => Some(InstructionKind::Addu),
            "sub" => Some(InstructionKind::Sub),
            "subu" => Some(InstructionKind::Subu),
            "mult" => Some(InstructionKind::Mult),
            "multu" => Some(InstructionKind::Multu),
            "div" => Some(InstructionKind::Div),
            "divu" => Some(InstructionKind::Divu),
            "and" => Some(InstructionKind::And),
            "andi" => Some(InstructionKind::Andi),
            "or" => Some(InstructionKind::Or),
            "ori" => Some(InstructionKind::Ori),
            "xor" => Some(InstructionKind::Xor),
            "xori" => Some(InstructionKind::Xori),
            "nor" => Some(InstructionKind::Nor),
            "slt" => Some(InstructionKind::Slt),
            "sltu" => Some(InstructionKind::Sltu),
            "slti" => Some(InstructionKind::Slti),
            "sltiu" => Some(InstructionKind::Sltiu),
            "sll" => Some(InstructionKind::Sll),
            "sllv" => Some(InstructionKind::Sllv),
            "srl" => Some(InstructionKind::Srl),
            "srlv" => Some(InstructionKind::Srlv),
            "sra" => Some(InstructionKind::Sra),
            "srav" => Some(InstructionKind::Srav),
            "beq" => Some(InstructionKind::Beq),
            "bne" => Some(InstructionKind::Bne),
            "blez" => Some(InstructionKind::Blez),
            "bgtz" => Some(InstructionKind::Bgtz),
            "j" => Some(InstructionKind::J),
            "jal" => Some(InstructionKind::Jal),
            "jr" => Some(InstructionKind::Jr),
            "jalr" => Some(InstructionKind::Jalr),
            "lb" => Some(InstructionKind::Lb),
            "lbu" => Some(InstructionKind::Lbu),
            "lh" => Some(InstructionKind::Lh),
            "lhu" => Some(InstructionKind::Lhu),
            "lw" => Some(InstructionKind::Lw),
            "lui" => Some(InstructionKind::Lui),
            "sb" => Some(InstructionKind::Sb),
            "sh" => Some(InstructionKind::Sh),
            "sw" => Some(InstructionKind::Sw),
            "li" => Some(InstructionKind::Li),
            "la" => Some(InstructionKind::La),
            "move" => Some(InstructionKind::Move),
            "nop" => Some(InstructionKind::Nop),
            "syscall" => Some(InstructionKind::Syscall),
            _ => None,
        }
    }

    /// Check if this is a pseudo instruction
    pub fn is_pseudo(&self) -> bool {
        matches!(self, 
            InstructionKind::Li | 
            InstructionKind::La | 
            InstructionKind::Move | 
            InstructionKind::Nop
        )
    }

    /// Get the instruction format
    pub fn format(&self) -> crate::InstructionFormat {
        match self {
            // R-type instructions
            InstructionKind::Add | InstructionKind::Addu | InstructionKind::Sub | InstructionKind::Subu |
            InstructionKind::And | InstructionKind::Or | InstructionKind::Xor | InstructionKind::Nor |
            InstructionKind::Slt | InstructionKind::Sltu | InstructionKind::Sll | InstructionKind::Srl |
            InstructionKind::Sra | InstructionKind::Sllv | InstructionKind::Srlv | InstructionKind::Srav |
            InstructionKind::Jr | InstructionKind::Jalr | InstructionKind::Mult | InstructionKind::Multu |
            InstructionKind::Div | InstructionKind::Divu => crate::InstructionFormat::R,
            
            // J-type instructions
            InstructionKind::J | InstructionKind::Jal => crate::InstructionFormat::J,
            
            // I-type instructions (and pseudo instructions)
            _ => crate::InstructionFormat::I,
        }
    }
}
