use std::collections::HashMap;

use colorful::Colorful;

use crate::program::{Word, REGISTER_COLOR};

#[derive(Debug, Default)]
pub struct Registers {
    values: HashMap<Register, Word>,
}

impl Registers {
    pub fn get(&self, register: &Register) -> Word {
        match register {
            Register::Zero => 0,
            _ => *self.values.get(register).unwrap_or(&0),
        }
    }

    pub fn set(&mut self, register: &Register, value: Word) {
        self.values.insert(*register, value);
    }
}

/// Represents a MIPS register.
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum Register {
    /// Zero register.
    /// Always contains the value 0.
    Zero = 0,
    /// Assembler temp
    At = 1,
    /// Function return
    V0 = 2,
    /// Function return
    V1 = 3,
    /// Argument
    A0 = 4,
    /// Argument
    A1 = 5,
    /// Argument
    A2 = 6,
    /// Argument
    A3 = 7,
    /// Temporary value
    T0 = 8,
    /// Temporary value
    T1 = 9,
    /// Temporary value
    T2 = 10,
    /// Temporary value
    T3 = 11,
    /// Temporary value
    T4 = 12,
    /// Temporary value
    T5 = 13,
    /// Temporary value
    T6 = 14,
    /// Temporary value
    T7 = 15,
    /// Saved temporary
    S0 = 16,
    /// Saved temporary
    S1 = 17,
    /// Saved temporary
    S2 = 18,
    /// Saved temporary
    S3 = 19,
    /// Saved temporary
    S4 = 20,
    /// Saved temporary
    S5 = 21,
    /// Saved temporary
    S6 = 22,
    /// Saved temporary
    S7 = 23,
    /// Temporary value
    T8 = 24,
    /// Temporary value
    T9 = 25,
    /// Reserved for OS
    K0 = 26,
    /// Reserved for OS
    K1 = 27,
    /// Global pointer
    Gp = 28,
    /// Stack pointer
    Sp = 29,
    /// Frame pointer
    Fp = 30,
    /// Return address
    Ra = 31,
}

impl Register {
    pub fn encode(&self) -> u8 {
        *self as u8
    }

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

    pub fn show_color(&self) -> String {
        self.show().color(REGISTER_COLOR).to_string()
    }
}

impl From<&str> for Register {
    fn from(s: &str) -> Register {
        match s {
            "$zero" => Register::Zero,
            "$0" => Register::Zero,
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
