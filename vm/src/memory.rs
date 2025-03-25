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

#[derive(Debug, Clone, PartialEq, Default)]
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

/// The size of a page in bytes.
/// For MIPS32, the page size is 4KB (4096 bytes).
const PAGE_SIZE: usize = 4096; // 4KB

/// A page is a fixed-length contiguous block of virtual memory, described by a single entry in the page table.
/// It is the smallest unit of data for memory management in a virtual memory system.
#[derive(Debug)]
struct Page {
    data: [u8; PAGE_SIZE],
    protection: ProtectionLevel,
}

/// A page table is the data structure used by a virtual memory system in an operating system to store the mapping between virtual addresses and physical addresses.
/// Virtual addresses are used by the CPU, and physical addresses are used by the hardware.
/// The page table is used to translate virtual addresses to physical addresses.
/// The page table is stored in memory and is managed by the operating system.
#[derive(Debug, Default)]
struct PageTable {
    pages: HashMap<u32, Page>,
}

impl PageTable {
    /// Insert a page into the page table.
    pub fn insert_page(&mut self, page_number: u32, protection: ProtectionLevel) {
        self.pages.insert(
            page_number,
            Page {
                data: [0; PAGE_SIZE],
                protection,
            },
        );
    }

    pub fn ensure_pages(&mut self, start_page: u32, end_page: u32, protection: ProtectionLevel) {
        for page_number in start_page..=end_page {
            if !self.pages.contains_key(&page_number) {
                self.insert_page(page_number, protection.clone());
            }
        }
    }

    pub fn set_protection(&mut self, page_number: u32, protection: ProtectionLevel) {
        if let Some(page) = self.pages.get_mut(&page_number) {
            page.protection = protection;
        }
    }

    pub fn set_protections(&mut self, start_page: u32, end_page: u32, protection: ProtectionLevel) {
        for page_number in start_page..=end_page {
            self.set_protection(page_number, protection.clone());
        }
    }

    /// Get a mutable reference to the page for a given page number.
    pub fn get_page_mut(&mut self, page_number: u32) -> Option<&mut Page> {
        self.pages.get_mut(&page_number)
    }

    /// Get an immutable reference to the page for a given page number.
    pub fn get_page(&self, page_number: u32) -> Option<&Page> {
        self.pages.get(&page_number)
    }

    /// Write data to one or more pages in the page table.
    /// Throw an error if the page is not writable or if the page is not found.
    pub fn write_bytes(&mut self, address: Address, bytes: &[u8]) -> Result<()> {
        let mut page_number = address.page_number();
        let mut offset = address.page_offset();
        let mut left = bytes.len();
        while left > 0 {
            let page = self
                .get_page_mut(page_number)
                .ok_or(MemoryError::SegmentFault)?;
            log::debug!(
                "Writing to page: {:?} ({:?}) at offset: {}",
                page_number,
                page.protection,
                offset
            );
            if !page.protection.is_writable() {
                return Err(MemoryError::ProtectionFault);
            }
            let page_offset = offset as usize;
            let write_size = left.min(PAGE_SIZE - page_offset);
            page.data[page_offset..(page_offset + write_size)]
                .copy_from_slice(&bytes[..write_size]);
            left -= write_size;
            offset = 0;
            page_number += 1;
        }
        Ok(())
    }

    /// Read data from one or more pages in the page table.
    /// Throw an error if the page is not readable or if the page is not found.
    /// Return the data read from the page.
    ///
    /// # Returns
    /// A vector of slices of the data read from the page.
    pub fn read_bytes(&self, address: Address, size: usize) -> Result<Vec<&[u8]>> {
        let page_number = address.page_number();
        let offset = address.page_offset();
        let mut data = Vec::new();
        let mut left = size;
        while left > 0 {
            let page = self
                .get_page(page_number)
                .ok_or(MemoryError::SegmentFault)?;
            if !page.protection.is_readable() {
                return Err(MemoryError::ProtectionFault);
            }
            let page_offset = offset as usize;
            let read_size = left.min(PAGE_SIZE - page_offset);
            data.push(&page.data[page_offset..(page_offset + read_size)]);
            left -= read_size;
        }
        Ok(data)
    }
}

type ReadHandler<T> = fn(Address) -> T;
type WriteHandler<T> = fn(Address, T);

/// Memory paging is a memory management scheme that eliminates the need for
/// contiguous allocation of physical memory.
#[derive(Debug, Default)]
pub struct MemorySegment<T> {
    #[allow(dead_code)]
    name: String,
    pub start_address: Address,
    pub end_address: Address,
    read_handler: Option<ReadHandler<T>>,
    write_handler: Option<WriteHandler<T>>,
}

#[derive(Debug)]
pub struct Memory {
    page_table: PageTable,
    /// Labels with their names as the key and their address as the value.
    /// This is used to store the mapping of all address of labels in the original program.
    labels: HashMap<String, Address>,
    /// Sections of memory with their start address as the key.
    sections: HashMap<Address, MemorySegment<u8>>,
    /// Text section: contains the program's instructions
    /// This section is read-only and executable (code).
    text_segment: MemorySegment<Instruction>,
    text_instructions: Vec<Instruction>,
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
        let mut page_table = PageTable::default();
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

        let text_segment: MemorySegment<Instruction> = MemorySegment {
            name: ".text".to_string(),
            start_address: text_start_address,
            end_address: text_end_address,
            read_handler: None,
            write_handler: None,
        };
        page_table.ensure_pages(
            text_segment.start_address.page_number(),
            text_segment.end_address.page_number(),
            ProtectionLevel::Write,
        );
        page_table
            .write_bytes(
                text_segment.start_address,
                &text_instructions
                    .iter()
                    .flat_map(|_| 0xC0DEu32.to_le_bytes())
                    .collect::<Vec<u8>>(),
            )
            .unwrap();
        page_table.set_protections(
            text_segment.start_address.page_number(),
            text_segment.end_address.page_number(),
            ProtectionLevel::ReadExecute,
        );

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
            let data_end_address = address;
            assert!(data_raw_initialized.len() == (data_end_address - data_start_address) as usize);

            let data = MemorySegment {
                name: ".data".to_string(),
                start_address: data_start_address,
                end_address: data_end_address,
                read_handler: None,
                write_handler: None,
            };

            page_table.ensure_pages(
                data.start_address.page_number(),
                data.end_address.page_number(),
                ProtectionLevel::ReadWrite,
            );
            page_table
                .write_bytes(data.start_address, &data_raw_initialized)
                .unwrap();

            sections.insert(data_start_address, data);
            Some(data_start_address)
        } else {
            None
        };

        let heap = MemorySegment {
            name: ".heap".to_string(),
            start_address: 0x10000000.into(),
            end_address: 0x10000000.into(),
            read_handler: None,
            write_handler: None,
        };
        page_table.ensure_pages(
            heap.start_address.page_number(),
            heap.end_address.page_number(),
            ProtectionLevel::ReadWrite,
        );
        let heap_address = heap.start_address;
        sections.insert(heap.start_address, heap);

        let stack = MemorySegment {
            name: ".stack".to_string(),
            start_address: 0x7fffefff.into(),
            end_address: 0x7fffefff.into(),
            read_handler: None,
            write_handler: None,
        };
        page_table.ensure_pages(
            stack.start_address.page_number(),
            stack.end_address.page_number(),
            ProtectionLevel::ReadWrite,
        );
        let stack_address = stack.start_address;
        sections.insert(stack.start_address, stack);

        Memory {
            page_table,
            labels,
            sections,
            text_segment,
            text_instructions,
            data,
            heap: heap_address,
            stack: stack_address,
        }
    }

    pub fn add_section(&mut self, section: MemorySegment<u8>) {
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

    pub fn find_section(&self, address: Address) -> Result<&MemorySegment<u8>> {
        self.sections
            .values()
            .find(|section| section.start_address <= address && address <= section.end_address)
            .ok_or(MemoryError::InvalidSection)
    }

    /// Read from a memory-mapped I/O address location.
    /// This is used to read from a memory-mapped I/O device.
    ///
    /// Write the result into the memory address location at `self.data`.
    fn mmio_try_read_to(
        &mut self,
        read_handler: Option<ReadHandler<u8>>,
        address: Address,
        size: usize,
    ) -> Result<Option<Vec<u8>>> {
        if let Some(read_handler) = read_handler {
            let mut bytes = vec![0; size]; // Pre-allocation
            for i in 0..size {
                bytes[i] = read_handler(address + i);
            }
            self.page_table.write_bytes(address, &bytes)?;
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }

    /// Write to an memory-mapped I/O address location.
    /// This is used to write to a memory-mapped I/O device.
    fn mmio_try_write_to(
        &mut self,
        write_handler: Option<WriteHandler<u8>>,
        address: Address,
        bytes: &[u8],
    ) -> Result<()> {
        if let Some(write_handler) = write_handler {
            (0..bytes.len()).for_each(|i| {
                write_handler(address + i, bytes[i]);
            });
        }
        Ok(())
    }

    /// Read from a memory address location and return the data of the specified size
    pub fn read(&mut self, address: Address, size: usize) -> Result<Vec<u8>> {
        let section = self.find_section(address)?;
        if address + size > section.end_address {
            return Err(MemoryError::OutOfBounds); // Out of bounds
        }
        if let Some(data) = self.mmio_try_read_to(section.read_handler, address, size)? {
            return Ok(data);
        } else {
            Ok(self
                .page_table
                .read_bytes(address, size)?
                .into_iter()
                .flatten()
                .cloned()
                .collect::<Vec<u8>>())
        }
    }

    pub fn read_buf(&mut self, address: Address, buf: &mut [u8]) -> Result<()> {
        let size = buf.len();
        let src = self.read(address, size)?;
        buf[..size].copy_from_slice(&src);
        Ok(())
    }

    pub fn read_const<const N: usize>(&mut self, address: Address) -> Result<[u8; N]> {
        let mut data = [0; N];
        self.read_buf(address, &mut data)?;
        Ok(data)
    }

    pub fn read_max(&mut self, address: Address, max_size: usize) -> Result<Vec<u8>> {
        let section = self.find_section(address)?;
        let size = max_size.min((section.end_address - address) as usize);
        self.read(address, size)
    }

    pub fn read_buf_max(&mut self, address: Address, buf: &mut [u8]) -> Result<usize> {
        let section = self.find_section(address)?;
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
    pub fn write(&mut self, address: Address, bytes: &[u8]) -> Result<()> {
        let section = self.find_section(address)?;
        if address + bytes.len() > section.end_address {
            return Err(MemoryError::OutOfBounds); // Out of bounds
        }
        self.mmio_try_write_to(section.write_handler, address, bytes)?;
        self.page_table.write_bytes(address, bytes)
    }

    /// Currently, only the text section will be executable
    pub fn execute(&self, address: Address) -> Result<&Instruction> {
        if self.text_segment.start_address <= address && address <= self.text_segment.end_address {
            // TODO: Improve finding the instruction by address performance
            let index = (address - self.text_segment.start_address) as usize / Instruction::size();
            let Some(page) = self.page_table.get_page(address.page_number()) else {
                return Err(MemoryError::SegmentFault);
            };
            if page.protection.is_executable() {
                self.text_instructions
                    .get(index)
                    .ok_or(MemoryError::InvalidInstruction)
            } else {
                Err(MemoryError::ProtectionFault)
            }
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

    pub fn text(&self) -> &MemorySegment<Instruction> {
        &self.text_segment
    }

    pub fn data(&self) -> Option<&MemorySegment<u8>> {
        self.sections.get(&self.data?)
    }

    pub fn data_mut(&mut self) -> Option<&mut MemorySegment<u8>> {
        self.sections.get_mut(&self.data?)
    }

    pub fn heap(&self) -> &MemorySegment<u8> {
        self.sections.get(&self.heap).unwrap()
    }

    pub fn heap_mut(&mut self) -> &mut MemorySegment<u8> {
        self.sections.get_mut(&self.heap).unwrap()
    }

    pub fn stack(&self) -> &MemorySegment<u8> {
        self.sections.get(&self.stack).unwrap()
    }

    pub fn stack_mut(&mut self) -> &mut MemorySegment<u8> {
        self.sections.get_mut(&self.stack).unwrap()
    }

    /// Push a byte to the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address -= 1` to adjust the range of the stack section.
    ///
    /// Returns:
    /// - `Ok(())` if the push is successful.
    pub fn stack_push(&mut self, values: &[u8]) -> Result<()> {
        // Check if the stack section is colliding with the heap section
        if self.stack().start_address - 1 <= self.heap().end_address {
            return Err(MemoryError::InvalidStack);
        }
        let stack = self.stack_mut();
        let stack_new_start = stack.start_address - values.len() as u32;
        stack.start_address = stack_new_start;
        self.page_table.write_bytes(stack_new_start, values)
    }

    /// Pop a byte from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += 1` to adjust the range of the stack section.
    pub fn stack_pop(&mut self, size: usize) -> Result<Vec<u8>> {
        let stack = self.stack_mut();
        let stack_new_start = stack.start_address + size as u32;
        stack.start_address = stack_new_start;
        self.read(stack_new_start, size)
    }

    /// Push a word to the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address -= WORD_SIZE` to adjust the range of the stack section.
    ///
    /// Returns:
    /// - `Ok(())` if the push is successful.
    /// - `Err` if the stack section is colliding with the heap section.
    pub fn stack_push_word(&mut self, value: Word) -> Result<()> {
        self.stack_push(&value.to_le_bytes())
    }

    /// Pop a word from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += WORD_SIZE` to adjust the range of the stack section.
    pub fn stack_pop_word(&mut self) -> Result<Word> {
        const WORD_SIZE: usize = size_of::<Word>();
        let bytes: [u8; WORD_SIZE] = self
            .stack_pop(WORD_SIZE)?
            .try_into()
            .map_err(|_| MemoryError::InvalidSize)?;
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
        self.stack_push(&value.to_le_bytes())
    }

    /// Pop an address from the stack.
    /// **The stack grows downwards** (from high address to lower addresses),
    /// so the `start_address += ADDRESS_SIZE` to adjust the range of the stack section.
    pub fn stack_pop_address(&mut self) -> Result<Address> {
        const ADDRESS_SIZE: usize = size_of::<Address>();
        let bytes: [u8; ADDRESS_SIZE] = self
            .stack_pop(ADDRESS_SIZE)?
            .try_into()
            .map_err(|_| MemoryError::InvalidSize)?;
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
        // Iterate through all allocated memory pages
        for (page_number, page) in &self.page_table.pages {
            let start = Address::from_page_number(*page_number).unwrap() as usize;
            let end = start + PAGE_SIZE;
            if buf.len() < end {
                buf.resize(end, 0);
            }
            buf[start..end].copy_from_slice(&page.data);
        }
        // Fill in text section with 0xFF
        let text_end = self.text_segment.end_address.unwrap() as usize;
        let text_start = self.text_segment.start_address.unwrap() as usize;
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
