use colorful::{Color, Colorful};
use std::collections::HashMap;

pub type Address = u32;
pub type Word = u32;

#[derive(Debug, PartialEq)]
pub enum Section {
    Data,
    Text,
}

impl Section {
    pub fn show(&self) -> &str {
        match self {
            Section::Data => ".data",
            Section::Text => ".text",
        }
    }

    pub fn show_color(&self) -> String {
        match self {
            Section::Data => ".data".color(Color::HotPink2).to_string(),
            Section::Text => ".text".color(Color::HotPink2).to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RawData {
    pub source: String,
    pub address: Address,
    pub data: Vec<u8>,
}

impl RawData {
    pub fn show(&self) -> String {
        self.source.clone()
    }

    pub fn show_color(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("{}\n", self.source.clone().yellow()));
        // Actual data in comment # ... in gray
        result.push_str(&"# ".dark_gray().to_string());
        for byte in &self.data {
            result.push_str(&format!("{:02x} ", byte).dark_gray().to_string());
        }
        result
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

#[derive(Debug, PartialEq)]
pub struct DataSection {
    pub globals: HashMap<String, RawData>,
}

impl DataSection {
    pub fn show(&self) -> String {
        let mut result = String::new();
        for (label, directive) in &self.globals {
            result.push_str(&format!("{}: {}\n", label, directive.show()));
        }
        result
    }

    pub fn show_color(&self) -> String {
        let mut result = String::new();
        for (label, directive) in &self.globals {
            result.push_str(&format!(
                "{}: {}\n",
                label.clone().light_green(),
                directive.show_color()
            ));
        }
        result
    }

    pub fn find_by_address(&self, address: Address) -> Option<&RawData> {
        self.globals
            .values()
            .find(|&directive| directive.address() == address)
    }
}

#[derive(Debug, PartialEq)]
pub enum InstructionKind {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Nor,
    Slt,
    Sll,
    Srl,
    Sra,
    Jr,
    Beq,
    Bne,
    Lw,
    Sw,
    Li,
    Move,
    La,
    B,
    J,
    Jal,
    Syscall,
}

impl InstructionKind {
    pub fn show(&self) -> &str {
        match self {
            InstructionKind::Add => "add",
            InstructionKind::Sub => "sub",
            InstructionKind::Mul => "mul",
            InstructionKind::Div => "div",
            InstructionKind::And => "and",
            InstructionKind::Or => "or",
            InstructionKind::Xor => "xor",
            InstructionKind::Nor => "nor",
            InstructionKind::Slt => "slt",
            InstructionKind::Sll => "sll",
            InstructionKind::Srl => "srl",
            InstructionKind::Sra => "sra",
            InstructionKind::Jr => "jr",
            InstructionKind::Beq => "beq",
            InstructionKind::Bne => "bne",
            InstructionKind::Lw => "lw",
            InstructionKind::Sw => "sw",
            InstructionKind::Li => "li",
            InstructionKind::Move => "move",
            InstructionKind::La => "la",
            InstructionKind::B => "b",
            InstructionKind::J => "j",
            InstructionKind::Jal => "jal",
            InstructionKind::Syscall => "syscall",
        }
    }
}

impl From<&str> for InstructionKind {
    fn from(s: &str) -> InstructionKind {
        match s {
            "add" => InstructionKind::Add,
            "sub" => InstructionKind::Sub,
            "mul" => InstructionKind::Mul,
            "div" => InstructionKind::Div,
            "and" => InstructionKind::And,
            "or" => InstructionKind::Or,
            "xor" => InstructionKind::Xor,
            "nor" => InstructionKind::Nor,
            "slt" => InstructionKind::Slt,
            "sll" => InstructionKind::Sll,
            "srl" => InstructionKind::Srl,
            "sra" => InstructionKind::Sra,
            "jr" => InstructionKind::Jr,
            "beq" => InstructionKind::Beq,
            "bne" => InstructionKind::Bne,
            "lw" => InstructionKind::Lw,
            "sw" => InstructionKind::Sw,
            "li" => InstructionKind::Li,
            "move" => InstructionKind::Move,
            "la" => InstructionKind::La,
            "b" => InstructionKind::B,
            "j" => InstructionKind::J,
            "jal" => InstructionKind::Jal,
            "syscall" => InstructionKind::Syscall,
            _ => panic!("Invalid instruction: {}", s),
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum Register {
    Zero,
    At,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    K0,
    K1,
    Gp,
    Sp,
    Fp,
    Ra,
}

impl Register {
    pub fn show(&self) -> &str {
        match self {
            Register::Zero => "$zero",
            Register::At => "$at",
            Register::V0 => "$v0",
            Register::V1 => "$v1",
            Register::A0 => "$a0",
            Register::A1 => "$a1",
            Register::A2 => "$a2",
            Register::A3 => "$a3",
            Register::T0 => "$t0",
            Register::T1 => "$t1",
            Register::T2 => "$t2",
            Register::T3 => "$t3",
            Register::T4 => "$t4",
            Register::T5 => "$t5",
            Register::T6 => "$t6",
            Register::T7 => "$t7",
            Register::S0 => "$s0",
            Register::S1 => "$s1",
            Register::S2 => "$s2",
            Register::S3 => "$s3",
            Register::S4 => "$s4",
            Register::S5 => "$s5",
            Register::S6 => "$s6",
            Register::S7 => "$s7",
            Register::T8 => "$t8",
            Register::T9 => "$t9",
            Register::K0 => "$k0",
            Register::K1 => "$k1",
            Register::Gp => "$gp",
            Register::Sp => "$sp",
            Register::Fp => "$fp",
            Register::Ra => "$ra",
        }
    }
}

impl From<&str> for Register {
    fn from(s: &str) -> Register {
        match s {
            "$zero" => Register::Zero,
            "$at" => Register::At,
            "$v0" => Register::V0,
            "$v1" => Register::V1,
            "$a0" => Register::A0,
            "$a1" => Register::A1,
            "$a2" => Register::A2,
            "$a3" => Register::A3,
            "$t0" => Register::T0,
            "$t1" => Register::T1,
            "$t2" => Register::T2,
            "$t3" => Register::T3,
            "$t4" => Register::T4,
            "$t5" => Register::T5,
            "$t6" => Register::T6,
            "$t7" => Register::T7,
            "$s0" => Register::S0,
            "$s1" => Register::S1,
            "$s2" => Register::S2,
            "$s3" => Register::S3,
            "$s4" => Register::S4,
            "$s5" => Register::S5,
            "$s6" => Register::S6,
            "$s7" => Register::S7,
            "$t8" => Register::T8,
            "$t9" => Register::T9,
            "$k0" => Register::K0,
            "$k1" => Register::K1,
            "$gp" => Register::Gp,
            "$sp" => Register::Sp,
            "$fp" => Register::Fp,
            "$ra" => Register::Ra,
            _ => panic!("Invalid register: {}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InstructionArg {
    Register(Register),
    Immediate(i32),
    Label(String),
}

impl InstructionArg {
    pub fn show(&self) -> String {
        match self {
            InstructionArg::Register(r) => r.show().to_string(),
            InstructionArg::Immediate(i) => i.to_string(),
            InstructionArg::Label(l) => l.to_string(),
        }
    }

    pub fn show_color(&self) -> String {
        match self {
            InstructionArg::Register(r) => r.show().color(Color::Orange1).to_string(),
            InstructionArg::Immediate(i) => i.to_string().magenta().to_string(),
            InstructionArg::Label(l) => l.to_string().light_green().to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub address: Address,
    pub kind: InstructionKind,
    pub args: Vec<InstructionArg>,
}

impl Instruction {
    pub fn show(&self) -> String {
        let mut result = self.kind.show().to_string();
        for (i, arg) in self.args.iter().enumerate() {
            if i == 0 {
                result.push(' ');
            } else {
                result.push_str(", ");
            }
            result.push_str(&arg.show());
        }
        result
    }

    pub fn show_color(&self) -> String {
        let mut result = self.kind.show().light_cyan().to_string();
        for (i, arg) in self.args.iter().enumerate() {
            if i == 0 {
                result.push(' ');
            } else {
                result.push_str(", ");
            }
            result.push_str(&arg.show_color());
        }
        result
    }

    pub fn size(&self) -> usize {
        4
    }
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub address: Address,
    pub label: String,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn show(&self) -> String {
        let mut result = format!("{}:\n", self.label);
        for instruction in &self.instructions {
            result.push_str(&format!("    {}\n", instruction.show()));
        }
        result
    }

    pub fn show_color(&self) -> String {
        let mut result = format!("{}:\n", self.label.clone().light_green());
        for instruction in &self.instructions {
            result.push_str(&format!("    {}\n", instruction.show_color()));
        }
        result
    }
}

#[derive(Debug, PartialEq)]
pub struct TextSection {
    pub blocks: Vec<Block>,
    pub global_labels: Vec<String>,
}

impl TextSection {
    pub fn show(&self) -> String {
        let mut result = String::new();
        for label in &self.global_labels {
            result.push_str(&format!(".global {}\n", label));
        }
        for block in &self.blocks {
            result.push_str(&block.show());
        }
        result
    }

    pub fn show_color(&self) -> String {
        let mut result = String::new();
        for label in &self.global_labels {
            result.push_str(&format!(
                "{} {}\n",
                ".global".color(Color::HotPink2),
                label.clone().light_green()
            ));
        }
        for block in &self.blocks {
            result.push_str(&block.show_color());
        }
        result
    }
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub data: DataSection,
    pub text: TextSection,
}

impl Program {
    pub fn show(&self) -> String {
        // Data
        let mut result = format!("{}\n", Section::Data.show());
        result.push_str(&self.data.show());
        // Text
        result.push_str(&format!("\n{}\n", Section::Text.show()));
        result.push_str(&self.text.show());
        result
    }

    pub fn show_color(&self) -> String {
        // Data
        let mut result = format!("{}\n", Section::Data.show_color());
        result.push_str(&self.data.show_color());
        // Text
        result.push_str(&format!("\n{}\n", Section::Text.show_color()));
        result.push_str(&self.text.show_color());
        result
    }
}
