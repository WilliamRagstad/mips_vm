use std::collections::HashMap;

use crate::program::{DataSection, InstructionArg, InstructionKind, Offset, Program, Register};

pub fn execute(program: Program) {
    let mut registers: HashMap<Register, i32> = HashMap::new();
    'execution: for block in program.text.blocks {
        log::trace!("Executing block {}...", block.label);
        for instruction in block.instructions {
            log::trace!("Executing instruction {:?}", instruction);
            let mut exec_instr = |operation: fn(i32, i32) -> i32| {
                arithmetic(&mut registers, &program.data, &instruction.args, operation)
            };
            match instruction.kind {
                InstructionKind::Li => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        let value = load_word(&instruction.args[1], &registers, &program.data);
                        registers.insert(*r, value);
                    }
                    _ => panic!("Invalid argument for LI instruction"),
                },
                InstructionKind::La => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        let addr = load_address(&instruction.args[1], &registers, &program.data);
                        registers.insert(*r, addr as i32);
                    }
                    _ => panic!("Invalid argument for LA instruction"),
                },
                InstructionKind::Move => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        registers.insert(
                            *r,
                            load_word(&instruction.args[1], &registers, &program.data),
                        );
                    }
                    _ => panic!("Invalid argument for MOV instruction"),
                },
                InstructionKind::Add => exec_instr(|a, b| a + b),
                InstructionKind::Sub => exec_instr(|a, b| a - b),
                InstructionKind::Mul => exec_instr(|a, b| a * b),
                InstructionKind::Div => exec_instr(|a, b| a / b),
                InstructionKind::And => exec_instr(|a, b| a & b),
                InstructionKind::Or => exec_instr(|a, b| a | b),
                InstructionKind::Xor => exec_instr(|a, b| a ^ b),
                InstructionKind::Nor => exec_instr(|a, b| !(a | b)),
                InstructionKind::Slt => exec_instr(|a, b| if a < b { 1 } else { 0 }),
                InstructionKind::Sll => exec_instr(|a, b| a << b),
                InstructionKind::Srl => exec_instr(|a, b| a >> b),
                InstructionKind::Sra => exec_instr(|a, b| a >> b),
                InstructionKind::Jr => {
                    let address = load_word(&instruction.args[0], &registers, &program.data);
                    log::trace!("Jumping to address {}", address);
                }
                InstructionKind::Syscall => {
                    if !syscall(&registers, &program.data) {
                        break 'execution;
                    }
                }
                _ => unimplemented!("Instruction not implemented: {:?}", instruction),
            }
        }
    }
    log::trace!("Program executed.");
}

fn load_word(arg: &InstructionArg, registers: &HashMap<Register, i32>, data: &DataSection) -> i32 {
    match arg {
        InstructionArg::Immediate(value) => *value,
        InstructionArg::Register(r) => *registers.get(r).unwrap(),
        InstructionArg::Label(l) => {
            let data = data.globals.get(l).unwrap();
            i32::from_le_bytes(data.data[0..4].try_into().unwrap())
        }
    }
}

fn load_address(
    arg: &InstructionArg,
    registers: &HashMap<Register, i32>,
    data: &DataSection,
) -> Offset {
    match arg {
        InstructionArg::Immediate(value) => *value as Offset,
        InstructionArg::Register(r) => *registers.get(r).unwrap() as Offset,
        InstructionArg::Label(l) => data.globals.get(l).unwrap().address(),
    }
}

fn arithmetic<F>(
    registers: &mut HashMap<Register, i32>,
    data: &DataSection,
    args: &[InstructionArg],
    operation: F,
) where
    F: Fn(i32, i32) -> i32,
{
    match &args[0] {
        InstructionArg::Register(r) => {
            let value = load_word(&args[1], registers, data);
            let entry = registers.entry(*r).or_insert(0);
            *entry = operation(*entry, value);
        }
        _ => panic!("Invalid argument for instruction"),
    }
}

fn syscall(registers: &HashMap<Register, i32>, data: &DataSection) -> bool {
    let v0 = *registers.get(&Register::V0).unwrap();
    match v0 {
        1 => {
            let a0 = *registers.get(&Register::A0).unwrap();
            println!("{}", a0);
        }
        4 => {
            let a0 = *registers.get(&Register::A0).unwrap();
            let raw = &data.find_by_address(a0 as Offset).unwrap().data;
            print!("{}", std::str::from_utf8(raw).unwrap());
        }
        10 => {
            log::trace!("Exiting program...");
            // std::process::exit(0);
            return false;
        }
        _ => panic!("Invalid syscall number: {}", v0),
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
        let prog = prog.unwrap();
        log::trace!("====== Parsed Program ======\n{}", prog.show());
        log::trace!("====== Executing Program ======");
        execute(prog);
        log::trace!("====== Done ======");
    }
}
