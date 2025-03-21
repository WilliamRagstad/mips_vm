use crate::registers::Register;
use colorful::{Color, Colorful};

/// Represents an address in a MIPS program.
pub type Address = u32;

/// Represents a 32 bits long word in a MIPS program.
pub type Word = u32;

/// Represents a 16 bits literal immediate value in a MIPS program.
pub type Immediate = u16;

pub const DIRECTIVE_COLOR: Color = Color::LightRed;
pub const LABEL_COLOR: Color = Color::LightGreen;
pub const REGISTER_COLOR: Color = Color::Orange1;
pub const IMMEDIATE_COLOR: Color = Color::Magenta;
pub const INSTRUCTION_COLOR: Color = Color::LightCyan;
pub const DATA_SOURCE_COLOR: Color = Color::Yellow;
pub const DATA_BYTES_COLOR: Color = Color::DarkGray;

/// Represents the different sections of a MIPS program.
#[derive(Debug, PartialEq)]
pub enum Section {
    /// The data section, which contains global and static data.
    Data,
    /// The text section, which contains the executable instructions.
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
            Section::Data => ".data".color(DIRECTIVE_COLOR).to_string(),
            Section::Text => ".text".color(DIRECTIVE_COLOR).to_string(),
        }
    }
}

/// Represents raw data in the data section.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticData {
    /// The source code of the data directive.
    pub source: String,
    /// The label of the data directive.
    pub label: String,
    /// The actual data bytes.
    pub data: Vec<u8>,
}

impl StaticData {
    pub fn show(&self) -> String {
        format!("{}: {}\n", self.label, self.source)
    }

    pub fn show_color(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!(
            "{}: {}\n",
            self.label.clone().color(LABEL_COLOR),
            self.source.clone().color(DATA_SOURCE_COLOR)
        ));
        // Actual data in comment # ... in gray
        result.push_str(&"# ".color(DATA_BYTES_COLOR).to_string());
        for byte in &self.data {
            result.push_str(&format!("{:02x} ", byte).color(DATA_BYTES_COLOR).to_string());
        }
        result
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Represents the data section of a MIPS program.
#[derive(Debug, PartialEq)]
pub struct DataSection {
    /// Initialized data.
    /// A list of global labels to their corresponding raw data.
    pub initialized: Vec<StaticData>,
}

impl DataSection {
    pub fn empty(&self) -> bool {
        self.initialized.is_empty()
    }

    pub fn initialized(&self) -> Vec<&StaticData> {
        self.initialized.iter().collect()
    }

    pub fn initialized_move(self) -> Vec<StaticData> {
        self.initialized
    }

    pub fn show(&self) -> String {
        let mut result = String::new();
        for data in &self.initialized {
            result.push_str(&data.show());
        }
        result
    }

    pub fn show_color(&self) -> String {
        let mut result = String::new();
        for data in &self.initialized {
            result.push_str(&data.show_color());
        }
        result
    }
}

/// Represents the different kinds of MIPS instructions.
#[derive(Clone, Debug, PartialEq)]
pub enum InstructionKind {
    /// Add two registers and store the result in a register.
    ///
    /// Syntax: `add $d, $s, $t`
    ///
    /// Description: `$d = $s + $t`
    Add,
    /// Add an immediate value to a register and store the result in a register.
    ///
    /// Syntax: `addi $d, $s, immediate`
    ///
    /// Description: `$d = $s + immediate`
    Addi,
    /// Subtract one register from another and store the result in a register.
    ///
    /// Syntax: `sub $d, $s, $t`
    ///
    /// Description: `$d = $s - $t`
    Sub,
    /// Multiply two registers and store the result in a register.
    ///
    /// Syntax: `mul $d, $s, $t`
    ///
    /// Description: `$d = $s * $t`
    Mul,
    /// Divide one register by another and store the result in a register.
    ///
    /// Syntax: `div $d, $s, $t`
    ///
    /// Description: `$d = $s / $t`
    Div,
    /// Perform a bitwise AND on two registers and store the result in a register.
    ///
    /// Syntax: `and $d, $s, $t`
    ///
    /// Description: `$d = $s & $t`
    And,
    /// Perform a bitwise ANDI on two registers and store the result in a register.
    ///
    /// Syntax: `andi $d, $s, immediate`
    ///
    /// Description: `$d = $s & immediate`
    Andi,
    /// Perform a bitwise OR on two registers and store the result in a register.
    ///
    /// Syntax: `or $d, $s, $t`
    ///
    /// Description: `$d = $s | $t`
    Or,
    /// Perform a bitwise XOR on two registers and store the result in a register.
    ///
    /// Syntax: `xor $d, $s, $t`
    ///
    /// Description: `$d = $s ^ $t`
    Xor,
    /// Perform a bitwise NOR on two registers and store the result in a register.
    ///
    /// Syntax: `nor $d, $s, $t`
    ///
    /// Description: `$d = ~($s | $t)`
    Nor,
    /// Set a register to 1 if one register is less than another, otherwise set it to 0.
    ///
    /// Syntax: `slt $d, $s, $t`
    ///
    /// Description: `$d = ($s < $t) ? 1 : 0`
    Slt,
    /// Shift a register left by a specified number of bits and store the result in a register.
    ///
    /// Syntax: `sll $d, $t, shamt`
    ///
    /// Description: `$d = $t << shamt`
    Sll,
    /// Shift a register right by a specified number of bits and store the result in a register.
    ///
    /// Syntax: `srl $d, $t, shamt`
    ///
    /// Description: `$d = $t >> shamt`
    Srl,
    /// Shift a register right by a specified number of bits with sign extension and store the result in a register.
    ///
    /// Syntax: `sra $d, $t, shamt`
    ///
    /// Description: `$d = $t >> shamt` (arithmetic shift)
    Sra,
    /// Branch if two registers are equal.
    ///
    /// Syntax: `beq $s, $t, offset`
    ///
    /// Description: `if ($s == $t) branch to address PC + 4 + (offset * 4)`
    Beq,
    /// Branch if two registers are not equal.
    ///
    /// Syntax: `bne $s, $t, offset`
    ///
    /// Description: `if ($s != $t) branch to address PC + 4 + (offset * 4)`
    Bne,
    /// Load a word from memory into a register.
    ///
    /// Syntax: `lw $t, offset($s)`
    ///
    /// Description: `$t = Memory[$s + offset]`
    Lw,
    /// Store a word from a register into memory.
    ///
    /// Syntax: `sw $t, offset($s)`
    ///
    /// Description: `Memory[$s + offset] = $t`
    Sw,
    /// Load an immediate value into a register.
    /// Use this when you want to put an integer value into a register.
    ///
    /// Syntax: `li $t, immediate`
    ///
    /// Description: `$t = immediate`
    Li,
    /// Load an upper immediate value into a register.
    ///
    /// Syntax: `lui $t, immediate`
    ///
    /// Description: `$t = immediate << 16`
    Lui,
    /// Load the address of a label into a register.
    /// Use this when you want to put an address value into a register.
    ///
    /// Syntax: `la $t, label`
    ///
    /// Description: `$t = address of label`
    ///
    /// Where `label` is pre-defined for something in memory (defined under the `.data` directive).
    La,
    /// Unconditional branch to a label.
    ///
    /// Syntax: `b label`
    ///
    /// Description: `branch to address of label`
    B,
    /// Jump to a label.
    ///
    /// Syntax: `j label`
    ///
    /// Description: `jump to address of label`
    J,
    /// Jump to the address contained in a register.
    ///
    /// Syntax: `jr $s`
    ///
    /// Description: `jump to address in $s`
    Jr,
    /// Jump and link to a label (store return address in $ra).
    ///
    /// Syntax: `jal label`
    ///
    /// Description: `$ra = PC + 4; jump to address of label`
    Jal,
    /// Move a value from one register to another.
    ///
    /// Syntax: `move $d, $s`
    ///
    /// Description: `$d = $s`
    Move,
    /// Do nothing.
    ///
    /// Syntax: `nop`
    ///
    /// Description: `do nothing`
    Nop,
    /// Perform a system call.
    ///
    /// Syntax: `syscall`
    ///
    /// Description: `perform a system call`
    Syscall,
}

impl InstructionKind {
    pub fn show(&self) -> &str {
        match self {
            InstructionKind::Add => "add",
            InstructionKind::Addi => "addi",
            InstructionKind::Sub => "sub",
            InstructionKind::Mul => "mul",
            InstructionKind::Div => "div",
            InstructionKind::And => "and",
            InstructionKind::Andi => "andi",
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
            InstructionKind::Lui => "lui",
            InstructionKind::Move => "move",
            InstructionKind::La => "la",
            InstructionKind::B => "b",
            InstructionKind::J => "j",
            InstructionKind::Jal => "jal",
            InstructionKind::Nop => "nop",
            InstructionKind::Syscall => "syscall",
        }
    }
}

impl From<&str> for InstructionKind {
    fn from(s: &str) -> InstructionKind {
        match s {
            "add" => InstructionKind::Add,
            "addi" => InstructionKind::Addi,
            "sub" => InstructionKind::Sub,
            "mul" => InstructionKind::Mul,
            "div" => InstructionKind::Div,
            "and" => InstructionKind::And,
            "andi" => InstructionKind::Andi,
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
            "lui" => InstructionKind::Lui,
            "move" => InstructionKind::Move,
            "la" => InstructionKind::La,
            "b" => InstructionKind::B,
            "j" => InstructionKind::J,
            "jal" => InstructionKind::Jal,
            "nop" => InstructionKind::Nop,
            "syscall" => InstructionKind::Syscall,
            _ => panic!("Invalid instruction: {}", s),
        }
    }
}

/// Represents an argument to a MIPS instruction.
#[derive(Clone, Debug, PartialEq)]
pub enum InstructionArg {
    /// A register argument.
    Register(Register),
    /// An immediate value argument.
    Immediate(Immediate),
    /// Register offset argument.
    RegisterOffset(Register, Immediate),
    /// A label argument.
    Label(String),
}

impl InstructionArg {
    pub fn show(&self) -> String {
        match self {
            InstructionArg::Register(r) => r.show().to_string(),
            InstructionArg::Immediate(i) => format!("0x{:x}", i),
            InstructionArg::RegisterOffset(r, offset) => format!("{}({})", offset, r.show()),
            InstructionArg::Label(l) => l.to_string(),
        }
    }

    pub fn show_color(&self) -> String {
        match self {
            InstructionArg::Register(r) => r.show().color(REGISTER_COLOR).to_string(),
            InstructionArg::Immediate(i) => format!("0x{:x}", i).color(IMMEDIATE_COLOR).to_string(),
            InstructionArg::RegisterOffset(r, offset) => format!(
                "{}({})",
                offset.to_string().color(IMMEDIATE_COLOR),
                r.show_color()
            )
            .color(REGISTER_COLOR)
            .to_string(),
            InstructionArg::Label(l) => l.to_string().color(LABEL_COLOR).to_string(),
        }
    }
}

/// Represents a MIPS instruction.
#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
    /// The kind of instruction.
    pub kind: InstructionKind,
    /// The arguments to the instruction.
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
        let mut result = self.kind.show().color(INSTRUCTION_COLOR).to_string();
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

    pub fn size() -> usize {
        4
    }
}

/// Represents a block of instructions in the text section.
#[derive(Debug, PartialEq)]
pub struct Block {
    /// The label of the block.
    pub label: String,
    /// The instructions in the block.
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn show(&self) -> String {
        let mut result = if self.label.is_empty() {
            String::new()
        } else {
            format!("{}:\n", self.label)
        };
        for instruction in &self.instructions {
            result.push_str(&format!("    {}\n", instruction.show()));
        }
        result
    }

    pub fn show_color(&self) -> String {
        let mut result = if self.label.is_empty() {
            String::new()
        } else {
            format!("{}:\n", self.label.clone().color(LABEL_COLOR))
        };
        for instruction in &self.instructions {
            result.push_str(&format!("    {}\n", instruction.show_color()));
        }
        result
    }
}

/// Represents the text section of a MIPS program.
#[derive(Debug, PartialEq)]
pub struct TextSection {
    /// The blocks of instructions.
    pub blocks: Vec<Block>,
    /// The global labels in the text section.
    pub global_labels: Vec<String>,
}

impl TextSection {
    pub fn empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn instructions(&self) -> Vec<&Instruction> {
        self.blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
            .collect()
    }

    pub fn instructions_move(self) -> Vec<Instruction> {
        self.blocks
            .into_iter()
            .flat_map(|block| block.instructions.into_iter())
            .collect()
    }

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
                ".global".color(DIRECTIVE_COLOR),
                label.clone().color(LABEL_COLOR)
            ));
        }
        for block in &self.blocks {
            result.push_str(&block.show_color());
        }
        result
    }
}

/// Represents a MIPS program.
#[derive(Debug, PartialEq)]
pub struct Program {
    /// The data section of the program.
    pub data_section: DataSection,
    /// The text section of the program.
    pub text_section: TextSection,
}

impl Program {
    pub fn show(&self) -> String {
        let mut result = String::new();
        // Data
        if !self.data_section.empty() {
            result.push_str(&format!("{}\n", Section::Data.show()));
            result.push_str(&self.data_section.show());
        }
        // Text
        if !self.text_section.empty() {
            result.push_str(&format!("\n{}\n", Section::Text.show()));
            result.push_str(&self.text_section.show());
        }
        result
    }

    pub fn show_color(&self) -> String {
        let mut result = String::new();
        // Data
        if !self.data_section.empty() {
            result.push_str(&format!("{}\n", Section::Data.show_color()));
            result.push_str(&self.data_section.show_color());
        }
        if !self.data_section.empty() && !self.text_section.empty() {
            result.push('\n'); // Add a newline between sections
        }
        // Text
        if !self.text_section.empty() {
            result.push_str(&format!("{}\n", Section::Text.show_color()));
            result.push_str(&self.text_section.show_color());
        }
        result
    }
}
