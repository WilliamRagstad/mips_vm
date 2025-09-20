//! Output generation for assembled programs

use crate::{AssembledProgram, AssemblerConfig};
use mips_elf::{ElfWriter, ElfType};
use std::io::{self, Write};

/// Generate ELF output from assembled program
pub fn generate_elf<W: Write>(
    writer: W,
    program: &AssembledProgram,
    config: &AssemblerConfig,
) -> io::Result<()> {
    let mut elf_writer = ElfWriter::new(writer);
    
    // Write ELF header
    elf_writer.write_header(config.output_type, program.entry_point)?;
    
    // TODO: Write sections, program headers, etc.
    
    Ok(())
}

/// Generate relocatable object file
pub fn generate_object<W: Write>(
    writer: W,
    program: &AssembledProgram,
) -> io::Result<()> {
    generate_elf(writer, program, &AssemblerConfig {
        output_type: ElfType::Relocatable,
        ..Default::default()
    })
}

/// Generate executable file
pub fn generate_executable<W: Write>(
    writer: W,
    program: &AssembledProgram,
) -> io::Result<()> {
    generate_elf(writer, program, &AssemblerConfig {
        output_type: ElfType::Executable,
        ..Default::default()
    })
}
