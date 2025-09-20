//! Modern parser using nom combinators with logos tokens

use crate::lexer::{Token, Lexer};
use mips_isa::{InstructionFormat, opcodes};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::{map, opt, recognize},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Lexer error at position {pos}: {message}")]
    LexError { pos: usize, message: String },
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),
    #[error("Invalid register: {0}")]
    InvalidRegister(String),
    #[error("Invalid immediate value: {0}")]
    InvalidImmediate(String),
}

/// Parsed MIPS assembly program
#[derive(Debug, Clone)]
pub struct ParsedProgram {
    pub statements: Vec<Statement>,
    pub labels: HashMap<String, usize>, // label -> instruction index
}

impl ParsedProgram {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            labels: HashMap::new(),
        }
    }
}

impl Default for ParsedProgram {
    fn default() -> Self {
        Self::new()
    }
}

/// Assembly statement
#[derive(Debug, Clone)]
pub enum Statement {
    Instruction(ParsedInstruction),
    Label(String),
    Directive(Directive),
}

/// Parsed instruction
#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub mnemonic: String,
    pub format: InstructionFormat,
    pub operands: Vec<Operand>,
    pub line: usize,
    pub column: usize,
}

/// Instruction operand
#[derive(Debug, Clone)]
pub enum Operand {
    Register(u8),
    Immediate(i32),
    Label(String),
    Memory { base: u8, offset: i32 }, // offset(base)
}

/// Assembler directive
#[derive(Debug, Clone)]
pub enum Directive {
    Text,
    Data,
    Word(Vec<i32>),
    Byte(Vec<u8>),
    Ascii(String),
    Asciiz(String),
    Space(usize),
    Align(usize),
    Global(String),
}

/// Token stream for nom parsing
pub struct TokenStream<'input> {
    tokens: Vec<(Token, &'input str, usize)>, // token, slice, position
    position: usize,
}

impl<'input> TokenStream<'input> {
    /// Create a new token stream from input
    pub fn new(input: &'input str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        
        while let Some(token) = lexer.next() {
            if token == Token::Error {
                return Err(ParseError::LexError {
                    pos: lexer.span().start,
                    message: format!("Unexpected character: '{}'", lexer.slice()),
                });
            }
            
            // Skip whitespace tokens for parsing
            if !matches!(token, Token::Whitespace) {
                tokens.push((token, lexer.slice(), lexer.span().start));
            }
        }
        
        Ok(Self { tokens, position: 0 })
    }

    /// Get current token
    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position).map(|(token, _, _)| token)
    }

    /// Get current token slice
    pub fn current_slice(&self) -> Option<&'input str> {
        self.tokens.get(self.position).map(|(_, slice, _)| *slice)
    }

    /// Get current position
    pub fn current_position(&self) -> usize {
        self.tokens.get(self.position).map(|(_, _, pos)| *pos).unwrap_or(0)
    }

    /// Advance to next token
    pub fn advance(&mut self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.current()
    }

    /// Check if we're at end of stream
    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Peek at token at relative offset
    pub fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset).map(|(token, _, _)| token)
    }
}

/// Parse a complete MIPS assembly program
pub fn parse_program(input: &str) -> Result<ParsedProgram, ParseError> {
    let mut stream = TokenStream::new(input)?;
    let mut program = ParsedProgram::new();
    let mut instruction_index = 0;

    while !stream.is_at_end() {
        // Skip newlines
        while matches!(stream.current(), Some(Token::Newline)) {
            stream.advance();
        }
        
        if stream.is_at_end() {
            break;
        }

        match stream.current() {
            Some(Token::Label(label)) => {
                let label = label.clone();
                program.labels.insert(label.clone(), instruction_index);
                program.statements.push(Statement::Label(label));
                stream.advance();
            }
            Some(token) if token.is_directive() => {
                let directive = parse_directive(&mut stream)?;
                program.statements.push(Statement::Directive(directive));
            }
            Some(token) if token.is_instruction() => {
                let instruction = parse_instruction(&mut stream)?;
                program.statements.push(Statement::Instruction(instruction));
                instruction_index += 1;
            }
            Some(token) => {
                return Err(ParseError::ParseError(format!(
                    "Unexpected token: {:?}",
                    token
                )));
            }
            None => break,
        }

        // Consume trailing newlines
        while matches!(stream.current(), Some(Token::Newline)) {
            stream.advance();
        }
    }

    Ok(program)
}

/// Parse a single instruction
fn parse_instruction(stream: &mut TokenStream) -> Result<ParsedInstruction, ParseError> {
    let mnemonic = match stream.current() {
        Some(token) if token.is_instruction() => {
            let mnemonic = format!("{:?}", token).to_lowercase();
            stream.advance();
            mnemonic
        }
        Some(token) => {
            return Err(ParseError::InvalidInstruction(format!("{:?}", token)));
        }
        None => {
            return Err(ParseError::ParseError("Expected instruction".to_string()));
        }
    };

    // Determine instruction format
    let format = determine_instruction_format(&mnemonic)?;

    // Parse operands based on format
    let operands = match format {
        InstructionFormat::R => parse_r_type_operands(stream, &mnemonic)?,
        InstructionFormat::I => parse_i_type_operands(stream, &mnemonic)?,
        InstructionFormat::J => parse_j_type_operands(stream)?,
    };

    Ok(ParsedInstruction {
        mnemonic,
        format,
        operands,
        line: 0,    // Would need to track this from lexer
        column: 0,  // Would need to track this from lexer
    })
}

/// Parse R-type instruction operands
fn parse_r_type_operands(stream: &mut TokenStream, mnemonic: &str) -> Result<Vec<Operand>, ParseError> {
    match mnemonic {
        "jr" | "jalr" => {
            // Single register operand
            vec![parse_register(stream)?]
        }
        "syscall" | "mfhi" | "mthi" | "mflo" | "mtlo" => {
            // No operands
            Vec::new()
        }
        "sll" | "srl" | "sra" => {
            // rd, rt, shamt
            let rd = parse_register(stream)?;
            expect_comma(stream)?;
            let rt = parse_register(stream)?;
            expect_comma(stream)?;
            let shamt = parse_immediate(stream)?;
            vec![rd, rt, shamt]
        }
        _ => {
            // Standard 3-register format: rd, rs, rt
            let rd = parse_register(stream)?;
            expect_comma(stream)?;
            let rs = parse_register(stream)?;
            expect_comma(stream)?;
            let rt = parse_register(stream)?;
            vec![rd, rs, rt]
        }
    }.into()
}

/// Parse I-type instruction operands  
fn parse_i_type_operands(stream: &mut TokenStream, mnemonic: &str) -> Result<Vec<Operand>, ParseError> {
    match mnemonic {
        "lui" => {
            // rt, immediate
            let rt = parse_register(stream)?;
            expect_comma(stream)?;
            let imm = parse_immediate(stream)?;
            vec![rt, imm]
        }
        "beq" | "bne" => {
            // rs, rt, label/offset
            let rs = parse_register(stream)?;
            expect_comma(stream)?;
            let rt = parse_register(stream)?;
            expect_comma(stream)?;
            let target = parse_branch_target(stream)?;
            vec![rs, rt, target]
        }
        "blez" | "bgtz" | "bltz" | "bgez" => {
            // rs, label/offset
            let rs = parse_register(stream)?;
            expect_comma(stream)?;
            let target = parse_branch_target(stream)?;
            vec![rs, target]
        }
        "lb" | "lh" | "lw" | "lbu" | "lhu" | "sb" | "sh" | "sw" => {
            // rt, offset(base)
            let rt = parse_register(stream)?;
            expect_comma(stream)?;
            let mem = parse_memory_operand(stream)?;
            vec![rt, mem]
        }
        _ => {
            // Standard immediate format: rt, rs, immediate
            let rt = parse_register(stream)?;
            expect_comma(stream)?;
            let rs = parse_register(stream)?;
            expect_comma(stream)?;
            let imm = parse_immediate(stream)?;
            vec![rt, rs, imm]
        }
    }.into()
}

/// Parse J-type instruction operands
fn parse_j_type_operands(stream: &mut TokenStream) -> Result<Vec<Operand>, ParseError> {
    // Just a label or address
    let target = parse_jump_target(stream)?;
    Ok(vec![target])
}

/// Parse a register operand
fn parse_register(stream: &mut TokenStream) -> Result<Operand, ParseError> {
    match stream.current() {
        Some(token) => {
            if let Some(reg_num) = token.register_number() {
                stream.advance();
                Ok(Operand::Register(reg_num))
            } else {
                Err(ParseError::InvalidRegister(format!("{:?}", token)))
            }
        }
        None => Err(ParseError::ParseError("Expected register".to_string())),
    }
}

/// Parse an immediate operand
fn parse_immediate(stream: &mut TokenStream) -> Result<Operand, ParseError> {
    match stream.current() {
        Some(token) => {
            if let Some(imm_val) = token.immediate_value() {
                stream.advance();
                Ok(Operand::Immediate(imm_val))
            } else {
                Err(ParseError::InvalidImmediate(format!("{:?}", token)))
            }
        }
        None => Err(ParseError::ParseError("Expected immediate value".to_string())),
    }
}

/// Parse a branch target (label or immediate)
fn parse_branch_target(stream: &mut TokenStream) -> Result<Operand, ParseError> {
    match stream.current() {
        Some(Token::Identifier(label)) => {
            let label = label.clone();
            stream.advance();
            Ok(Operand::Label(label))
        }
        Some(token) if token.immediate_value().is_some() => parse_immediate(stream),
        Some(token) => Err(ParseError::ParseError(format!(
            "Expected label or immediate, got {:?}",
            token
        ))),
        None => Err(ParseError::ParseError("Expected branch target".to_string())),
    }
}

/// Parse a jump target
fn parse_jump_target(stream: &mut TokenStream) -> Result<Operand, ParseError> {
    parse_branch_target(stream) // Same as branch target for now
}

/// Parse memory operand like offset(base)
fn parse_memory_operand(stream: &mut TokenStream) -> Result<Operand, ParseError> {
    // Parse immediate first (offset)
    let offset = match stream.current() {
        Some(token) if token.immediate_value().is_some() => {
            let val = token.immediate_value().unwrap();
            stream.advance();
            val
        }
        _ => 0, // No offset
    };

    // Expect opening parenthesis
    match stream.current() {
        Some(Token::LeftParen) => {
            stream.advance();
        }
        _ => return Err(ParseError::ParseError("Expected '(' in memory operand".to_string())),
    }

    // Parse base register
    let base = match parse_register(stream)? {
        Operand::Register(reg) => reg,
        _ => return Err(ParseError::ParseError("Expected register in memory operand".to_string())),
    };

    // Expect closing parenthesis
    match stream.current() {
        Some(Token::RightParen) => {
            stream.advance();
        }
        _ => return Err(ParseError::ParseError("Expected ')' in memory operand".to_string())),
    }

    Ok(Operand::Memory { base, offset })
}

/// Parse a directive
fn parse_directive(stream: &mut TokenStream) -> Result<Directive, ParseError> {
    let directive = match stream.current() {
        Some(Token::TextDirective) => {
            stream.advance();
            Directive::Text
        }
        Some(Token::DataDirective) => {
            stream.advance();
            Directive::Data
        }
        Some(Token::WordDirective) => {
            stream.advance();
            let values = parse_immediate_list(stream)?;
            Directive::Word(values)
        }
        Some(Token::GlobalDirective) | Some(Token::GloblDirective) => {
            stream.advance();
            let name = match stream.current() {
                Some(Token::Identifier(name)) => {
                    let name = name.clone();
                    stream.advance();
                    name
                }
                _ => return Err(ParseError::ParseError("Expected identifier after .global".to_string())),
            };
            Directive::Global(name)
        }
        Some(token) => {
            return Err(ParseError::ParseError(format!(
                "Unsupported directive: {:?}",
                token
            )));
        }
        None => {
            return Err(ParseError::ParseError("Expected directive".to_string()));
        }
    };

    Ok(directive)
}

/// Parse a list of immediate values
fn parse_immediate_list(stream: &mut TokenStream) -> Result<Vec<i32>, ParseError> {
    let mut values = Vec::new();
    
    loop {
        match stream.current() {
            Some(token) if token.immediate_value().is_some() => {
                values.push(token.immediate_value().unwrap());
                stream.advance();
                
                // Check for comma
                if matches!(stream.current(), Some(Token::Comma)) {
                    stream.advance();
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    
    Ok(values)
}

/// Expect a comma token
fn expect_comma(stream: &mut TokenStream) -> Result<(), ParseError> {
    match stream.current() {
        Some(Token::Comma) => {
            stream.advance();
            Ok(())
        }
        Some(token) => Err(ParseError::ParseError(format!(
            "Expected comma, got {:?}",
            token
        ))),
        None => Err(ParseError::ParseError("Expected comma".to_string())),
    }
}

/// Determine instruction format from mnemonic
fn determine_instruction_format(mnemonic: &str) -> Result<InstructionFormat, ParseError> {
    match mnemonic {
        // R-type instructions
        "add" | "addu" | "sub" | "subu" | "and" | "or" | "xor" | "nor" |
        "slt" | "sltu" | "sll" | "srl" | "sra" | "sllv" | "srlv" | "srav" |
        "jr" | "jalr" | "syscall" | "mfhi" | "mthi" | "mflo" | "mtlo" |
        "mult" | "multu" | "div" | "divu" => Ok(InstructionFormat::R),

        // I-type instructions  
        "addi" | "addiu" | "slti" | "sltiu" | "andi" | "ori" | "xori" | "lui" |
        "beq" | "bne" | "blez" | "bgtz" | "bltz" | "bgez" |
        "lb" | "lh" | "lw" | "lbu" | "lhu" | "sb" | "sh" | "sw" => Ok(InstructionFormat::I),

        // J-type instructions
        "j" | "jal" => Ok(InstructionFormat::J),

        _ => Err(ParseError::InvalidInstruction(format!(
            "Unknown instruction: {}",
            mnemonic
        ))),
    }
}
