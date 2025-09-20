//! MIPS register definitions - Moved from mips-vm

/// Represents a MIPS register.
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum Register {
    /// Zero register - Always contains the value 0.
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
    /// Saved register
    S0 = 16,
    /// Saved register
    S1 = 17,
    /// Saved register
    S2 = 18,
    /// Saved register
    S3 = 19,
    /// Saved register
    S4 = 20,
    /// Saved register
    S5 = 21,
    /// Saved register
    S6 = 22,
    /// Saved register
    S7 = 23,
    /// Temporary value
    T8 = 24,
    /// Temporary value
    T9 = 25,
    /// Kernel
    K0 = 26,
    /// Kernel
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
    /// Get register name as string
    pub fn name(&self) -> &'static str {
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

    /// Get register number
    pub fn number(&self) -> u8 {
        *self as u8
    }

    /// Create register from number
    pub fn from_number(num: u8) -> Option<Register> {
        match num {
            0 => Some(Register::Zero),
            1 => Some(Register::At),
            2 => Some(Register::V0),
            3 => Some(Register::V1),
            4 => Some(Register::A0),
            5 => Some(Register::A1),
            6 => Some(Register::A2),
            7 => Some(Register::A3),
            8 => Some(Register::T0),
            9 => Some(Register::T1),
            10 => Some(Register::T2),
            11 => Some(Register::T3),
            12 => Some(Register::T4),
            13 => Some(Register::T5),
            14 => Some(Register::T6),
            15 => Some(Register::T7),
            16 => Some(Register::S0),
            17 => Some(Register::S1),
            18 => Some(Register::S2),
            19 => Some(Register::S3),
            20 => Some(Register::S4),
            21 => Some(Register::S5),
            22 => Some(Register::S6),
            23 => Some(Register::S7),
            24 => Some(Register::T8),
            25 => Some(Register::T9),
            26 => Some(Register::K0),
            27 => Some(Register::K1),
            28 => Some(Register::Gp),
            29 => Some(Register::Sp),
            30 => Some(Register::Fp),
            31 => Some(Register::Ra),
            _ => None,
        }
    }

    /// Parse register from string name
    pub fn from_name(name: &str) -> Option<Register> {
        let name = name.to_lowercase();
        let name = name.strip_prefix('$').unwrap_or(&name);
        
        match name {
            "zero" | "0" => Some(Register::Zero),
            "at" | "1" => Some(Register::At),
            "v0" | "2" => Some(Register::V0),
            "v1" | "3" => Some(Register::V1),
            "a0" | "4" => Some(Register::A0),
            "a1" | "5" => Some(Register::A1),
            "a2" | "6" => Some(Register::A2),
            "a3" | "7" => Some(Register::A3),
            "t0" | "8" => Some(Register::T0),
            "t1" | "9" => Some(Register::T1),
            "t2" | "10" => Some(Register::T2),
            "t3" | "11" => Some(Register::T3),
            "t4" | "12" => Some(Register::T4),
            "t5" | "13" => Some(Register::T5),
            "t6" | "14" => Some(Register::T6),
            "t7" | "15" => Some(Register::T7),
            "s0" | "16" => Some(Register::S0),
            "s1" | "17" => Some(Register::S1),
            "s2" | "18" => Some(Register::S2),
            "s3" | "19" => Some(Register::S3),
            "s4" | "20" => Some(Register::S4),
            "s5" | "21" => Some(Register::S5),
            "s6" | "22" => Some(Register::S6),
            "s7" | "23" => Some(Register::S7),
            "t8" | "24" => Some(Register::T8),
            "t9" | "25" => Some(Register::T9),
            "k0" | "26" => Some(Register::K0),
            "k1" | "27" => Some(Register::K1),
            "gp" | "28" => Some(Register::Gp),
            "sp" | "29" => Some(Register::Sp),
            "fp" | "30" => Some(Register::Fp),
            "ra" | "31" => Some(Register::Ra),
            _ => {
                if let Ok(num) = name.parse::<u8>() {
                    Register::from_number(num)
                } else {
                    None
                }
            }
        }
    }
}
