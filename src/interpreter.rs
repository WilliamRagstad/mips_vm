use colorful::Colorful;

use crate::{
    memory::Memory,
    program::{Address, Instruction, InstructionArg, InstructionKind, Program, Word, LABEL_COLOR},
    registers::{Register, Registers},
};

pub fn execute(program: Program, entrypoint: Address) {
    log::debug!(
        "{}\n{}",
        "======= EXECUTE =======".blue(),
        program.show_color()
    );
    log::debug!("{}", "======= OUTPUT =======".blue());

    let mut registers = Registers::default();
    let mut memory = Memory::load(program);

    // Dump the memory before execution
    let dump = memory.dump();
    let cwd = std::env::current_dir().unwrap();
    let dump_path = cwd.join("init_dump.bin");
    std::fs::write(&dump_path, dump).unwrap();

    // Program counter (instruction pointer): address of the next instruction to execute
    let mut pc = entrypoint;

    'execution: loop {
        if let Some(new_block) = memory.label_at_address(pc) {
            log::debug!(
                "Executing block at 0x{:08x} {}...",
                pc,
                new_block.clone().color(LABEL_COLOR)
            );
        }
        let instruction = memory
            .execute(pc)
            .unwrap_or_else(|| panic!("No instruction found at address 0x{:08x}", pc))
            .clone();
        log::debug!(
            "Executing instruction at 0x{:08x}: {}",
            pc,
            instruction.show_color()
        );

        // Move pointer to the next instruction in advance
        pc += Instruction::size() as Address;

        // Process the instruction
        match instruction.kind {
            InstructionKind::Li => match &instruction.args[0] {
                InstructionArg::Register(r) => {
                    let value = load_word(&instruction.args[1], &registers, &mut memory);
                    registers.set(r, value);
                }
                _ => panic!("Invalid argument for LI instruction"),
            },
            InstructionKind::La => match &instruction.args[0] {
                InstructionArg::Register(r) => {
                    let addr = load_address(&instruction.args[1], &registers, &mut memory);
                    registers.set(r, addr as Word);
                }
                _ => panic!("Invalid argument for LA instruction"),
            },
            InstructionKind::Move => match &instruction.args[0] {
                InstructionArg::Register(r) => {
                    registers.set(r, load_word(&instruction.args[1], &registers, &mut memory));
                }
                _ => panic!("Invalid argument for MOV instruction"),
            },
            InstructionKind::Add => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| a + b)
            }
            InstructionKind::Sub => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| a - b)
            }
            InstructionKind::Mul => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| a * b)
            }
            InstructionKind::Div => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| a / b)
            }
            InstructionKind::And => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| a & b)
            }
            InstructionKind::Or => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| a | b)
            }
            InstructionKind::Xor => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| a ^ b)
            }
            InstructionKind::Nor => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| {
                    !(a | b)
                })
            }
            InstructionKind::Slt => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| {
                    if a < b {
                        1
                    } else {
                        0
                    }
                })
            }
            InstructionKind::Sll => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| {
                    a << b
                })
            }
            InstructionKind::Srl => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| {
                    a >> b
                })
            }
            InstructionKind::Sra => {
                arithmetic(&mut registers, &mut memory, &instruction.args, |a, b| {
                    a >> b
                })
            }
            InstructionKind::Jr => {
                let address = load_address(&instruction.args[0], &registers, &mut memory);
                log::debug!("Jumping to address {}", address);
            }
            InstructionKind::Syscall => {
                if !syscall(&mut registers, &mut memory) {
                    break 'execution;
                }
            }
            InstructionKind::Addi => {
                let dest = match &instruction.args[0] {
                    InstructionArg::Register(r) => r,
                    _ => panic!("Invalid argument for ADDI instruction"),
                };
                let src = load_word(&instruction.args[1], &registers, &mut memory);
                let imm = load_word(&instruction.args[2], &registers, &mut memory);
                registers.set(dest, src + imm);
            }
            InstructionKind::Andi => {
                let dest = match &instruction.args[0] {
                    InstructionArg::Register(r) => r,
                    _ => panic!("Invalid argument for ANDI instruction"),
                };
                let src = load_word(&instruction.args[1], &registers, &mut memory);
                let imm = load_word(&instruction.args[2], &registers, &mut memory);
                registers.set(dest, src & imm);
            }
            InstructionKind::Beq => {
                let lhs = load_word(&instruction.args[0], &registers, &mut memory);
                let rhs = load_word(&instruction.args[1], &registers, &mut memory);
                let offset = load_word(&instruction.args[2], &registers, &mut memory);
                if lhs == rhs {
                    pc += offset as Address;
                }
            }
            InstructionKind::Bne => {
                let lhs = load_word(&instruction.args[0], &registers, &mut memory);
                let rhs = load_word(&instruction.args[1], &registers, &mut memory);
                let offset = load_word(&instruction.args[2], &registers, &mut memory);
                if lhs != rhs {
                    pc += offset as Address;
                }
            }
            InstructionKind::Lw => {
                let dest = match &instruction.args[0] {
                    InstructionArg::Register(r) => r,
                    _ => panic!("Invalid argument for LW instruction"),
                };
                let src = load_word(&instruction.args[1], &registers, &mut memory);
                let offset = load_word(&instruction.args[2], &registers, &mut memory);
                let address = src + offset;
                let data: [u8; size_of::<Word>()] = memory.read_const(address).unwrap();
                let value = Word::from_le_bytes(data);
                registers.set(&dest, value);
            }
            InstructionKind::Sw => {
                let src = load_word(&instruction.args[0], &registers, &mut memory);
                let dest = load_word(&instruction.args[1], &registers, &mut memory);
                memory.write(dest as Address, &src.to_le_bytes()).unwrap();
            }
            InstructionKind::Lui => {
                let dest = match &instruction.args[0] {
                    InstructionArg::Register(r) => r,
                    _ => panic!("Invalid argument for LUI instruction"),
                };
                let imm = load_word(&instruction.args[1], &registers, &mut memory);
                registers.set(dest, imm << 16);
            }
            InstructionKind::Nop => { /* Do nothing */ }
            InstructionKind::B => {
                let offset = load_word(&instruction.args[0], &registers, &mut memory);
                pc += offset as Address;
            }
            InstructionKind::J => {
                let address = load_address(&instruction.args[0], &registers, &memory);
                pc = address;
            }
            InstructionKind::Jal => {
                let address = load_address(&instruction.args[0], &registers, &memory);
                registers.set(&Register::Ra, pc + 4);
                pc = address;
            }
        }
    }
    log::debug!("{}", "====== Done ======".blue());
}

fn load_word(arg: &InstructionArg, registers: &Registers, memory: &mut Memory) -> Word {
    match arg {
        InstructionArg::Immediate(value) => *value as Word,
        InstructionArg::Register(register) => registers.get(register),
        InstructionArg::RegisterOffset(register, offset) => {
            let base = registers.get(register);
            let address = base as Address + *offset as Address;
            let result: Option<[u8; size_of::<Word>()]> = memory.read_const(address);
            let Some(data) = result else {
                panic!("Invalid address: 0x{:08x}", address);
            };
            Word::from_le_bytes(data)
        }
        InstructionArg::Label(label) => {
            let address = memory.address_of_label(label).expect("Label not found");
            let data: [u8; size_of::<Word>()] = memory.read_const(address).unwrap();
            Word::from_le_bytes(data)
        }
    }
}

fn load_address(arg: &InstructionArg, registers: &Registers, memory: &Memory) -> Address {
    match arg {
        InstructionArg::Immediate(value) => *value as Address,
        InstructionArg::Register(register) => registers.get(register),
        InstructionArg::RegisterOffset(register, offset) => {
            let base = registers.get(register);
            base as Address + *offset as Address
        }
        InstructionArg::Label(label) => memory.address_of_label(label).expect("Label not found"),
    }
}

fn arithmetic<F>(
    registers: &mut Registers,
    memory: &mut Memory,
    args: &[InstructionArg],
    operation: F,
) where
    F: Fn(Word, Word) -> Word,
{
    match &args[0] {
        InstructionArg::Register(r) => {
            let dest = load_word(&args[0], registers, memory);
            let src = load_word(&args[1], registers, memory);
            registers.set(r, operation(dest, src));
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
    /// Sbrk - Increment the program break (brk) pointer
    /// - `a0`: Number of bytes to increment the program break pointer
    /// - `v0`: Address of the new program break pointer
    ///
    /// On success, the syscall returns the address of the new program break pointer,
    /// which is the address of the first newly allocated byte.
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

fn syscall(registers: &mut Registers, memory: &mut Memory) -> bool {
    let v0: Syscall = registers.get(&Register::V0).into();
    match v0 {
        Syscall::PrintInt => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, memory);
            print!("{}", a0);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        Syscall::PrintFloat => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, memory);
            print!("{}", f32::from_bits(a0 as u32));
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        Syscall::PrintDouble => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, memory);
            print!("{}", f64::from_bits(a0 as u64));
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        Syscall::PrintChar => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, memory);
            print!("{}", a0 as u8 as char);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        Syscall::PrintString => {
            const FLUSH_THRESHOLD: usize = 64;
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, memory);
            let mut addr = a0 as Address;
            let mut buffer = [0u8; 128];
            let mut i = 0;
            'print: loop {
                let bytes_res = memory.read_buf(addr, &mut buffer);
                if bytes_res.is_none() {
                    // If we read less than the buffer size, we reached the end of the memory section
                    panic!("Invalid address: 0x{:08x}", addr);
                };
                for &byte in &buffer {
                    if byte == 0 {
                        break 'print;
                    }
                    print!("{}", byte as char);
                    i += 1;
                    if i % FLUSH_THRESHOLD == 0 {
                        // Flush every 64 characters
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }
                }
                addr += buffer.len() as Address;
            }
            // Flush the remaining characters
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        Syscall::ReadInt => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().parse::<Word>().unwrap();
            registers.set(&Register::V0, value);
        }
        Syscall::ReadFloat => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().parse::<f32>().unwrap();
            registers.set(&Register::V0, value.to_bits() as Word);
        }
        Syscall::ReadDouble => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().parse::<f64>().unwrap();
            registers.set(&Register::V0, value.to_bits() as Word);
        }
        Syscall::ReadChar => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let value = input.trim().chars().next().unwrap() as Word;
            registers.set(&Register::V0, value);
        }
        Syscall::ReadString => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, memory); // address of the buffer
            let a1 = load_word(&InstructionArg::Register(Register::A1), registers, memory); // maximum number of characters to read
                                                                                            // TODO: Read at most `a1` characters from stdin
            let mut input = String::with_capacity(a1 as usize);
            std::io::stdin().read_line(&mut input).unwrap();
            memory
                .write(a0 as Address, &input.as_bytes()[..a1 as usize])
                .unwrap();
        }
        Syscall::Sbrk => {
            let a0 = load_word(&InstructionArg::Register(Register::A0), registers, memory);
            let address = memory
                .heap_allocate(a0 as usize)
                .expect("Failed to allocate memory");
            registers.set(&Register::V0, address as Word);
        }
        Syscall::Exit | Syscall::Exit2 => {
            log::debug!("Exiting program...");
            // std::process::exit(0);
            return false;
        }
    };
    true
}

#[cfg(test)]
mod test_interpreter {
    use crate::{execute, parse};

    #[test]
    fn hello_world() {
        let input = include_str!("../examples/hello_world.asm");
        let prog = parse(input);
        assert_ne!(prog, None);
        let program = prog.unwrap();
        let entry = program
            .text
            .entry_block()
            .expect("No entry block found")
            .address;
        execute(program, entry);
    }
}
