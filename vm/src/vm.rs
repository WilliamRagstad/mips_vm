use crate::{
    address::Address,
    memory::{Memory, MemoryError, MemorySegment},
    program::{InstructionArg, InstructionKind as K, Program, Word},
    registers::{Register, Registers},
};

#[derive(Debug, PartialEq)]
pub enum VmError {
    Memory(MemoryError),
    InvalidOperands,
    Overflow,
    DivisionByZero,
    UnsupportedSyscall(u32),
    Input(String),
}
impl From<MemoryError> for VmError {
    fn from(e: MemoryError) -> Self {
        Self::Memory(e)
    }
}
pub type Result<T> = std::result::Result<T, VmError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Running,
    Halted,
}

pub struct VM {
    registers: Registers,
    memory: Memory,
    pc: Address,
    halted: bool,
}
impl VM {
    pub fn new(program: Program, mmio: Vec<MemorySegment>) -> Self {
        let memory = Memory::load(program, mmio);
        let pc = ["main", "entry", "start"]
            .iter()
            .find_map(|s| memory.address_of_label(s).ok())
            .unwrap_or(memory.text().start_address);
        Self {
            registers: Registers::default(),
            memory,
            pc,
            halted: false,
        }
    }
    pub fn entrypoint(&self) -> Option<Address> {
        Some(
            ["main", "entry", "start"]
                .iter()
                .find_map(|s| self.memory.address_of_label(s).ok())
                .unwrap_or(self.memory.text().start_address),
        )
    }
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    pub fn registers(&self) -> &Registers {
        &self.registers
    }
    pub fn pc(&self) -> Address {
        self.pc
    }
    pub fn halted(&self) -> bool {
        self.halted
    }
    /// Inspect at most 256 bytes. Does not invoke device read handlers.
    pub fn inspect(&self, address: Address, size: usize) -> Result<Vec<u8>> {
        if size == 0 || size > 256 {
            return Err(MemoryError::InvalidSize.into());
        }
        Ok(self.memory.inspect(address, size)?)
    }
    pub fn execute(&mut self, entrypoint: Address) -> Result<()> {
        self.pc = entrypoint;
        while self.step()? != Step::Halted {}
        Ok(())
    }
    fn value(&self, a: &InstructionArg) -> Result<Word> {
        match a {
            InstructionArg::Register(r) => Ok(self.registers.get(r)),
            InstructionArg::Immediate(i) => Ok(*i as u32),
            _ => Err(VmError::InvalidOperands),
        }
    }
    fn address(&self, a: &InstructionArg) -> Result<Address> {
        match a {
            InstructionArg::Label(l) => Ok(self.memory.address_of_label(l)?),
            InstructionArg::RegisterOffset(o, r) => Ok(Address::new(
                self.registers.get(r).wrapping_add(*o as i16 as i32 as u32),
            )),
            _ => Ok(Address::new(self.value(a)?)),
        }
    }
    /// Execute exactly one IR instruction. Faults leave PC at the faulting instruction.
    pub fn step(&mut self) -> Result<Step> {
        if self.halted {
            return Ok(Step::Halted);
        }
        if self.pc == self.memory.text().end_address {
            self.halted = true;
            return Ok(Step::Halted);
        }
        let i = self.memory.execute(self.pc)?.clone();
        let a = &i.args;
        let expected = match i.kind {
            K::Nop | K::Syscall => 0,
            K::J | K::Jal | K::Jr => 1,
            K::Li
            | K::La
            | K::Move
            | K::Lui
            | K::Jalr
            | K::Blez
            | K::Bgtz
            | K::Lb
            | K::Lbu
            | K::Lh
            | K::Lhu
            | K::Lw
            | K::Sb
            | K::Sh
            | K::Sw => 2,
            _ => 3,
        };
        if a.len() != expected {
            return Err(VmError::InvalidOperands);
        }
        let mut next = Address::new(self.pc.unwrap().wrapping_add(4));
        let dest = || match a.first() {
            Some(InstructionArg::Register(r)) => Ok(*r),
            _ => Err(VmError::InvalidOperands),
        };
        match i.kind {
            K::Nop => {}
            K::Syscall => {
                if !self.syscall()? {
                    self.halted = true;
                }
            }
            K::J | K::Jr | K::Jal => {
                let target = self.address(&a[0])?;
                self.memory.execute(target)?;
                if i.kind == K::Jal {
                    self.registers.set(&Register::Ra, next.unwrap());
                }
                next = target;
            }
            K::Jalr => {
                let r = dest()?;
                let target = self.address(&a[1])?;
                self.memory.execute(target)?;
                self.registers.set(&r, next.unwrap());
                next = target;
            }
            K::Beq | K::Bne | K::Blez | K::Bgtz => {
                let x = self.value(&a[0])?;
                let taken = match i.kind {
                    K::Beq => x == self.value(&a[1])?,
                    K::Bne => x != self.value(&a[1])?,
                    K::Blez => (x as i32) <= 0,
                    _ => (x as i32) > 0,
                };
                if taken {
                    next = match a.last().unwrap() {
                        InstructionArg::Immediate(o) => {
                            Address::new(next.unwrap().wrapping_add((*o as i16 as i32 * 4) as u32))
                        }
                        arg => self.address(arg)?,
                    };
                    self.memory.execute(next)?;
                }
            }
            K::Li | K::Move | K::La | K::Lui => {
                let r = dest()?;
                let v = match i.kind {
                    K::La => self.address(&a[1])?.unwrap(),
                    K::Lui => self.value(&a[1])? << 16,
                    _ => self.value(&a[1])?,
                };
                self.registers.set(&r, v);
            }
            K::Lb | K::Lbu | K::Lh | K::Lhu | K::Lw => {
                let r = dest()?;
                let address = self.address(&a[1])?;
                let width = match i.kind {
                    K::Lw => 4,
                    K::Lh | K::Lhu => 2,
                    _ => 1,
                };
                if address.unwrap() % width != 0 {
                    return Err(MemoryError::InvalidAddress.into());
                }
                let v = match i.kind {
                    K::Lb => self.memory.read_byte(address)? as i8 as u32,
                    K::Lbu => self.memory.read_byte(address)? as u32,
                    K::Lh => self.memory.read_halfword(address)? as i16 as u32,
                    K::Lhu => self.memory.read_halfword(address)? as u32,
                    _ => self.memory.read_word(address)?,
                };
                self.registers.set(&r, v);
            }
            K::Sb | K::Sh | K::Sw => {
                let v = self.value(&a[0])?;
                let address = self.address(&a[1])?;
                let width = match i.kind {
                    K::Sw => 4,
                    K::Sh => 2,
                    _ => 1,
                };
                if address.unwrap() % width != 0 {
                    return Err(MemoryError::InvalidAddress.into());
                }
                self.memory
                    .write(address, &v.to_le_bytes()[..width as usize])?;
            }
            _ => {
                let r = dest()?;
                let x = self.value(&a[1])?;
                let mut y = self.value(&a[2])?;
                if matches!(i.kind, K::Addi | K::Addiu | K::Slti | K::Sltiu) {
                    y = y as u16 as i16 as i32 as u32;
                }
                let v = match i.kind {
                    K::Add | K::Addi => {
                        (x as i32).checked_add(y as i32).ok_or(VmError::Overflow)? as u32
                    }
                    K::Sub => (x as i32).checked_sub(y as i32).ok_or(VmError::Overflow)? as u32,
                    K::Addu | K::Addiu => x.wrapping_add(y),
                    K::Subu => x.wrapping_sub(y),
                    K::Mult | K::Multu => x.wrapping_mul(y),
                    K::Div => {
                        if y == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        (x as i32).wrapping_div(y as i32) as u32
                    }
                    K::Divu => x.checked_div(y).ok_or(VmError::DivisionByZero)?,
                    K::And | K::Andi => x & y,
                    K::Or | K::Ori => x | y,
                    K::Xor | K::Xori => x ^ y,
                    K::Nor => !(x | y),
                    K::Slt | K::Slti => ((x as i32) < (y as i32)) as u32,
                    K::Sltu | K::Sltiu => (x < y) as u32,
                    K::Sll | K::Sllv => x << (y & 31),
                    K::Srl | K::Srlv => x >> (y & 31),
                    K::Sra | K::Srav => ((x as i32) >> (y & 31)) as u32,
                    _ => return Err(VmError::InvalidOperands),
                };
                self.registers.set(&r, v);
            }
        }
        self.pc = next;
        Ok(if self.halted {
            Step::Halted
        } else {
            Step::Running
        })
    }
    fn syscall(&mut self) -> Result<bool> {
        use std::io::{self, Write};
        let n = self.registers.get(&Register::V0);
        let a0 = self.registers.get(&Register::A0);
        match n {
            1 => print!("{}", a0 as i32),
            4 => {
                let mut address = Address::new(a0);
                loop {
                    let b = self.memory.read_byte(address)?;
                    if b == 0 {
                        break;
                    }
                    print!("{}", b as char);
                    address = Address::new(
                        address
                            .unwrap()
                            .checked_add(1)
                            .ok_or(MemoryError::OutOfBounds)?,
                    );
                }
            }
            5 | 12 => {
                let mut s = String::new();
                io::stdin()
                    .read_line(&mut s)
                    .map_err(|e| VmError::Input(e.to_string()))?;
                let v = if n == 5 {
                    s.trim()
                        .parse::<i32>()
                        .map_err(|e| VmError::Input(e.to_string()))? as u32
                } else {
                    s.bytes()
                        .next()
                        .ok_or_else(|| VmError::Input("EOF".into()))? as u32
                };
                self.registers.set(&Register::V0, v);
            }
            8 => {
                let size = self.registers.get(&Register::A1) as usize;
                if size > 0 {
                    let mut s = String::new();
                    io::stdin()
                        .read_line(&mut s)
                        .map_err(|e| VmError::Input(e.to_string()))?;
                    let mut bytes = s.into_bytes();
                    bytes.truncate(size - 1);
                    bytes.push(0);
                    self.memory.write(Address::new(a0), &bytes)?;
                }
            }
            9 => {
                let addr = self.memory.heap_allocate(a0 as usize)?;
                self.registers.set(&Register::V0, addr.unwrap());
            }
            10 | 17 => return Ok(false),
            11 => print!("{}", a0 as u8 as char),
            _ => return Err(VmError::UnsupportedSyscall(n)),
        }
        io::stdout()
            .flush()
            .map_err(|e| VmError::Input(e.to_string()))?;
        Ok(true)
    }
}
