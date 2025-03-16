use std::{collections::HashMap, mem::size_of};

use crate::program::{Address, Instruction, Program, Word};

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

#[derive(Debug, Default)]
pub struct MemorySection<T> {
    protection: ProtectionLevel,
    start_address: Address,
    end_address: Address,
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
    pub fn load(program: Program) -> Self {
        let mut labels = HashMap::new();
        for (name, data) in &program.data.globals {
            labels.insert(name.clone(), data.address);
        }
        for block in &program.text.blocks {
            if !block.label.is_empty() {
                labels.insert(block.label.clone(), block.address);
            }
        }

        let mut sections = HashMap::new();

        if program.text.blocks.is_empty() {
            panic!("Invalid program: no .text code blocks found");
        }
        let text: MemorySection<Instruction> = MemorySection {
            protection: ProtectionLevel::ReadExecute,
            start_address: program.text.blocks.first().unwrap().address,
            end_address: program
                .text
                .blocks
                .last()
                .unwrap()
                .instructions
                .last()
                .unwrap()
                .address,
            data: program.text.instructions_move(),
            read_handler: None,
            write_handler: None,
        };

        let data = if !program.data.empty() {
            let data = MemorySection {
                protection: ProtectionLevel::ReadWrite,
                start_address: program
                    .data
                    .data()
                    .iter()
                    .map(|d| d.address())
                    .min()
                    .unwrap_or(text.end_address + 1),
                end_address: program
                    .data
                    .data()
                    .iter()
                    .map(|d| d.address())
                    .max()
                    .unwrap_or(text.end_address + 1),
                data: program
                    .data
                    .data_move()
                    .into_iter()
                    .flat_map(|rd| rd.data)
                    .collect(),
                read_handler: None,
                write_handler: None,
            };
            let data_address = data.start_address;
            sections.insert(data.start_address, data);
            Some(data_address)
        } else {
            None
        };

        let heap = MemorySection {
            protection: ProtectionLevel::ReadWrite,
            start_address: 0x10000000,
            end_address: 0x10000000,
            data: Vec::new(),
            read_handler: None,
            write_handler: None,
        };
        let heap_address = heap.start_address;
        sections.insert(heap.start_address, heap);

        let stack = MemorySection {
            protection: ProtectionLevel::ReadWrite,
            start_address: 0x7fffefff,
            end_address: 0x7fffefff,
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

    pub fn address_of_label(&self, label: &str) -> Option<Address> {
        self.labels.get(label).copied()
    }

    pub fn label_at_address(&self, address: Address) -> Option<&String> {
        self.labels
            .iter()
            .find_map(|(label, &addr)| if addr == address { Some(label) } else { None })
    }

    pub fn find_section(&self, address: Address) -> Option<&MemorySection<u8>> {
        self.sections
            .values()
            .find(|section| section.start_address <= address && address <= section.end_address)
    }

    pub fn find_section_mut(&mut self, address: Address) -> Option<&mut MemorySection<u8>> {
        self.sections
            .values_mut()
            .find(|section| section.start_address <= address && address <= section.end_address)
    }

    /// Read from a memory-mapped I/O address location.
    /// This is used to read from a memory-mapped I/O device.
    ///
    /// Write the result into the memory address location at `self.data`.
    fn mmio_read_to(section: &mut MemorySection<u8>, address: Address, size: usize) {
        if let Some(read_handler) = section.read_handler {
            let offset = (address - section.start_address) as usize;
            for i in 0..size {
                section.write()[offset + i] = read_handler(address + i as Address);
            }
        }
    }

    /// Read from a memory address location and return the data of the specified size
    pub fn read(&mut self, address: Address, size: usize) -> Option<Vec<u8>> {
        let section = self.find_section_mut(address)?;
        let offset = (address - section.start_address) as usize;
        Self::mmio_read_to(section, address, size);
        section
            .read()
            .get(offset..offset + size)
            .map(|d| d.to_vec())
    }

    pub fn read_buf(&mut self, address: Address, buf: &mut [u8]) -> Option<()> {
        let section = self.find_section_mut(address)?;
        let offset = (address - section.start_address) as usize;
        Self::mmio_read_to(section, address, buf.len());
        let len = buf.len();
        buf[..len].copy_from_slice(&section.read()[offset..len]);
        Some(())
    }

    pub fn read_const<const N: usize>(&mut self, address: Address) -> Option<[u8; N]> {
        let mut data = [0; N];
        self.read_buf(address, &mut data)?;
        Some(data)
    }

    /// Write to a memory address location.
    /// The value is written in between `(start_address + offset)` to `(start_address + offset + value.len())`.
    pub fn write(&mut self, address: Address, value: &[u8]) -> Option<()> {
        let s = self.find_section_mut(address)?;
        let offset = (address - s.start_address) as usize;
        s.write()
            .get_mut(offset..offset + value.len())?
            .copy_from_slice(value);
        Some(())
    }

    /// Currently, only the text section will be executable
    pub fn execute(&self, address: Address) -> Option<&Instruction> {
        if self.text.start_address <= address && address <= self.text.end_address {
            // TODO: Improve finding the instruction by address performance
            // let index = (address - self.text.start_address) as usize / Instruction::size();
            // return self.text.execute().get(index);
            self.text.execute().iter().find(|i| i.address == address)
        } else {
            None
        }
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
    /// - `Some(())` if the push is successful.
    pub fn stack_push(&mut self, value: u8) -> Option<()> {
        // Check if the stack section is colliding with the heap section
        if self.stack().start_address - 1 <= self.heap().end_address {
            return None;
        }
        self.stack_mut().write().push(value);
        self.stack_mut().start_address -= 1;
        Some(())
    }

    /// Pop a byte from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += 1` to adjust the range of the stack section.
    pub fn stack_pop(&mut self) -> Option<u8> {
        let res = self.stack_mut().write().pop();
        if res.is_some() {
            self.stack_mut().start_address += 1;
        }
        res
    }

    /// Push a word to the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address -= WORD_SIZE` to adjust the range of the stack section.
    ///
    /// Returns:
    /// - `Some(())` if the push is successful.
    /// - `None` if the stack section is colliding with the heap section.
    pub fn stack_push_word(&mut self, value: Word) -> Option<()> {
        const WORD_SIZE: usize = size_of::<Word>();
        // Check if the stack section is colliding with the heap section
        if self.stack().start_address - WORD_SIZE as Address <= self.heap().end_address {
            return None;
        }
        let bytes: [u8; WORD_SIZE] = value.to_le_bytes();
        self.stack_mut().write().extend_from_slice(&bytes);
        self.stack_mut().start_address -= WORD_SIZE as Address;
        Some(())
    }

    /// Pop a word from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += WORD_SIZE` to adjust the range of the stack section.
    pub fn stack_pop_word(&mut self) -> Option<Word> {
        const WORD_SIZE: usize = size_of::<Word>();
        let mut bytes = [0; WORD_SIZE];
        #[allow(clippy::needless_range_loop)]
        for i in 0..WORD_SIZE {
            bytes[i] = self.stack_pop()?;
        }
        Some(Word::from_le_bytes(bytes))
    }

    /// Push an address to the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address -= ADDRESS_SIZE` to adjust the range of the stack section.
    ///
    /// Returns:
    /// - `Some(())` if the push is successful.
    /// - `None` if the stack section is colliding with the heap section.
    pub fn stack_push_address(&mut self, value: Address) -> Option<()> {
        const ADDRESS_SIZE: usize = size_of::<Address>();
        // Check if the stack section is colliding with the heap section
        if self.stack().start_address - ADDRESS_SIZE as Address <= self.heap().end_address {
            return None;
        }
        let bytes: [u8; ADDRESS_SIZE] = value.to_le_bytes();
        self.stack_mut().write().extend_from_slice(&bytes);
        self.stack_mut().start_address -= ADDRESS_SIZE as Address;

        Some(())
    }

    /// Pop an address from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += ADDRESS_SIZE` to adjust the range of the stack section.
    pub fn stack_pop_address(&mut self) -> Option<Address> {
        const ADDRESS_SIZE: usize = size_of::<Address>();
        let mut bytes = [0; ADDRESS_SIZE];
        #[allow(clippy::needless_range_loop)]
        for i in 0..ADDRESS_SIZE {
            bytes[i] = self.stack_pop()?;
        }
        Some(Address::from_le_bytes(bytes))
    }

    /// Allocate memory on the heap of a given size (number of bytes).
    /// The heap grows upwards (from low address to higher addresses),
    /// so the `end_address += size` to adjust the range of the heap section.
    ///
    /// Returns:
    /// - `Some(Address)` if the allocation is successful, with the address of the allocated memory.
    /// - `None` if the allocation is unsuccessful, due to out-of-memory or heap-stack collision.
    pub fn heap_allocate(&mut self, size: usize) -> Option<Address> {
        // Check if the heap section is colliding with the stack section
        if self.heap().end_address + size as Address >= self.stack().start_address {
            return None;
        }
        let address = self.heap_mut().end_address;
        self.heap_mut().end_address += size as Address;
        Some(address)
    }

    /// Deallocate memory on the heap of a given size (number of bytes).
    /// This is done by writing zeros to the memory at the address.
    /// No need to adjust the range of the heap section.
    pub fn heap_deallocate(&mut self, address: Address, size: usize) {
        // Nuke the data at the address
        self.write(address, &(vec![0; size]));
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut offset = 0;
        for section in self.sections.values() {
            let data = section.read();
            if buf.len() < offset + data.len() {
                buf.resize(offset + data.len(), 0);
            }

            buf[offset..offset + data.len()].copy_from_slice(data);
            offset += data.len();
        }
        // Fill in text section with 0xFF
        let text_end = self.text.end_address as usize;
        let text_start = self.text.start_address as usize;
        let text_size = text_end - text_start + 1;
        if buf.len() < text_end + text_size {
            buf.resize(text_end + text_size, 0);
        }
        buf[text_end..text_end + text_size].fill(0xFF);
        buf
    }
}
