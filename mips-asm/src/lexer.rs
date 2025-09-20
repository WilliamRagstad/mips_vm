//! Modern lexer using logos for MIPS assembly

use logos::Logos;
use std::fmt;

/// MIPS assembly tokens using logos lexer
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Skip whitespace and comments
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"#[^\r\n]*", logos::skip)]
    #[regex(r"//[^\r\n]*", logos::skip)]
    Whitespace,

    // Newlines are significant for separating statements
    #[token("\n")]
    #[token("\r\n")]
    Newline,

    // R-type instructions
    #[token("add")]
    Add,
    #[token("addu")]
    Addu,
    #[token("sub")]
    Sub,
    #[token("subu")]
    Subu,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("xor")]
    Xor,
    #[token("nor")]
    Nor,
    #[token("slt")]
    Slt,
    #[token("sltu")]
    Sltu,
    #[token("sll")]
    Sll,
    #[token("srl")]
    Srl,
    #[token("sra")]
    Sra,
    #[token("sllv")]
    Sllv,
    #[token("srlv")]
    Srlv,
    #[token("srav")]
    Srav,
    #[token("jr")]
    Jr,
    #[token("jalr")]
    Jalr,
    #[token("syscall")]
    Syscall,
    #[token("mfhi")]
    Mfhi,
    #[token("mthi")]
    Mthi,
    #[token("mflo")]
    Mflo,
    #[token("mtlo")]
    Mtlo,
    #[token("mult")]
    Mult,
    #[token("multu")]
    Multu,
    #[token("div")]
    Div,
    #[token("divu")]
    Divu,

    // I-type instructions
    #[token("addi")]
    Addi,
    #[token("addiu")]
    Addiu,
    #[token("slti")]
    Slti,
    #[token("sltiu")]
    Sltiu,
    #[token("andi")]
    Andi,
    #[token("ori")]
    Ori,
    #[token("xori")]
    Xori,
    #[token("lui")]
    Lui,
    #[token("beq")]
    Beq,
    #[token("bne")]
    Bne,
    #[token("blez")]
    Blez,
    #[token("bgtz")]
    Bgtz,
    #[token("bltz")]
    Bltz,
    #[token("bgez")]
    Bgez,
    #[token("lb")]
    Lb,
    #[token("lh")]
    Lh,
    #[token("lw")]
    Lw,
    #[token("lbu")]
    Lbu,
    #[token("lhu")]
    Lhu,
    #[token("sb")]
    Sb,
    #[token("sh")]
    Sh,
    #[token("sw")]
    Sw,

    // J-type instructions
    #[token("j")]
    J,
    #[token("jal")]
    Jal,

    // Registers - MIPS has $0-$31 and named registers
    #[regex(r"\$([0-9]|[12][0-9]|3[01])", |lex| lex.slice()[1..].parse::<u8>().unwrap())]
    Register(u8),

    #[regex(r"\$zero|\$at|\$v[01]|\$a[0-3]|\$t[0-9]|\$s[0-7]|\$k[01]|\$gp|\$sp|\$fp|\$ra", |lex| {
        match lex.slice() {
            "$zero" => 0, "$at" => 1, "$v0" => 2, "$v1" => 3,
            "$a0" => 4, "$a1" => 5, "$a2" => 6, "$a3" => 7,
            "$t0" => 8, "$t1" => 9, "$t2" => 10, "$t3" => 11,
            "$t4" => 12, "$t5" => 13, "$t6" => 14, "$t7" => 15,
            "$s0" => 16, "$s1" => 17, "$s2" => 18, "$s3" => 19,
            "$s4" => 20, "$s5" => 21, "$s6" => 22, "$s7" => 23,
            "$t8" => 24, "$t9" => 25, "$k0" => 26, "$k1" => 27,
            "$gp" => 28, "$sp" => 29, "$fp" => 30, "$ra" => 31,
            _ => 0, // Should not happen with the regex
        }
    })]
    NamedRegister(u8),

    // Immediate values
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i32>().unwrap())]
    Immediate(i32),

    #[regex(r"0x[0-9a-fA-F]+", |lex| i32::from_str_radix(&lex.slice()[2..], 16).unwrap())]
    HexImmediate(i32),

    #[regex(r"0b[01]+", |lex| i32::from_str_radix(&lex.slice()[2..], 2).unwrap())]
    BinImmediate(i32),

    // Labels and identifiers
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*:", |lex| lex.slice()[..lex.slice().len()-1].to_string())]
    Label(String),

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Punctuation
    #[token(",")]
    Comma,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    // Assembler directives
    #[token(".text")]
    TextDirective,

    #[token(".data")]
    DataDirective,

    #[token(".word")]
    WordDirective,

    #[token(".byte")]
    ByteDirective,

    #[token(".ascii")]
    AsciiDirective,

    #[token(".asciiz")]
    AsciizDirective,

    #[token(".space")]
    SpaceDirective,

    #[token(".align")]
    AlignDirective,

    #[token(".global")]
    GlobalDirective,

    #[token(".globl")]
    GloblDirective,

    // String literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let slice = lex.slice();
        slice[1..slice.len()-1].to_string()
    })]
    StringLiteral(String),

    // Error token
    #[error]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Register(n) => write!(f, "${}", n),
            Token::NamedRegister(n) => write!(f, "${}", n),
            Token::Immediate(n) => write!(f, "{}", n),
            Token::HexImmediate(n) => write!(f, "0x{:x}", n),
            Token::BinImmediate(n) => write!(f, "0b{:b}", n),
            Token::Label(s) => write!(f, "{}:", s),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Token {
    /// Check if token is a register (numbered or named)
    pub fn register_number(&self) -> Option<u8> {
        match self {
            Token::Register(n) | Token::NamedRegister(n) => Some(*n),
            _ => None,
        }
    }

    /// Check if token is an immediate value
    pub fn immediate_value(&self) -> Option<i32> {
        match self {
            Token::Immediate(n) | Token::HexImmediate(n) | Token::BinImmediate(n) => Some(*n),
            _ => None,
        }
    }

    /// Check if token is an instruction mnemonic
    pub fn is_instruction(&self) -> bool {
        matches!(
            self,
            Token::Add | Token::Addu | Token::Sub | Token::Subu |
            Token::And | Token::Or | Token::Xor | Token::Nor |
            Token::Slt | Token::Sltu | Token::Sll | Token::Srl | Token::Sra |
            Token::Sllv | Token::Srlv | Token::Srav | Token::Jr | Token::Jalr |
            Token::Syscall | Token::Mfhi | Token::Mthi | Token::Mflo | Token::Mtlo |
            Token::Mult | Token::Multu | Token::Div | Token::Divu |
            Token::Addi | Token::Addiu | Token::Slti | Token::Sltiu |
            Token::Andi | Token::Ori | Token::Xori | Token::Lui |
            Token::Beq | Token::Bne | Token::Blez | Token::Bgtz | Token::Bltz | Token::Bgez |
            Token::Lb | Token::Lh | Token::Lw | Token::Lbu | Token::Lhu |
            Token::Sb | Token::Sh | Token::Sw | Token::J | Token::Jal
        )
    }

    /// Check if token is a directive
    pub fn is_directive(&self) -> bool {
        matches!(
            self,
            Token::TextDirective | Token::DataDirective | Token::WordDirective |
            Token::ByteDirective | Token::AsciiDirective | Token::AsciizDirective |
            Token::SpaceDirective | Token::AlignDirective | Token::GlobalDirective |
            Token::GloblDirective
        )
    }
}

/// Lexer for MIPS assembly source
pub struct Lexer<'input> {
    inner: logos::Lexer<'input, Token>,
}

impl<'input> Lexer<'input> {
    /// Create a new lexer for the given input
    pub fn new(input: &'input str) -> Self {
        Self {
            inner: Token::lexer(input),
        }
    }

    /// Get the current token
    pub fn token(&self) -> Token {
        self.inner.token()
    }

    /// Get the current token's text slice
    pub fn slice(&self) -> &'input str {
        self.inner.slice()
    }

    /// Get the current position in the input
    pub fn span(&self) -> std::ops::Range<usize> {
        self.inner.span()
    }

    /// Advance to the next token
    pub fn next(&mut self) -> Option<Token> {
        self.inner.next()
    }

    /// Collect all tokens (useful for debugging)
    pub fn collect_tokens(mut self) -> Vec<(Token, &'input str)> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next() {
            let slice = self.slice();
            tokens.push((token, slice));
        }
        tokens
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
