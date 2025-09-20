//! ELF32 builder for MIPS following the blueprint design

use crate::{ElfType, EM_MIPS};
use byteorder::{BigEndian, ByteOrder, LittleEndian, WriteBytesExt};
use mips_util::MipsResult;
use std::collections::HashMap;
use std::io::{Cursor, Write};

/// Endianness support for MIPS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Big,
    Little,
}

/// Section types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecType {
    Null = 0,
    Progbits = 1,
    Symtab = 2,
    Strtab = 3,
    Rela = 4,
    Hash = 5,
    Dynamic = 6,
    Note = 7,
    Nobits = 8,
    Rel = 9,
}

/// Section flags
#[derive(Debug, Clone, Copy)]
pub struct SecFlags(pub u32);

impl SecFlags {
    pub const WRITE: SecFlags = SecFlags(0x1);
    pub const ALLOC: SecFlags = SecFlags(0x2);
    pub const EXECINSTR: SecFlags = SecFlags(0x4);
}

/// Symbol binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bind {
    Local = 0,
    Global = 1,
    Weak = 2,
}

/// Symbol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymType {
    Notype = 0,
    Object = 1,
    Func = 2,
    Section = 3,
    File = 4,
}

/// Relocation type for MIPS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocType {
    None = 0,
    Mips32 = 2,   // R_MIPS_32
    Mips26 = 4,   // R_MIPS_26
    MipsHi16 = 5, // R_MIPS_HI16
    MipsLo16 = 6, // R_MIPS_LO16
}

/// Section ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecId(pub usize);

/// Symbol ID  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymId(pub usize);

/// Section data
#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub sec_type: SecType,
    pub flags: SecFlags,
    pub data: Vec<u8>,
    pub addr: u32,
    pub align: u32,
    pub entsize: u32,
    pub link: u32,
    pub info: u32,
}

/// Symbol table entry
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub value: u32,
    pub size: u32,
    pub bind: Bind,
    pub sym_type: SymType,
    pub section: Option<SecId>,
}

/// Relocation entry
#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: u32,
    pub symbol: SymId,
    pub reloc_type: RelocType,
    pub addend: i32,
}

/// ELF32 builder for MIPS
pub struct ElfBuilder32 {
    pub endian: Endianness,
    sections: Vec<Section>,
    symbols: Vec<Symbol>,
    relocations: HashMap<SecId, Vec<Relocation>>,
    string_table: Vec<u8>,
    shstring_table: Vec<u8>,
    entry_point: u32,
    elf_type: ElfType,
}

impl ElfBuilder32 {
    /// Create a new ELF32 builder
    pub fn new(endian: Endianness) -> Self {
        let mut builder = Self {
            endian,
            sections: Vec::new(),
            symbols: Vec::new(),
            relocations: HashMap::new(),
            string_table: vec![0],   // First byte is always null
            shstring_table: vec![0], // First byte is always null
            entry_point: 0,
            elf_type: ElfType::Executable,
        };

        // Add null section (index 0)
        builder.sections.push(Section {
            name: String::new(),
            sec_type: SecType::Null,
            flags: SecFlags(0),
            data: Vec::new(),
            addr: 0,
            align: 0,
            entsize: 0,
            link: 0,
            info: 0,
        });

        builder
    }

    /// Set the ELF type
    pub fn set_type(&mut self, elf_type: ElfType) {
        self.elf_type = elf_type;
    }

    /// Set entry point
    pub fn set_entry(&mut self, entry: u32) {
        self.entry_point = entry;
    }

    /// Add a section and return its ID
    pub fn add_section(&mut self, name: &str, sec_type: SecType, flags: SecFlags) -> SecId {
        let _name_offset = self.add_to_shstring_table(name);
        let section = Section {
            name: name.to_string(),
            sec_type,
            flags,
            data: Vec::new(),
            addr: 0,
            align: 1,
            entsize: 0,
            link: 0,
            info: 0,
        };

        self.sections.push(section);
        SecId(self.sections.len() - 1)
    }

    /// Convenience method to add text section
    pub fn add_text(&mut self, name: &str) -> SecId {
        self.add_section(
            name,
            SecType::Progbits,
            SecFlags(SecFlags::ALLOC.0 | SecFlags::EXECINSTR.0),
        )
    }

    /// Set section data
    pub fn set_section_data(&mut self, sec_id: SecId, data: Vec<u8>) {
        if let Some(section) = self.sections.get_mut(sec_id.0) {
            section.data = data;
        }
    }

    /// Set section address
    pub fn set_section_addr(&mut self, sec_id: SecId, addr: u32) {
        if let Some(section) = self.sections.get_mut(sec_id.0) {
            section.addr = addr;
        }
    }

    /// Set section alignment
    pub fn set_section_align(&mut self, sec_id: SecId, align: u32) {
        if let Some(section) = self.sections.get_mut(sec_id.0) {
            section.align = align;
        }
    }

    /// Add a symbol and return its ID
    pub fn add_symbol(
        &mut self,
        name: &str,
        section: Option<SecId>,
        value: u32,
        size: u32,
        bind: Bind,
        sym_type: SymType,
    ) -> SymId {
        let symbol = Symbol {
            name: name.to_string(),
            value,
            size,
            bind,
            sym_type,
            section,
        };

        self.symbols.push(symbol);
        SymId(self.symbols.len() - 1)
    }

    /// Add a relocation
    pub fn add_reloc(
        &mut self,
        sec_id: SecId,
        offset: u32,
        symbol: SymId,
        reloc_type: RelocType,
        addend: i32,
    ) {
        let reloc = Relocation {
            offset,
            symbol,
            reloc_type,
            addend,
        };

        self.relocations
            .entry(sec_id)
            .or_insert_with(Vec::new)
            .push(reloc);
    }

    /// Write the ELF file
    pub fn write<W: Write>(&self, mut writer: W) -> MipsResult<()> {
        let mut buffer = Cursor::new(Vec::new());

        // Write ELF header
        self.write_elf_header(&mut buffer)?;

        // Calculate section header offset
        let _section_header_offset = buffer.position() as u32;

        // Write section data
        for section in self.sections.iter().skip(1) {
            // Align to section alignment
            while buffer.position() % section.align as u64 != 0 {
                buffer.write_u8(0)?;
            }

            // Write section data
            buffer.write_all(&section.data)?;
        }

        // Write section headers (simplified for now)
        self.write_section_headers(&mut buffer)?;

        // Write final data
        writer.write_all(buffer.get_ref())?;
        Ok(())
    }

    fn write_elf_header<W: Write>(&self, writer: &mut W) -> MipsResult<()> {
        // ELF magic
        writer.write_all(b"\x7fELF")?;

        // EI_CLASS (32-bit)
        writer.write_u8(1)?;

        // EI_DATA (endianness)
        writer.write_u8(match self.endian {
            Endianness::Little => 1,
            Endianness::Big => 2,
        })?;

        // EI_VERSION (current)
        writer.write_u8(1)?;

        // EI_OSABI (SYSV)
        writer.write_u8(0)?;

        // EI_ABIVERSION
        writer.write_u8(0)?;

        // EI_PAD (7 bytes)
        writer.write_all(&[0; 7])?;

        // e_type
        self.write_u16(writer, self.elf_type as u16)?;

        // e_machine (MIPS)
        self.write_u16(writer, EM_MIPS)?;

        // e_version
        self.write_u32(writer, 1)?;

        // e_entry
        self.write_u32(writer, self.entry_point)?;

        // e_phoff (program header offset - 0 for relocatable)
        self.write_u32(writer, 0)?;

        // e_shoff (section header offset - calculate later)
        self.write_u32(writer, 0)?; // Will be fixed up

        // e_flags (MIPS flags)
        self.write_u32(writer, 0x20000000)?; // Basic MIPS32 flags

        // e_ehsize (ELF header size)
        self.write_u16(writer, 52)?;

        // e_phentsize (program header size)
        self.write_u16(writer, 32)?;

        // e_phnum (program header count)
        self.write_u16(writer, 0)?;

        // e_shentsize (section header size)
        self.write_u16(writer, 40)?;

        // e_shnum (section header count)
        self.write_u16(writer, self.sections.len() as u16)?;

        // e_shstrndx (section header string table index)
        self.write_u16(writer, (self.sections.len() - 1) as u16)?; // Assume last section is shstrtab

        Ok(())
    }

    fn write_section_headers<W: Write>(&self, writer: &mut W) -> MipsResult<()> {
        for section in &self.sections {
            // sh_name (offset in shstrtab)
            let name_offset = self.find_string_in_shstrtab(&section.name);
            self.write_u32(writer, name_offset)?;

            // sh_type
            self.write_u32(writer, section.sec_type as u32)?;

            // sh_flags
            self.write_u32(writer, section.flags.0)?;

            // sh_addr
            self.write_u32(writer, section.addr)?;

            // sh_offset (would need proper tracking)
            self.write_u32(writer, 0)?;

            // sh_size
            self.write_u32(writer, section.data.len() as u32)?;

            // sh_link
            self.write_u32(writer, section.link)?;

            // sh_info
            self.write_u32(writer, section.info)?;

            // sh_addralign
            self.write_u32(writer, section.align)?;

            // sh_entsize
            self.write_u32(writer, section.entsize)?;
        }
        Ok(())
    }

    fn write_u16<W: Write>(&self, writer: &mut W, value: u16) -> MipsResult<()> {
        match self.endian {
            Endianness::Little => writer.write_u16::<LittleEndian>(value)?,
            Endianness::Big => writer.write_u16::<BigEndian>(value)?,
        }
        Ok(())
    }

    fn write_u32<W: Write>(&self, writer: &mut W, value: u32) -> MipsResult<()> {
        match self.endian {
            Endianness::Little => writer.write_u32::<LittleEndian>(value)?,
            Endianness::Big => writer.write_u32::<BigEndian>(value)?,
        }
        Ok(())
    }

    fn add_to_shstring_table(&mut self, s: &str) -> u32 {
        let offset = self.shstring_table.len() as u32;
        self.shstring_table.extend_from_slice(s.as_bytes());
        self.shstring_table.push(0);
        offset
    }

    fn find_string_in_shstrtab(&self, s: &str) -> u32 {
        // This is a simple implementation - in practice you'd want to track offsets
        let search = format!("{}\0", s);
        let search_bytes = search.as_bytes();

        for i in 0..=self.shstring_table.len().saturating_sub(search_bytes.len()) {
            if self.shstring_table[i..].starts_with(search_bytes) {
                return i as u32;
            }
        }
        0 // Not found, return null string
    }
}
