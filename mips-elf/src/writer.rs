//! ELF file writer

use crate::{ElfType, EM_MIPS};
use std::io::{self, Write};

/// ELF file writer
pub struct ElfWriter<W: Write> {
    writer: W,
}

impl<W: Write> ElfWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write ELF header
    pub fn write_header(&mut self, elf_type: ElfType, entry_point: u32) -> io::Result<()> {
        // ELF magic number
        self.writer.write_all(b"\x7fELF")?;

        // EI_CLASS (32-bit)
        self.writer.write_all(&[1])?;

        // EI_DATA (little-endian for now)
        self.writer.write_all(&[1])?;

        // EI_VERSION (current)
        self.writer.write_all(&[1])?;

        // EI_PAD (padding)
        self.writer.write_all(&[0; 9])?;

        // e_type
        self.writer.write_all(&(elf_type as u16).to_le_bytes())?;

        // e_machine
        self.writer.write_all(&EM_MIPS.to_le_bytes())?;

        // e_version
        self.writer.write_all(&1u32.to_le_bytes())?;

        // e_entry
        self.writer.write_all(&entry_point.to_le_bytes())?;

        // TODO: Add remaining header fields

        Ok(())
    }
}
