use std::collections::HashMap;

use colorful::Colorful;

use crate::program::{
    Address, DataSection, InstructionArg, InstructionKind, Program, Register, Word,
};

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

    pub fn set(&mut self, register: Register, value: Word) {
        self.values.insert(register, value);
    }
}

pub fn execute(program: Program) {
    log::debug!(
        "{}\n{}",
        "======= EXECUTE =======".blue(),
        program.show_color()
    );
    log::debug!("{}", "======= OUTPUT =======".blue());

    let mut registers = Registers::default();
    'execution: for block in program.text.blocks {
        log::trace!("Executing block {}...", block.show_color());
        for instruction in block.instructions {
            log::trace!("Executing instruction {}", instruction.show_color());
            let mut arith = |operation: fn(Word, Word) -> Word| {
                arithmetic(&mut registers, &program.data, &instruction.args, operation)
            };
            match instruction.kind {
                InstructionKind::Li => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        let value = load_word(&instruction.args[1], &registers, &program.data);
                        registers.set(*r, value);
                    }
                    _ => panic!("Invalid argument for LI instruction"),
                },
                InstructionKind::La => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        let addr = load_address(&instruction.args[1], &registers, &program.data);
                        registers.set(*r, addr as Word);
                    }
                    _ => panic!("Invalid argument for LA instruction"),
                },
                InstructionKind::Move => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        registers.set(
                            *r,
                            load_word(&instruction.args[1], &registers, &program.data),
                        );
                    }
                    _ => panic!("Invalid argument for MOV instruction"),
                },
                InstructionKind::Add => arith(|a, b| a + b),
                InstructionKind::Sub => arith(|a, b| a - b),
                InstructionKind::Mul => arith(|a, b| a * b),
                InstructionKind::Div => arith(|a, b| a / b),
                InstructionKind::And => arith(|a, b| a & b),
                InstructionKind::Or => arith(|a, b| a | b),
                InstructionKind::Xor => arith(|a, b| a ^ b),
                InstructionKind::Nor => arith(|a, b| !(a | b)),
                InstructionKind::Slt => arith(|a, b| if a < b { 1 } else { 0 }),
                InstructionKind::Sll => arith(|a, b| a << b),
                InstructionKind::Srl => arith(|a, b| a >> b),
                InstructionKind::Sra => arith(|a, b| a >> b),
                InstructionKind::Jr => {
                    let address = load_word(&instruction.args[0], &registers, &program.data);
                    log::trace!("Jumping to address {}", address);
                }
                InstructionKind::Syscall => {
                    if !syscall(&mut registers, &program.data) {
                        break 'execution;
                    }
                }
                _ => unimplemented!("Instruction not implemented: {:?}", instruction),
            }
        }
    }
    log::trace!("Program executed.");
}

fn load_word(arg: &InstructionArg, registers: &Registers, data: &DataSection) -> Word {
    match arg {
        InstructionArg::Immediate(value) => *value as Word,
        InstructionArg::Register(r) => registers.get(r),
        InstructionArg::Label(l) => {
            let data = data.globals.get(l).unwrap();
            Word::from_le_bytes(data.data[0..4].try_into().unwrap())
        }
    }
}

fn load_address(arg: &InstructionArg, registers: &Registers, data: &DataSection) -> Address {
    match arg {
        InstructionArg::Immediate(value) => *value as Address,
        InstructionArg::Register(r) => registers.get(r),
        InstructionArg::Label(l) => data.globals.get(l).unwrap().address(),
    }
}

fn arithmetic<F>(
    registers: &mut Registers,
    data: &DataSection,
    args: &[InstructionArg],
    operation: F,
) where
    F: Fn(Word, Word) -> Word,
{
    match &args[0] {
        InstructionArg::Register(r) => {
            let dest = load_word(&args[0], registers, data);
            let src = load_word(&args[1], registers, data);
            registers.set(*r, operation(dest, src));
        }
        _ => panic!("Invalid argument for instruction"),
    }
}

#[derive(Debug, PartialEq)]
enum Syscall {
    PrintInt = 1,
    PrintFloat = 2,
    PrintDouble = 3,
    PrintChar = 11,
    PrintString = 4,
    ReadInt = 5,
    ReadFloat = 6,
    ReadDouble = 7,
    ReadChar = 12,
    ReadString = 8,
    Sbrk = 9,
    Exit = 10,
    Exit2 = 17,
}

impl From<Word> for Syscall {
    fn from(value: Word) -> Self {
        match value {
            _ if value == Syscall::PrintInt as Word => Syscall::PrintInt,
            _ if value == Syscall::PrintFloat as Word => Syscall::PrintFloat,
            _ if value == Syscall::PrintDouble as Word => Syscall::PrintDouble,
            _ if value == Syscall::PrintChar as Word => Syscall::PrintChar,
            _ if value == Syscall::PrintString as Word => Syscall::PrintString,
            _ if value == Syscall::ReadInt as Word => Syscall::ReadInt,
            _ if value == Syscall::ReadFloat as Word => Syscall::ReadFloat,
            _ if value == Syscall::ReadDouble as Word => Syscall::ReadDouble,
            _ if value == Syscall::ReadChar as Word => Syscall::ReadChar,
            _ if value == Syscall::ReadString as Word => Syscall::ReadString,
            _ if value == Syscall::Sbrk as Word => Syscall::Sbrk,
            _ if value == Syscall::Exit as Word => Syscall::Exit,
            _ if value == Syscall::Exit2 as Word => Syscall::Exit2,
            _ => panic!("Invalid syscall number: {}", value),
        }
    }
}

fn syscall(registers: &mut Registers, data: &DataSection) -> bool {
    let v0: Syscall = registers.get(&Register::V0).into();
    match v0 {
        Syscall::PrintInt => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, data);
            println!("{}", a0);
        }
        Syscall::PrintFloat => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, data);
            println!("{}", f32::from_bits(a0 as u32));
        }
        Syscall::PrintDouble => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, data);
            println!("{}", f64::from_bits(a0 as u64));
        }
        Syscall::PrintChar => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, data);
            print!("{}", a0 as u8 as char);
        }
        Syscall::PrintString => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, data);
            let raw = &data.find_by_address(a0 as Address).unwrap().data;
            print!("{}", std::str::from_utf8(raw).unwrap());
        }
        Syscall::ReadInt => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().parse::<Word>().unwrap();
            registers.set(Register::V0, value);
        }
        Syscall::ReadFloat => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().parse::<f32>().unwrap();
            registers.set(Register::V0, value.to_bits() as Word);
        }
        Syscall::ReadDouble => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().parse::<f64>().unwrap();
            registers.set(Register::V0, value.to_bits() as Word);
        }
        Syscall::ReadChar => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().chars().next().unwrap() as Word;
            registers.set(Register::V0, value);
        }
        Syscall::ReadString => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, data);
            let a1 = load_word(&InstructionArg::Register(Register::A1), registers, data);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let data = data.find_by_address(a0 as Address).unwrap();
            let mut data = data.data.clone();
            data.extend(input.as_bytes());
            data.push(0);
            data.truncate(a1 as usize);
            data.resize(a1 as usize, 0);
            data.shrink_to_fit();
            data.truncate(a1 as usize);
            data.resize(a1 as usize, 0);
            data.shrink_to_fit();
        }
        Syscall::Sbrk => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, data);
            let data = data.find_by_address(a0 as Address).unwrap();
            let address = data.address();
            registers.set(Register::V0, address as Word);
        }
        Syscall::Exit | Syscall::Exit2 => {
            log::trace!("Exiting program...");
            // std::process::exit(0);
            return false;
        }
    };
    true
}

#[cfg(test)]
mod test {
    use crate::{execute, parse};

    #[test]
    fn test() {
        let input = include_str!("../tests/prog1.asm");
        let prog = parse(input);
        assert_ne!(prog, None);
        let program = prog.unwrap();
        log::trace!("====== Parsed Program ======\n{}", program.show_color());
        log::trace!("====== Executing Program ======");
        let entry = program
            .text
            .entry_block()
            .expect("No entry block found")
            .address;
        execute(program, entry);
        log::trace!("====== Done ======");
    }
}
