use std::sync::RwLock;

use crate::{
    address::Address,
    memory::LabelMap,
    program::{Immediate, Instruction, InstructionKind},
    registers::Register,
};

pub type EncodedInstruction = u32;

pub struct RegisterFormat {
    pub rd: Option<Register>,
    pub rs: Option<Register>,
    pub rt: Option<Register>,
    pub shamt: Option<u8>,
}

pub struct ImmediateFormat {
    pub rt: Option<Register>,
    pub rs: Option<Register>,
    pub imm: Immediate,
}

pub struct JumpFormat {
    pub address: Address,
}

pub enum InstructionFormat {
    Register(RegisterFormat),
    Immediate(ImmediateFormat),
    Jump(JumpFormat),
}

impl InstructionFormat {
    pub fn register(
        rd: Option<Register>,
        rs: Option<Register>,
        rt: Option<Register>,
        shamt: Option<u8>,
    ) -> Self {
        Self::Register(RegisterFormat { rd, rs, rt, shamt })
    }

    pub fn immediate(rt: Option<Register>, rs: Option<Register>, imm: Immediate) -> Self {
        Self::Immediate(ImmediateFormat { rt, rs, imm })
    }

    pub fn jump(address: Address) -> Self {
        Self::Jump(JumpFormat { address })
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Self::Register { .. })
    }

    pub fn unwrap_register(&self) -> &RegisterFormat {
        match self {
            Self::Register(r) => r,
            _ => panic!("Expected register format"),
        }
    }

    pub fn is_immediate(&self) -> bool {
        matches!(self, Self::Immediate { .. })
    }

    pub fn unwrap_immediate(&self) -> &ImmediateFormat {
        match self {
            Self::Immediate(i) => i,
            _ => panic!("Expected immediate format"),
        }
    }

    pub fn is_jump(&self) -> bool {
        matches!(self, Self::Jump { .. })
    }

    pub fn unwrap_jump(&self) -> &JumpFormat {
        match self {
            Self::Jump(j) => j,
            _ => panic!("Expected jump format"),
        }
    }
}

pub struct InstructionInfo {
    pub opcode: u8,
    pub funct: u8,
    pub format: InstructionFormat,
}

impl InstructionInfo {
    pub fn new(format: InstructionFormat, opcode: u8, funct: u8) -> Self {
        Self {
            opcode,
            funct,
            format,
        }
    }
}
pub fn assemble_all(instructions: &[Instruction], labels: &LabelMap) -> Vec<EncodedInstruction> {
    instructions
        .into_iter()
        .map(|i| encode_instruction(i, labels))
        .collect()
}

pub fn info(instruction: &Instruction, labels: &LabelMap) -> InstructionInfo {
    let args = RwLock::new(instruction.args.iter());
    let next = || args.write().unwrap().next();
    let reg = || next().map(|arg| arg.clone().as_register().unwrap());
    let imm = || {
        next()
            .map(|arg| arg.clone().as_immediate().unwrap())
            .expect("Expected immediate argument")
    };
    let addr = || {
        next()
            .map(|arg| {
                let label = arg.clone().as_label().unwrap();
                *labels.get(&label).expect("Expected label argument")
            })
            .expect("Expected address argument")
    };
    let offset = || {
        next()
            .map(|arg| arg.clone().as_offset().unwrap())
            .expect("Expected offset argument")
    };
    match instruction.kind {
        // Arithmetic Logical Unit
        InstructionKind::Add => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x20,
        ),
        InstructionKind::Addi => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 8, 0)
        }
        InstructionKind::Addiu => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 9, 0)
        }
        InstructionKind::Addu => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x21,
        ),
        InstructionKind::Sub => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x22,
        ),
        InstructionKind::Subu => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x23,
        ),
        InstructionKind::And => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x24,
        ),
        InstructionKind::Andi => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 0xC, 0)
        }

        InstructionKind::Nor => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x27,
        ),
        InstructionKind::Or => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x25,
        ),
        InstructionKind::Ori => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 0x0D, 0)
        }
        InstructionKind::Slt => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x2A,
        ),
        InstructionKind::Slti => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 0x0A, 0)
        }
        InstructionKind::Sltiu => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 0x0B, 0)
        }
        InstructionKind::Sltu => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x2B,
        ),
        InstructionKind::Xor => InstructionInfo::new(
            InstructionFormat::register(reg(), reg(), reg(), None),
            0,
            0x26,
        ),
        InstructionKind::Xori => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 0x0E, 0)
        }

        // Shifter
        InstructionKind::Sll => InstructionInfo::new(
            InstructionFormat::register(reg(), None, reg(), Some(imm() as u8)),
            0,
            0,
        ),
        InstructionKind::Sllv => {
            // Note the order or registers is different from the other instructions
            let rd = reg();
            let rt = reg();
            let rs = reg();
            InstructionInfo::new(InstructionFormat::register(rd, rs, rt, None), 0, 0x04)
        }
        InstructionKind::Sra => InstructionInfo::new(
            InstructionFormat::register(reg(), None, reg(), Some(imm() as u8)),
            0,
            0x03,
        ),
        InstructionKind::Srav => {
            // Note the order or registers is different from the other instructions
            let rd = reg();
            let rt = reg();
            let rs = reg();
            InstructionInfo::new(InstructionFormat::register(rd, rs, rt, None), 0, 0x07)
        }

        InstructionKind::Srl => InstructionInfo::new(
            InstructionFormat::register(reg(), None, reg(), Some(imm() as u8)),
            0,
            0x02,
        ),
        InstructionKind::Srlv => {
            // Note the order or registers is different from the other instructions
            let rd = reg();
            let rt = reg();
            let rs = reg();
            InstructionInfo::new(InstructionFormat::register(rd, rs, rt, None), 0, 0x06)
        }

        // Multiply and Divide
        InstructionKind::Div => InstructionInfo::new(
            InstructionFormat::register(None, reg(), reg(), None),
            0,
            0x1A,
        ),
        InstructionKind::Divu => InstructionInfo::new(
            InstructionFormat::register(None, reg(), reg(), None),
            0,
            0x1B,
        ),
        InstructionKind::Mult => {
            let rd = reg();
            let rs = reg();
            let rt = reg();
            InstructionInfo::new(InstructionFormat::register(rd, rs, rt, None), 0, 0x18)
        }
        InstructionKind::Multu => {
            let rd = reg();
            let rs = reg();
            let rt = reg();
            InstructionInfo::new(InstructionFormat::register(rd, rs, rt, None), 0, 0x19)
        }

        // Branch
        InstructionKind::Beq => InstructionInfo::new(
            InstructionFormat::immediate(reg(), reg(), addr().unwrap() as u16),
            4,
            0,
        ),
        InstructionKind::Blez => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), None, imm()), 6, 0)
        }
        InstructionKind::Bne => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), reg(), imm()), 5, 0)
        }
        InstructionKind::Bgtz => {
            InstructionInfo::new(InstructionFormat::immediate(reg(), None, imm()), 7, 0)
        }
        InstructionKind::J => InstructionInfo::new(InstructionFormat::jump(addr()), 2, 0),
        InstructionKind::Jal => InstructionInfo::new(InstructionFormat::jump(addr()), 3, 0),
        InstructionKind::Jalr => InstructionInfo::new(
            InstructionFormat::register(None, reg(), None, None),
            0,
            0x09,
        ),
        InstructionKind::Jr => InstructionInfo::new(
            InstructionFormat::register(None, reg(), None, None),
            0,
            0x08,
        ),

        // Memory Access
        InstructionKind::Lb => {
            let rt = reg();
            let (offset, rs) = offset();

            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x20, 0)
        }
        InstructionKind::Lbu => {
            let rt = reg();
            let (offset, rs) = offset();
            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x24, 0)
        }
        InstructionKind::Lh => {
            let rt = reg();
            let (offset, rs) = offset();
            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x21, 0)
        }
        InstructionKind::Lhu => {
            let rt = reg();
            let (offset, rs) = offset();
            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x25, 0)
        }
        InstructionKind::Lui => {
            let rt = reg();
            let imm = imm();
            InstructionInfo::new(InstructionFormat::immediate(None, rt, imm), 0xF, 0)
        }
        InstructionKind::Lw => {
            let rt = reg();
            let (offset, rs) = offset();
            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x23, 0)
        }
        InstructionKind::Sb => {
            let rt = reg();
            let (offset, rs) = offset();
            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x28, 0)
        }
        InstructionKind::Sh => {
            let rt = reg();
            let (offset, rs) = offset();
            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x29, 0)
        }
        InstructionKind::Sw => {
            let rt = reg();
            let (offset, rs) = offset();
            InstructionInfo::new(InstructionFormat::immediate(Some(rs), rt, offset), 0x2B, 0)
        }

        // Psuedo instructions
        // TODO: Replace with actual instructions in parser
        InstructionKind::La => InstructionInfo::new(
            InstructionFormat::immediate(None, reg(), addr().unwrap() as u16),
            0,
            0,
        ),
        InstructionKind::Li => {
            InstructionInfo::new(InstructionFormat::immediate(None, reg(), imm()), 0, 0)
        }
        InstructionKind::Move => {
            InstructionInfo::new(InstructionFormat::register(reg(), reg(), None, None), 0, 0)
        }
        InstructionKind::Nop => {
            InstructionInfo::new(InstructionFormat::register(None, None, None, None), 0, 0)
        }
        InstructionKind::Syscall => {
            InstructionInfo::new(InstructionFormat::register(None, None, None, None), 0, 0x0C)
        }
    }
}

pub fn encode_instruction(instruction: &Instruction, labels: &LabelMap) -> EncodedInstruction {
    let info = info(instruction, labels);
    if info.format.is_register() {
        encode_register_type(&info, info.format.unwrap_register())
    } else if info.format.is_immediate() {
        encode_immediate_type(&info, info.format.unwrap_immediate())
    } else {
        encode_jump_type(&info, info.format.unwrap_jump())
    }
}

/// Encode R-type instruction.
///
/// Sizes of fields (bits):
///
/// | `opcode` |  `rs`  | `rt` | `rd` | `shamt` | `funct` |
/// |:--------:|:------:|:----:|:----:|:-------:|:-------:|
/// |    6     |   5    |  5   |  5   |    5    |    6    |
pub fn encode_register_type(info: &InstructionInfo, args: &RegisterFormat) -> EncodedInstruction {
    let mut opcode = info.opcode as u32;
    let mut rs = args.rs.as_ref().map(Register::encode).unwrap_or(0) as u32;
    let mut rt = args.rt.as_ref().map(Register::encode).unwrap_or(0) as u32;
    let mut rd = args.rd.as_ref().map(Register::encode).unwrap_or(0) as u32;
    let mut shamt = args.shamt.unwrap_or(0) as u32;
    let mut funct = info.funct as u32;

    assert!(funct < 2 << 6);
    assert!(shamt < 2 << 5);
    assert!(rd < 2 << 5);
    assert!(rt < 2 << 5);
    assert!(rs < 2 << 5);
    assert!(opcode < 2 << 6);

    funct <<= 0;
    shamt <<= 6;
    rd <<= 6 + 5;
    rt <<= 6 + 5 + 5;
    rs <<= 6 + 5 + 5 + 5;
    opcode <<= 6 + 5 + 5 + 5 + 5;

    opcode | rs | rt | rd | shamt | funct
}

/// Encode I-type instruction.
///
/// Sizes of fields (bits):
///     
/// | `opcode` |  `rs`  | `rt` |       `immediate`       |
/// |:--------:|:------:|:----:|:-----------------------:|
/// |    6     |   5    |  5   |           16            |
pub fn encode_immediate_type(info: &InstructionInfo, args: &ImmediateFormat) -> EncodedInstruction {
    let mut immediate = args.imm as u32;
    let mut rt = args.rt.as_ref().map(Register::encode).unwrap_or(0) as u32;
    let mut rs = args.rs.as_ref().map(Register::encode).unwrap_or(0) as u32;
    let mut opcode = info.opcode as u32;

    assert!(immediate < 2 << 16);
    assert!(rt < 2 << 5);
    assert!(rs < 2 << 5);
    assert!(opcode < 2 << 6);

    immediate <<= 0;
    rt <<= 16;
    rs <<= 16 + 5;
    opcode <<= 16 + 5 + 5;

    opcode | rs | rt | immediate
}

/// Encode J-type instruction.
///
/// Sizes of fields (bits):
///
/// | `opcode` |          `address`          |
/// |:--------:|:---------------------------:|
/// |    6     |              26             |
pub fn encode_jump_type(info: &InstructionInfo, args: &JumpFormat) -> EncodedInstruction {
    let mut address = args.address.unwrap();
    let mut opcode = info.opcode as u32;

    assert!(address < 2 << 26);
    assert!(opcode < 2 << 6);

    address <<= 0;
    opcode <<= 26;

    opcode | address
}
