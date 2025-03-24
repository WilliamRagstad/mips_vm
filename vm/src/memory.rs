use std::{collections::HashMap, mem::size_of};

use crate::address::Address;
use crate::program::{Instruction, Program, Word};

#[derive(Debug, PartialEq)]
pub enum MemoryError {
    OutOfBounds,
    InvalidAddress,
    InvalidSize,
    InvalidValue,
    InvalidSection,
    InvalidLabel,
    InvalidInstruction,
    InvalidData,
    InvalidHeap,
    InvalidStack,
    SegmentFault,    // Invalid memory access
    ProtectionFault, // Invalid memory access
}

pub type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Debug, PartialEq, Default)]
pub enum ProtectionLevel {
    #[default]
    Read = 0b001,
    Write = 0b010,
    ReadWrite = Self::Read as isize | Self::Write as isize,
    Execute = 0b100,
    ReadExecute = Self::Read as isize | Self::Execute as isize,
    WriteExecute = Self::Write as isize | Self::Execute as isize,
    ReadWriteExecute = Self::Read as isize | Self::Write as isize | Self::Execute as isize,
}

impl ProtectionLevel {
    fn is_readable(&self) -> bool {
        matches!(
            self,
            Self::Read | Self::ReadWrite | Self::ReadExecute | Self::ReadWriteExecute
        )
    }

    fn is_writable(&self) -> bool {
        matches!(
            self,
            Self::Write | Self::ReadWrite | Self::WriteExecute | Self::ReadWriteExecute
        )
    }

    fn is_executable(&self) -> bool {
        matches!(
            self,
            Self::Execute | Self::ReadExecute | Self::WriteExecute | Self::ReadWriteExecute
        )
    }
}

type ReadHandler<T> = fn(Address) -> T;
type WriteHandler<T> = fn(Address, T);

/// Memory paging is a memory management scheme that eliminates the need for
/// contiguous allocation of physical memory.
#[derive(Debug, Default)]
pub struct MemorySection<T> {
    #[allow(dead_code)]
    name: String,
    protection: ProtectionLevel,
    pub start_address: Address,
    pub end_address: Address,
    data: Vec<T>,
    read_handler: Option<ReadHandler<T>>,
    write_handler: Option<WriteHandler<T>>,
}

impl<T> MemorySection<T> {
    pub fn read(&self) -> &[T] {
        if !self.protection.is_readable() {
            panic!("Memory section is not readable");
        }
        &self.data
    }

    pub fn write(&mut self) -> &mut Vec<T> {
        if !self.protection.is_writable() {
            panic!("Memory section is not writable");
        }
        &mut self.data
    }

    pub fn execute(&self) -> &[T] {
        if !self.protection.is_executable() {
            panic!("Memory section is not executable");
        }
        &self.data
    }
}

#[derive(Debug)]
pub struct Memory {
    /// Labels with their names as the key and their address as the value.
    /// This is used to store the mapping of all address of labels in the original program.
    labels: HashMap<String, Address>,
    /// Sections of memory with their start address as the key.
    sections: HashMap<Address, MemorySection<u8>>,
    /// Text section: contains the program's instructions
    /// This section is read-only and executable (code).
    text: MemorySection<Instruction>,
    /// Data section: contains initialized data
    /// This section is read-write and typically contains global variables.
    data: Option<Address>,
    /// Heap section: contains dynamically allocated memory
    /// This section is read-write and is used for dynamic memory allocation.
    /// The section is **allocated by the operating system at runtime**.
    /// The heap:
    /// - Is managed by the programmer using functions like malloc and free.
    /// - Is shared among all threads of a process.
    /// - Grows upwards, starting from a low address and growing towards higher addresses.
    heap: Address,
    /// Stack section: contains the stack
    /// This section is read-write and is used for function calls and local variables.
    /// The stack:
    /// - Is managed by the operating system and the compiler.
    /// - Is private to each thread of a process.
    /// - Is used for function calls, local variables, and bookkeeping information.
    /// - Grows downwards, starting from a high address and growing towards lower addresses.
    stack: Address,
}

impl Memory {
    /// Load the program into memory.
    /// The program memory is loaded  with the following sections:
    /// - `.text` section: read-only and executable (code) from the program's instructions. (**Lowest addresses**)
    /// - `.data` section: read-write and typically contains global variables from the initialized data. (**Slighly higher addresses**)
    /// - `.bss` section: read-write and is used for uninitialized data. (**Higher addresses**)
    /// - `.heap` section: read-write and is used for dynamic memory allocation from the dynamically allocated memory. (**Second-to-highest addresses**)
    /// - `.stack` section: read-write and is used for function calls and local variables from the stack. (**Highest addresses**)
    pub fn load(program: Program) -> Self {
        let mut address: Address = 0.into();
        let mut labels = HashMap::new();
        let mut sections = HashMap::new();

        if program.text_section.blocks.is_empty() {
            panic!("Invalid program: no .text code blocks found");
        }
        let text_start_address = address;
        let mut text_label_address: Address = text_start_address;
        for block in &program.text_section.blocks {
            if !block.label.is_empty() {
                labels.insert(block.label.clone(), text_label_address);
            }
            text_label_address += block.instructions.len() * Instruction::size();
        }
        let text_instructions = program.text_section.instructions_move();
        address += text_instructions.len() * Instruction::size();
        let text_end_address = address; // - Instruction::size().into();
        assert!(
            text_instructions.len()
                == ((text_end_address - text_start_address) / Instruction::size() as u32) as usize
        );

        let text: MemorySection<Instruction> = MemorySection {
            name: ".text".to_string(),
            protection: ProtectionLevel::ReadExecute,
            start_address: text_start_address,
            end_address: text_end_address,
            data: text_instructions,
            read_handler: None,
            write_handler: None,
        };

        let data = if !program.data_section.empty() {
            let data_start_address = address;
            let data_initialized = program.data_section.initialized_move();
            let mut data_label_address: Address = data_start_address;
            for data in &data_initialized {
                labels.insert(data.label.clone(), data_label_address);
                data_label_address += data.data.len();
            }
            let data_raw_initialized: Vec<u8> = data_initialized
                .into_iter()
                .flat_map(|rd| rd.data)
                .collect();
            address += data_raw_initialized.len();
            let data_end_address = address; // - 1;
            assert!(data_raw_initialized.len() == (data_end_address - data_start_address) as usize);

            let data = MemorySection {
                name: ".data".to_string(),
                protection: ProtectionLevel::ReadWrite,
                start_address: data_start_address,
                end_address: data_end_address,
                data: data_raw_initialized,
                read_handler: None,
                write_handler: None,
            };
            sections.insert(data_start_address, data);
            Some(data_start_address)
        } else {
            None
        };

        let heap = MemorySection {
            name: ".heap".to_string(),
            protection: ProtectionLevel::ReadWrite,
            start_address: 0x10000000.into(),
            end_address: 0x10000000.into(),
            data: Vec::new(),
            read_handler: None,
            write_handler: None,
        };
        let heap_address = heap.start_address;
        sections.insert(heap.start_address, heap);

        let stack = MemorySection {
            name: ".stack".to_string(),
            protection: ProtectionLevel::ReadWrite,
            start_address: 0x7fffefff.into(),
            end_address: 0x7fffefff.into(),
            data: Vec::new(),
            read_handler: None,
            write_handler: None,
        };
        let stack_address = stack.start_address;
        sections.insert(stack.start_address, stack);

        Memory {
            labels,
            sections,
            text,
            data,
            heap: heap_address,
            stack: stack_address,
        }
    }

    pub fn add_section(&mut self, section: MemorySection<u8>) {
        self.sections.insert(section.start_address, section);
    }

    pub fn address_of_label(&self, label: &str) -> Result<Address> {
        self.labels
            .get(label)
            .copied()
            .ok_or(MemoryError::InvalidLabel)
    }

    pub fn label_at_address(&self, address: Address) -> Result<&String> {
        self.labels
            .iter()
            .find_map(|(label, &addr)| if addr == address { Some(label) } else { None })
            .ok_or(MemoryError::InvalidAddress)
    }

    pub fn find_section(&self, address: Address) -> Result<&MemorySection<u8>> {
        self.sections
            .values()
            .find(|section| section.start_address <= address && address <= section.end_address)
            .ok_or(MemoryError::InvalidSection)
    }

    pub fn find_section_mut(&mut self, address: Address) -> Result<&mut MemorySection<u8>> {
        self.sections
            .values_mut()
            .find(|section| section.start_address <= address && address <= section.end_address)
            .ok_or(MemoryError::InvalidSection)
    }

    /// Read from a memory-mapped I/O address location.
    /// This is used to read from a memory-mapped I/O device.
    ///
    /// Write the result into the memory address location at `self.data`.
    fn mmio_read_to(section: &mut MemorySection<u8>, address: Address, size: usize) {
        if let Some(read_handler) = section.read_handler {
            let offset = (address - section.start_address) as usize;
            for i in 0..size {
                section.write()[offset + i] = read_handler(address + i);
            }
        }
    }

    /// Write to an memory-mapped I/O address location.
    /// This is used to write to a memory-mapped I/O device.
    fn mmio_write_to(section: &mut MemorySection<u8>, address: Address, value: &[u8]) {
        if let Some(write_handler) = section.write_handler {
            (0..value.len()).for_each(|i| {
                write_handler(address + i, value[i]);
            });
        }
    }

    /// Read from a memory address location and return the data of the specified size
    pub fn read(&mut self, address: Address, size: usize) -> Result<Vec<u8>> {
        let section = self.find_section_mut(address)?;
        if address + size > section.end_address {
            return Err(MemoryError::OutOfBounds); // Out of bounds
        }
        let offset = (address - section.start_address) as usize;
        Self::mmio_read_to(section, address, size);
        section
            .read()
            .get(offset..offset + size)
            .map(|d| d.to_vec())
            .ok_or(MemoryError::InvalidSize)
    }

    pub fn read_buf(&mut self, address: Address, buf: &mut [u8]) -> Result<()> {
        let section = self.find_section_mut(address)?;
        if address + buf.len() > section.end_address {
            return Err(MemoryError::OutOfBounds); // Out of bounds
        }
        let offset = (address - section.start_address) as usize;
        let size = buf.len();
        Self::mmio_read_to(section, address, size);
        let src = &section.read()[offset..offset + size];
        if src.is_empty() {
            return Err(MemoryError::InvalidSize);
        }
        buf[..size].copy_from_slice(src);
        Ok(())
    }

    pub fn read_const<const N: usize>(&mut self, address: Address) -> Result<[u8; N]> {
        let mut data = [0; N];
        self.read_buf(address, &mut data)?;
        Ok(data)
    }

    pub fn read_max(&mut self, address: Address, max_size: usize) -> Result<Vec<u8>> {
        let section = self.find_section_mut(address)?;
        let size = max_size.min((section.end_address - address) as usize);
        self.read(address, size)
    }

    pub fn read_buf_max(&mut self, address: Address, buf: &mut [u8]) -> Result<usize> {
        let section = self.find_section_mut(address)?;
        let size = buf.len().min((section.end_address - address) as usize);
        self.read_buf(address, &mut buf[..size])?;
        Ok(size)
    }

    pub fn read_word(&mut self, address: Address) -> Result<Word> {
        let mut data = [0; size_of::<Word>()];
        self.read_buf(address, &mut data)?;
        Ok(Word::from_le_bytes(data))
    }

    pub fn read_address(&mut self, address: Address) -> Result<Address> {
        let mut data = [0; size_of::<Address>()];
        self.read_buf(address, &mut data)?;
        Ok(Address::from_le_bytes(data))
    }

    /// Write to a memory address location.
    /// The value is written in between `(start_address + offset)` to `(start_address + offset + value.len())`.
    pub fn write(&mut self, address: Address, value: &[u8]) -> Result<()> {
        let section = self.find_section_mut(address)?;
        if address + value.len() > section.end_address {
            return Err(MemoryError::OutOfBounds); // Out of bounds
        }
        let offset = (address - section.start_address) as usize;
        Self::mmio_write_to(section, address, value);
        section
            .write()
            .get_mut(offset..offset + value.len())
            .ok_or(MemoryError::InvalidSize)?
            .copy_from_slice(value);
        Ok(())
    }

    /// Currently, only the text section will be executable
    pub fn execute(&self, address: Address) -> Result<&Instruction> {
        if self.text.start_address <= address && address <= self.text.end_address {
            // TODO: Improve finding the instruction by address performance
            let index = (address - self.text.start_address) as usize / Instruction::size();
            self.text
                .execute()
                .get(index)
                .ok_or(MemoryError::InvalidInstruction)
            // self.text
            //     .execute()
            //     .iter()
            //     .find(|i| i.address == address)
            //     .ok_or(MemoryError::InvalidInstruction)
        } else {
            Err(MemoryError::ProtectionFault)
        }
    }

    pub fn labels(&self) -> &HashMap<String, Address> {
        &self.labels
    }

    pub fn text(&self) -> &MemorySection<Instruction> {
        &self.text
    }

    pub fn data(&self) -> Option<&MemorySection<u8>> {
        self.sections.get(&self.data?)
    }

    pub fn data_mut(&mut self) -> Option<&mut MemorySection<u8>> {
        self.sections.get_mut(&self.data?)
    }

    pub fn heap(&self) -> &MemorySection<u8> {
        self.sections.get(&self.heap).unwrap()
    }

    pub fn heap_mut(&mut self) -> &mut MemorySection<u8> {
        self.sections.get_mut(&self.heap).unwrap()
    }

    pub fn stack(&self) -> &MemorySection<u8> {
        self.sections.get(&self.stack).unwrap()
    }

    pub fn stack_mut(&mut self) -> &mut MemorySection<u8> {
        self.sections.get_mut(&self.stack).unwrap()
    }

    /// Push a byte to the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address -= 1` to adjust the range of the stack section.
    ///
    /// Returns:
    /// - `Ok(())` if the push is successful.
    pub fn stack_push(&mut self, value: u8) -> Result<()> {
        // Check if the stack section is colliding with the heap section
        if self.stack().start_address - 1 <= self.heap().end_address {
            return Err(MemoryError::InvalidStack);
        }
        self.stack_mut().write().push(value);
        self.stack_mut().start_address -= 1;
        Ok(())
    }

    /// Pop a byte from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += 1` to adjust the range of the stack section.
    pub fn stack_pop(&mut self) -> Result<u8> {
        let res = self.stack_mut().write().pop();
        if res.is_some() {
            self.stack_mut().start_address += 1;
        }
        res.ok_or(MemoryError::InvalidStack)
    }

    /// Push a word to the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address -= WORD_SIZE` to adjust the range of the stack section.
    ///
    /// Returns:
    /// - `Ok(())` if the push is successful.
    /// - `Err` if the stack section is colliding with the heap section.
    pub fn stack_push_word(&mut self, value: Word) -> Result<()> {
        const WORD_SIZE: usize = size_of::<Word>();
        // Check if the stack section is colliding with the heap section
        if self.stack().start_address - WORD_SIZE <= self.heap().end_address {
            return Err(MemoryError::InvalidStack);
        }
        let bytes: [u8; WORD_SIZE] = value.to_le_bytes();
        self.stack_mut().write().extend_from_slice(&bytes);
        self.stack_mut().start_address -= WORD_SIZE;
        Ok(())
    }

    /// Pop a word from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += WORD_SIZE` to adjust the range of the stack section.
    pub fn stack_pop_word(&mut self) -> Result<Word> {
        const WORD_SIZE: usize = size_of::<Word>();
        let mut bytes = [0; WORD_SIZE];
        #[allow(clippy::needless_range_loop)]
        for i in 0..WORD_SIZE {
            bytes[i] = self.stack_pop()?;
        }
        Ok(Word::from_le_bytes(bytes))
    }

    /// Push an address to the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address -= ADDRESS_SIZE` to adjust the range of the stack section.
    ///
    /// Returns:
    /// - `Some(())` if the push is successful.
    /// - `None` if the stack section is colliding with the heap section.
    pub fn stack_push_address(&mut self, value: Address) -> Result<()> {
        const ADDRESS_SIZE: usize = size_of::<Address>();
        // Check if the stack section is colliding with the heap section
        if self.stack().start_address - ADDRESS_SIZE <= self.heap().end_address {
            return Err(MemoryError::InvalidStack);
        }
        let bytes: [u8; ADDRESS_SIZE] = value.to_le_bytes();
        self.stack_mut().write().extend_from_slice(&bytes);
        self.stack_mut().start_address -= ADDRESS_SIZE;
        Ok(())
    }

    /// Pop an address from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += ADDRESS_SIZE` to adjust the range of the stack section.
    pub fn stack_pop_address(&mut self) -> Result<Address> {
        const ADDRESS_SIZE: usize = size_of::<Address>();
        let mut bytes = [0; ADDRESS_SIZE];
        #[allow(clippy::needless_range_loop)]
        for i in 0..ADDRESS_SIZE {
            bytes[i] = self.stack_pop()?;
        }
        Ok(Address::from_le_bytes(bytes))
    }

    /// Allocate memory on the heap of a given size (number of bytes).
    /// The heap grows upwards (from low address to higher addresses),
    /// so the `end_address += size` to adjust the range of the heap section.
    ///
    /// Returns:
    /// - `Some(Address)` if the allocation is successful, with the address of the allocated memory.
    /// - `None` if the allocation is unsuccessful, due to out-of-memory or heap-stack collision.
    pub fn heap_allocate(&mut self, size: usize) -> Result<Address> {
        // Check if the heap section is colliding with the stack section
        if self.heap().end_address + size >= self.stack().start_address {
            return Err(MemoryError::InvalidHeap);
        }
        let address = self.heap_mut().end_address;
        self.heap_mut().end_address += size;
        Ok(address)
    }

    /// Deallocate memory on the heap of a given size (number of bytes).
    /// This is done by writing zeros to the memory at the address.
    /// No need to adjust the range of the heap section.
    pub fn heap_deallocate(&mut self, address: Address, size: usize) {
        // Nuke the data at the address
        let _ = self.write(address, &(vec![0; size]));
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for section in self.sections.values() {
            if section.data.is_empty() {
                continue;
            }
            let start = section.start_address.unwrap() as usize;
            let end = section.end_address.unwrap() as usize;
            if buf.len() < end {
                buf.resize(end, 0);
            }
            buf[start..end].copy_from_slice(section.read());
        }
        // Fill in text section with 0xFF
        let text_end = self.text.end_address.unwrap() as usize;
        let text_start = self.text.start_address.unwrap() as usize;
        let text_size = text_end - text_start + 1;
        if buf.len() < ((text_end + text_size) / Instruction::size()) {
            buf.resize(text_end + text_size, 0);
        }
        const CODE: u32 = 0xEEDD00CC;
        (text_start..text_end).for_each(|i| {
            buf[i] = (CODE >> (((i - text_start) % 4) * 8)) as u8;
        });
        buf
    }
}
