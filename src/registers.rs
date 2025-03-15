use std::collections::HashMap;

use crate::program::{Register, Word};

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
