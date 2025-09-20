//! ELF file reader

use std::io::{self, Read};

/// ELF file reader
pub struct ElfReader<R: Read> {
    reader: R,
}

impl<R: Read> ElfReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read and validate ELF header
    pub fn read_header(&mut self) -> io::Result<ElfHeader> {
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;

        if &magic != b"\x7fELF" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid ELF magic number",
            ));
        }

        // TODO: Read remaining header fields

        Ok(ElfHeader {
            magic,
            // TODO: Fill in other fields
        })
    }
}

/// ELF header structure
#[derive(Debug)]
pub struct ElfHeader {
    pub magic: [u8; 4],
    // TODO: Add remaining header fields
}
