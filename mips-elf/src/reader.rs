//! ELF file reader using the object crate

use object::{Object, ObjectSection, ObjectSymbol, SectionKind, SymbolKind};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ElfReadError {
    #[error("Failed to parse object file: {0}")]
    ParseError(#[from] object::Error),
    #[error("Invalid file format")]
    InvalidFormat,
    #[error("Section not found: {0}")]
    SectionNotFound(String),
}

/// ELF file reader using the object crate
pub struct ElfReader<'data> {
    object: object::File<'data>,
}

impl<'data> ElfReader<'data> {
    /// Create a new ELF reader from binary data
    pub fn new(data: &'data [u8]) -> Result<Self, ElfReadError> {
        let object = object::File::parse(data)?;

        // Verify it's a MIPS ELF file
        if object.architecture() != object::Architecture::Mips {
            return Err(ElfReadError::InvalidFormat);
        }

        Ok(Self { object })
    }

    /// Get the entry point address
    pub fn entry_point(&self) -> u64 {
        self.object.entry()
    }

    /// Get the file format
    pub fn is_executable(&self) -> bool {
        matches!(self.object.kind(), object::FileKind::Executable)
    }

    /// Get the file format
    pub fn is_relocatable(&self) -> bool {
        matches!(self.object.kind(), object::FileKind::Relocatable)
    }

    /// Get section by name
    pub fn get_section(&self, name: &str) -> Result<ElfSection<'data>, ElfReadError> {
        for section in self.object.sections() {
            if let Ok(section_name) = section.name() {
                if section_name == name {
                    return Ok(ElfSection { section });
                }
            }
        }
        Err(ElfReadError::SectionNotFound(name.to_string()))
    }

    /// Get all sections
    pub fn sections(&self) -> impl Iterator<Item = ElfSection<'data>> {
        self.object.sections().map(|section| ElfSection { section })
    }

    /// Get all symbols
    pub fn symbols(&self) -> impl Iterator<Item = ElfSymbol<'data>> {
        self.object.symbols().map(|symbol| ElfSymbol { symbol })
    }

    /// Find symbol by name
    pub fn find_symbol(&self, name: &str) -> Option<ElfSymbol<'data>> {
        self.object
            .symbols()
            .find(|sym| sym.name().map_or(false, |n| n == name))
            .map(|symbol| ElfSymbol { symbol })
    }
}

/// Wrapper for object crate section
pub struct ElfSection<'data> {
    section: object::Section<'data, 'data>,
}

impl<'data> ElfSection<'data> {
    /// Get section name
    pub fn name(&self) -> Result<&str, ElfReadError> {
        self.section.name().map_err(ElfReadError::ParseError)
    }

    /// Get section data
    pub fn data(&self) -> Result<&'data [u8], ElfReadError> {
        self.section.data().map_err(ElfReadError::ParseError)
    }

    /// Get section size
    pub fn size(&self) -> u64 {
        self.section.size()
    }

    /// Get section address
    pub fn address(&self) -> u64 {
        self.section.address()
    }

    /// Check if section is executable
    pub fn is_text(&self) -> bool {
        self.section.kind() == SectionKind::Text
    }

    /// Check if section contains data
    pub fn is_data(&self) -> bool {
        matches!(
            self.section.kind(),
            SectionKind::Data | SectionKind::ReadOnlyData
        )
    }
}

impl fmt::Debug for ElfSection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElfSection")
            .field("name", &self.name().unwrap_or("<invalid>"))
            .field("size", &self.size())
            .field("address", &format_args!("0x{:x}", self.address()))
            .finish()
    }
}

/// Wrapper for object crate symbol
pub struct ElfSymbol<'data> {
    symbol: object::Symbol<'data, 'data>,
}

impl<'data> ElfSymbol<'data> {
    /// Get symbol name
    pub fn name(&self) -> Result<&str, ElfReadError> {
        self.symbol.name().map_err(ElfReadError::ParseError)
    }

    /// Get symbol address
    pub fn address(&self) -> u64 {
        self.symbol.address()
    }

    /// Get symbol size
    pub fn size(&self) -> u64 {
        self.symbol.size()
    }

    /// Check if symbol is a function
    pub fn is_function(&self) -> bool {
        self.symbol.kind() == SymbolKind::Text
    }

    /// Check if symbol is global
    pub fn is_global(&self) -> bool {
        self.symbol.is_global()
    }
}

impl fmt::Debug for ElfSymbol<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElfSymbol")
            .field("name", &self.name().unwrap_or("<invalid>"))
            .field("address", &format_args!("0x{:x}", self.address()))
            .field("size", &self.size())
            .finish()
    }
}
