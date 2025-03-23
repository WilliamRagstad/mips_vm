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
