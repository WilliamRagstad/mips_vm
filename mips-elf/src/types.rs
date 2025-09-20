//! ELF type definitions

/// ELF section header
#[derive(Debug, Clone)]
pub struct SectionHeader {
    pub name: u32,
    pub section_type: u32,
    pub flags: u32,
    pub addr: u32,
    pub offset: u32,
    pub size: u32,
    pub link: u32,
    pub info: u32,
    pub addralign: u32,
    pub entsize: u32,
}

/// ELF program header
#[derive(Debug, Clone)]
pub struct ProgramHeader {
    pub segment_type: u32,
    pub offset: u32,
    pub vaddr: u32,
    pub paddr: u32,
    pub filesz: u32,
    pub memsz: u32,
    pub flags: u32,
    pub align: u32,
}
