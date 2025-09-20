//! ELF file writer using the object crate

use crate::ElfType;
use object::{
    write::{Object, Symbol, SymbolSection},
    Architecture, BinaryFormat, Endianness, SymbolFlags, SymbolKind, SymbolScope,
};
use std::io::{self, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ElfWriteError {
    #[error("Object creation failed: {0}")]
    ObjectCreation(String),
    #[error("Section creation failed: {0}")]
    SectionCreation(String),
    #[error("Symbol creation failed: {0}")]
    SymbolCreation(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// ELF file writer using the object crate
pub struct ElfWriter {
    object: Object<'static>,
}

impl ElfWriter {
    /// Create a new ELF writer
    pub fn new(elf_type: ElfType) -> Result<Self, ElfWriteError> {
        let obj_type = match elf_type {
            ElfType::Relocatable => object::FileKind::Relocatable,
            ElfType::Executable => object::FileKind::Executable,
            ElfType::SharedObject => object::FileKind::Dynamic,
            ElfType::Core => object::FileKind::Core,
            ElfType::None => object::FileKind::Relocatable, // Default fallback
        };

        let mut object = Object::new(
            BinaryFormat::Elf,
            Architecture::Mips,
            Endianness::Little, // Can be configured later
        );

        object.flags = match elf_type {
            ElfType::Executable => 0,
            ElfType::Relocatable => 1,
            ElfType::SharedObject => 3,
            ElfType::Core => 4,
            ElfType::None => 0,
        };

        Ok(Self { object })
    }

    /// Set the entry point for executable files
    pub fn set_entry_point(&mut self, entry: u64) {
        self.object.entry = entry;
    }

    /// Add a text section with code
    pub fn add_text_section(&mut self, data: &[u8]) -> Result<(), ElfWriteError> {
        let section_id = self
            .object
            .add_section(vec![], b".text".to_vec(), object::SectionKind::Text)
            .map_err(|e| ElfWriteError::SectionCreation(e.to_string()))?;

        self.object
            .set_section_data(section_id, data.to_vec(), 4)
            .map_err(|e| ElfWriteError::SectionCreation(e.to_string()))?;

        Ok(())
    }

    /// Add a data section
    pub fn add_data_section(&mut self, data: &[u8]) -> Result<(), ElfWriteError> {
        let section_id = self
            .object
            .add_section(vec![], b".data".to_vec(), object::SectionKind::Data)
            .map_err(|e| ElfWriteError::SectionCreation(e.to_string()))?;

        self.object
            .set_section_data(section_id, data.to_vec(), 4)
            .map_err(|e| ElfWriteError::SectionCreation(e.to_string()))?;

        Ok(())
    }

    /// Add a symbol to the symbol table
    pub fn add_symbol(
        &mut self,
        name: &[u8],
        value: u64,
        size: u64,
        kind: SymbolKind,
        scope: SymbolScope,
        section: Option<object::write::SectionId>,
    ) -> Result<(), ElfWriteError> {
        let symbol = Symbol {
            name: name.to_vec(),
            value,
            size,
            kind,
            scope,
            weak: false,
            section: section
                .map(SymbolSection::Section)
                .unwrap_or(SymbolSection::Undefined),
            flags: SymbolFlags::None,
        };

        self.object
            .add_symbol(symbol)
            .map_err(|e| ElfWriteError::SymbolCreation(e.to_string()))?;

        Ok(())
    }

    /// Write the ELF file to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), ElfWriteError> {
        let data = self
            .object
            .write()
            .map_err(|e| ElfWriteError::ObjectCreation(e.to_string()))?;

        writer.write_all(&data)?;
        Ok(())
    }

    /// Get the object for advanced manipulation
    pub fn object_mut(&mut self) -> &mut Object<'static> {
        &mut self.object
    }
}
