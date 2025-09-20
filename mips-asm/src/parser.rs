//! Assembly source parser - Enhanced from existing mips-vm parser

use pest::Parser;
use pest_derive::Parser;
use mips_isa::{Instruction as IsaInstruction, InstructionFormat};

#[derive(Parser)]
#[grammar = "mips.pest"]
pub struct MipsParser;

/// Parse MIPS assembly source
pub fn parse_assembly(source: &str) -> Result<ParsedProgram, ParseError> {
    let pairs = MipsParser::parse(Rule::program, source)
        .map_err(|e| ParseError::SyntaxError(e.to_string()))?;
    
    let mut program = ParsedProgram::new();
    
    for pair in pairs {
        match pair.as_rule() {
            Rule::statement => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::instruction => {
                            let instruction = parse_instruction(inner_pair)?;
                            program.instructions.push(instruction);
                        }
                        Rule::label => {
                            let label = parse_label(inner_pair)?;
                            program.labels.push(label);
                        }
                        Rule::directive => {
                            let directive = parse_directive(inner_pair)?;
                            program.directives.push(directive);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    Ok(program)
}

fn parse_instruction(pair: pest::iterators::Pair<Rule>) -> Result<ParsedInstruction, ParseError> {
    let mut mnemonic = String::new();
    let mut operands = Vec::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::opcode | Rule::pseudo => {
                mnemonic = inner_pair.as_str().to_string();
            }
            Rule::args => {
                for arg_pair in inner_pair.into_inner() {
                    operands.push(parse_operand(arg_pair)?);
                }
            }
            _ => {}
        }
    }
    
    Ok(ParsedInstruction {
        mnemonic,
        operands,
        line: 0, // TODO: Get actual line number
    })
}

fn parse_label(pair: pest::iterators::Pair<Rule>) -> Result<Label, ParseError> {
    let mut name = String::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                name = inner_pair.as_str().to_string();
            }
            _ => {}
        }
    }
    
    Ok(Label {
        name,
        address: None,
        line: 0, // TODO: Get actual line number
    })
}

fn parse_directive(pair: pest::iterators::Pair<Rule>) -> Result<Directive, ParseError> {
    let mut name = String::new();
    let mut args = Vec::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::section_directive | Rule::data_directive | Rule::text_directive => {
                name = inner_pair.as_str().to_string();
            }
            Rule::args => {
                for arg_pair in inner_pair.into_inner() {
                    args.push(parse_operand(arg_pair)?);
                }
            }
            _ => {}
        }
    }
    
    Ok(Directive { name, args })
}

fn parse_operand(pair: pest::iterators::Pair<Rule>) -> Result<Operand, ParseError> {
    match pair.as_rule() {
        Rule::register => {
            let reg_str = pair.as_str();
            Ok(Operand::Register(reg_str.to_string()))
        }
        Rule::immediate => {
            let imm_str = pair.as_str();
            let value = if imm_str.starts_with("0x") {
                i32::from_str_radix(&imm_str[2..], 16)
            } else {
                imm_str.parse::<i32>()
            };
            Ok(Operand::Immediate(value.map_err(|_| ParseError::InvalidImmediate(imm_str.to_string()))?))
        }
        Rule::memory_reference => {
            let mut offset = 0;
            let mut register = String::new();
            
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::immediate => {
                        offset = inner_pair.as_str().parse().unwrap_or(0);
                    }
                    Rule::register => {
                        register = inner_pair.as_str().to_string();
                    }
                    _ => {}
                }
            }
            
            Ok(Operand::MemoryReference { offset, register })
        }
        Rule::identifier => {
            Ok(Operand::Label(pair.as_str().to_string()))
        }
        _ => Err(ParseError::UnexpectedToken(pair.as_str().to_string())),
    }
}

#[derive(Debug, Clone)]
pub struct ParsedProgram {
    pub instructions: Vec<ParsedInstruction>,
    pub labels: Vec<Label>,
    pub directives: Vec<Directive>,
}

impl ParsedProgram {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: Vec::new(),
            directives: Vec::new(),
        }
    }
}

impl Default for ParsedProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub mnemonic: String,
    pub operands: Vec<Operand>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: String,
    pub address: Option<u32>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub name: String,
    pub args: Vec<Operand>,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(String),
    Immediate(i32),
    MemoryReference { offset: i32, register: String },
    Label(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Syntax error: {0}")]
    SyntaxError(String),
    #[error("Invalid immediate value: {0}")]
    InvalidImmediate(String),
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
}
