use colorful::Colorful;

use crate::address::Address;
use crate::memory::MemorySegment;
use crate::{
    memory::Memory,
    program::{Instruction, InstructionArg, InstructionKind, Program, Word, LABEL_COLOR},
    registers::{Register, Registers},
};

pub struct VM {
    registers: Registers,
    memory: Memory,
}

impl VM {
    pub fn new(program: Program, mmio: Vec<MemorySegment>) -> Self {
        log::debug!(
            "{}\n{}",
            "======= LOADED PROGRAM =======".blue(),
            program.show_color()
        );
        let registers = Registers::default();
        let memory = Memory::load(program, mmio);
        log::trace!("Memory: {:#?}", memory);
        Self { registers, memory }
    }

    pub fn entrypoint(&self) -> Option<Address> {
        self.memory
            .labels()
            .iter()
            .find_map(|(label, address)| {
                if label.contains("main") || label.contains("entry") || label.contains("start") {
                    Some(*address)
                } else {
                    None
                }
            })
            .or(Some(self.memory.text().start_address))
        // Or return the first address of the text section
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn execute(&mut self, entrypoint: Address) {
        log::debug!("{}", "======= EXECUTION =======".blue());

        // Program counter (instruction pointer): address of the next instruction to execute
        let mut pc = entrypoint;

        'execution: loop {
            if let Ok(new_block) = self.memory.label_at_address(pc) {
                log::debug!(
                    "Executing block at {} {}...",
                    pc,
                    new_block.clone().color(LABEL_COLOR)
                );
            }
            let instruction = self
                .memory
                .execute(pc)
                .unwrap_or_else(|_| panic!("No instruction found at address {}", pc))
                .clone();
            let instruction_code = self.memory.read_word(pc).unwrap();
            log::debug!(
                "Executing instruction 0x{:08x} at {}: {}",
                instruction_code,
                pc,
                instruction.show_color()
            );

            // Move pointer to the next instruction in advance
            pc += Instruction::size();

            // Process the instruction
            match instruction.kind {
                InstructionKind::Li => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        let value = self.load_word(&instruction.args[1]);
                        self.registers.set(r, value);
                    }
                    _ => panic!("Invalid argument for LI instruction"),
                },
                InstructionKind::La => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        let addr = self.load_address(&instruction.args[1]);
                        self.registers.set(r, addr.unwrap() as Word);
                    }
                    _ => panic!("Invalid argument for LA instruction"),
                },
                InstructionKind::Move => match &instruction.args[0] {
                    InstructionArg::Register(r) => {
                        let value = self.load_word(&instruction.args[1]);
                        self.registers.set(r, value);
                    }
                    _ => panic!("Invalid argument for MOV instruction"),
                },
                InstructionKind::Add => self.arithmetic(&instruction.args, |a, b| a + b),
                InstructionKind::Sub => self.arithmetic(&instruction.args, |a, b| a - b),
                InstructionKind::Mult => self.arithmetic(&instruction.args, |a, b| a * b),
                InstructionKind::Div => self.arithmetic(&instruction.args, |a, b| a / b),
                InstructionKind::And => self.arithmetic(&instruction.args, |a, b| a & b),
                InstructionKind::Or => self.arithmetic(&instruction.args, |a, b| a | b),
                InstructionKind::Xor => self.arithmetic(&instruction.args, |a, b| a ^ b),
                InstructionKind::Nor => self.arithmetic(&instruction.args, |a, b| !(a | b)),
                InstructionKind::Slt => {
                    self.arithmetic(&instruction.args, |a, b| if a < b { 1 } else { 0 })
                }
                InstructionKind::Sll => self.arithmetic(&instruction.args, |a, b| a << b),
                InstructionKind::Srl => self.arithmetic(&instruction.args, |a, b| a >> b),
                InstructionKind::Sra => self.arithmetic(&instruction.args, |a, b| a >> b),
                InstructionKind::Jr => {
                    let address = self.load_address(&instruction.args[0]);
                    log::debug!("Jumping to address {}", address);
                }
                InstructionKind::Syscall => {
                    if !self.syscall() {
                        break 'execution;
                    }
                }
                InstructionKind::Addi => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for ADDI instruction"),
                    };
                    let src = self.load_word(&instruction.args[1]);
                    let imm = self.load_word(&instruction.args[2]);
                    self.registers.set(dest, src + imm);
                }
                InstructionKind::Andi => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for ANDI instruction"),
                    };
                    let src = self.load_word(&instruction.args[1]);
                    let imm = self.load_word(&instruction.args[2]);
                    self.registers.set(dest, src & imm);
                }
                InstructionKind::Beq => {
                    let lhs = self.load_word(&instruction.args[0]);
                    let rhs = self.load_word(&instruction.args[1]);
                    let offset = self.load_word(&instruction.args[2]);
                    if lhs == rhs {
                        pc += offset;
                    }
                }
                InstructionKind::Bne => {
                    let lhs = self.load_word(&instruction.args[0]);
                    let rhs = self.load_word(&instruction.args[1]);
                    let offset = self.load_word(&instruction.args[2]);
                    if lhs != rhs {
                        pc += offset;
                    }
                }
                InstructionKind::Lw => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for LW instruction"),
                    };
                    let src = Address::new(self.load_word(&instruction.args[1]));
                    let offset = self.load_word(&instruction.args[2]);
                    let address = src + offset;
                    let value = self.memory.read_word(address).unwrap();
                    self.registers.set(dest, value);
                }
                InstructionKind::Sw => {
                    let src = self.load_word(&instruction.args[0]);
                    let dest = Address::new(self.load_word(&instruction.args[1]));
                    self.memory.write(dest, &src.to_le_bytes()).unwrap();
                }
                InstructionKind::Lui => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for LUI instruction"),
                    };
                    let imm = self.load_word(&instruction.args[1]);
                    self.registers.set(dest, imm << 16);
                }
                InstructionKind::Nop => { /* Do nothing */ }
                InstructionKind::J => {
                    let address = self.load_address(&instruction.args[0]);
                    pc = address;
                }
                InstructionKind::Jal => {
                    let address = self.load_address(&instruction.args[0]);
                    self.registers.set(&Register::Ra, pc.unwrap() + 4);
                    pc = address;
                }
                InstructionKind::Addiu => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for ADDIU instruction"),
                    };
                    let src = self.load_word(&instruction.args[1]);
                    let imm = self.load_word(&instruction.args[2]);
                    self.registers.set(dest, src.wrapping_add(imm));
                }
                InstructionKind::Addu => {
                    self.arithmetic(&instruction.args, |a, b| a.wrapping_add(b))
                }
                InstructionKind::Blez => {
                    let src = self.load_word(&instruction.args[0]);
                    let offset = self.load_word(&instruction.args[1]);
                    if src as i32 <= 0 {
                        pc += offset;
                    }
                }
                InstructionKind::Bgtz => {
                    let src = self.load_word(&instruction.args[0]);
                    let offset = self.load_word(&instruction.args[1]);
                    if src as i32 > 0 {
                        pc += offset;
                    }
                }
                InstructionKind::Jalr => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for JALR instruction"),
                    };
                    let address = self.load_address(&instruction.args[1]);
                    self.registers.set(dest, pc.unwrap());
                    pc = address;
                }
                InstructionKind::Lb => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for LB instruction"),
                    };
                    let address = self.load_address(&instruction.args[1]);
                    let value = self.memory.read_byte(address).unwrap() as i8 as Word;
                    self.registers.set(dest, value);
                }
                InstructionKind::Lbu => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for LBU instruction"),
                    };
                    let address = self.load_address(&instruction.args[1]);
                    let value = self.memory.read_byte(address).unwrap() as Word;
                    self.registers.set(dest, value);
                }
                InstructionKind::Lh => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for LH instruction"),
                    };
                    let address = self.load_address(&instruction.args[1]);
                    let value = self.memory.read_halfword(address).unwrap() as i16 as Word;
                    self.registers.set(dest, value);
                }
                InstructionKind::Lhu => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for LHU instruction"),
                    };
                    let address = self.load_address(&instruction.args[1]);
                    let value = self.memory.read_halfword(address).unwrap() as Word;
                    self.registers.set(dest, value);
                }
                InstructionKind::Multu => {
                    self.arithmetic(&instruction.args, |a, b| a.wrapping_mul(b))
                }
                InstructionKind::Divu => self.arithmetic(&instruction.args, |a, b| a / b),
                InstructionKind::Ori => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for ORI instruction"),
                    };
                    let src = self.load_word(&instruction.args[1]);
                    let imm = self.load_word(&instruction.args[2]);
                    self.registers.set(dest, src | imm);
                }
                InstructionKind::Sltu => {
                    self.arithmetic(&instruction.args, |a, b| if a < b { 1 } else { 0 })
                }
                InstructionKind::Slti => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for SLTI instruction"),
                    };
                    let src = self.load_word(&instruction.args[1]);
                    let imm = self.load_word(&instruction.args[2]);
                    self.registers
                        .set(dest, if (src as i32) < (imm as i32) { 1 } else { 0 });
                }
                InstructionKind::Sltiu => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for SLTIU instruction"),
                    };
                    let src = self.load_word(&instruction.args[1]);
                    let imm = self.load_word(&instruction.args[2]);
                    self.registers.set(dest, if src < imm { 1 } else { 0 });
                }
                InstructionKind::Sllv => self.arithmetic(&instruction.args, |a, b| a << b),
                InstructionKind::Srav => {
                    self.arithmetic(&instruction.args, |a, b| (a as i32 >> b) as Word)
                }
                InstructionKind::Srlv => self.arithmetic(&instruction.args, |a, b| a >> b),
                InstructionKind::Sb => {
                    let value = self.load_word(&instruction.args[0]) as u8;
                    let address = self.load_address(&instruction.args[1]);
                    self.memory.write_byte(address, value).unwrap();
                }
                InstructionKind::Sh => {
                    let value = self.load_word(&instruction.args[0]) as u16;
                    let address = self.load_address(&instruction.args[1]);
                    self.memory.write_halfword(address, value).unwrap();
                }
                InstructionKind::Subu => {
                    self.arithmetic(&instruction.args, |a, b| a.wrapping_sub(b))
                }
                InstructionKind::Xori => {
                    let dest = match &instruction.args[0] {
                        InstructionArg::Register(r) => r,
                        _ => panic!("Invalid argument for XORI instruction"),
                    };
                    let src = self.load_word(&instruction.args[1]);
                    let imm = self.load_word(&instruction.args[2]);
                    self.registers.set(dest, src ^ imm);
                }
            }
        }
        log::debug!("{}", "====== Done ======".blue());
    }

    fn load_word(&mut self, arg: &InstructionArg) -> Word {
        match arg {
            InstructionArg::Immediate(value) => *value as Word,
            InstructionArg::Register(register) => self.registers.get(register),
            InstructionArg::RegisterOffset(offset, register) => {
                let base = Address::new(self.registers.get(register));
                let address = base + *offset;
                self.memory.read_word(address).unwrap_or_else(|err| {
                    panic!("Invalid address: {}: {:?}", address, err);
                })
            }
            InstructionArg::Label(label) => {
                let address = self.memory.address_of_label(label).unwrap();
                self.memory.read_word(address).unwrap()
            }
        }
    }

    fn load_address(&self, arg: &InstructionArg) -> Address {
        match arg {
            InstructionArg::Immediate(value) => Address::new(*value as u32),
            InstructionArg::Register(register) => Address::new(self.registers.get(register)),
            InstructionArg::RegisterOffset(offset, register) => {
                let base = Address::new(self.registers.get(register));
                base + *offset
            }
            InstructionArg::Label(label) => self.memory.address_of_label(label).unwrap(),
        }
    }

    fn arithmetic<F>(&mut self, args: &[InstructionArg], operation: F)
    where
        F: Fn(Word, Word) -> Word,
    {
        match &args[0] {
            InstructionArg::Register(r) => {
                let dest = self.load_word(&args[0]);
                let src = self.load_word(&args[1]);
                self.registers.set(r, operation(dest, src));
            }
            _ => panic!("Invalid argument for instruction"),
        }
    }

    fn syscall(&mut self) -> bool {
        let v0: Syscall = self.registers.get(&Register::V0).into();
        match v0 {
            Syscall::PrintInt => {
                let a0 = self.load_word(&InstructionArg::Register(Register::A0));
                print!("{}", a0);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            Syscall::PrintFloat => {
                let a0 = self.load_word(&InstructionArg::Register(Register::A0));
                print!("{}", f32::from_bits(a0));
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            Syscall::PrintDouble => {
                let a0 = self.load_word(&InstructionArg::Register(Register::A0));
                print!("{}", f64::from_bits(a0 as u64));
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            Syscall::PrintChar => {
                let a0 = self.load_word(&InstructionArg::Register(Register::A0));
                print!("{}", a0 as u8 as char);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            Syscall::PrintString => {
                const FLUSH_THRESHOLD: usize = 64;
                const BUFFER_SIZE: usize = 128;
                let a0 = Address::new(self.load_word(&InstructionArg::Register(Register::A0)));
                let mut addr = a0;
                let mut buffer = [0u8; BUFFER_SIZE];
                let mut i = 0;
                'print: loop {
                    match self.memory.read_buf_max(addr, &mut buffer) {
                        Ok(n) => {
                            for &byte in &buffer[..n] {
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
                            if n < BUFFER_SIZE {
                                // If we read less than the buffer size, we reached the end of the memory section
                                break 'print;
                            }
                            addr += buffer.len();
                        }
                        Err(err) => {
                            panic!(
                                "Invalid reading {} bytes at address {}: {:?}",
                                BUFFER_SIZE, addr, err
                            );
                        }
                    }
                }
                // Flush the remaining characters
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            Syscall::ReadInt => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let value = input.trim().parse::<Word>().unwrap();
                self.registers.set(&Register::V0, value);
            }
            Syscall::ReadFloat => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let value = input.trim().parse::<f32>().unwrap();
                self.registers.set(&Register::V0, value.to_bits() as Word);
            }
            Syscall::ReadDouble => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let value = input.trim().parse::<f64>().unwrap();
                self.registers.set(&Register::V0, value.to_bits() as Word);
            }
            Syscall::ReadChar => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let value = input.trim().chars().next().unwrap() as Word;
                self.registers.set(&Register::V0, value);
            }
            Syscall::ReadString => {
                let a0 = Address::new(self.load_word(&InstructionArg::Register(Register::A0))); // address of the buffer
                let a1 = self.load_word(&InstructionArg::Register(Register::A1)); // maximum number of characters to read
                                                                                  // TODO: Read at most `a1` characters from stdin
                let mut input = String::with_capacity(a1 as usize);
                std::io::stdin().read_line(&mut input).unwrap();
                self.memory
                    .write(a0, &input.as_bytes()[..a1 as usize])
                    .unwrap();
            }
            Syscall::Sbrk => {
                let a0 = self.load_word(&InstructionArg::Register(Register::A0));
                let address = self.memory.heap_allocate(a0 as usize).unwrap();
                self.registers.set(&Register::V0, address.unwrap());
            }
            Syscall::Exit | Syscall::Exit2 => {
                log::debug!("Exiting program...");
                // std::process::exit(0);
                return false;
            }
        };
        true
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

#[cfg(test)]
mod test_interpreter {
    use crate::{parser::parse, vm::VM};

    #[test]
    fn hello_world() {
        let input = include_str!("../../examples/hello_world.asm");
        let prog = parse(input);
        assert_ne!(prog, None);
        let program = prog.unwrap();
        let mut vm = VM::new(program, Vec::new());
        vm.execute(vm.entrypoint().expect("No entrypoint found"));
    }
}
