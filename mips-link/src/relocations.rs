//! MIPS relocation handling

/// MIPS relocation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationType {
    None = 0,
    Hi16 = 5,      // R_MIPS_HI16
    Lo16 = 6,      // R_MIPS_LO16
    Pc16 = 10,     // R_MIPS_PC16
    Call16 = 11,   // R_MIPS_CALL16
    GpRel32 = 12,  // R_MIPS_GPREL32
}

/// Relocation entry
#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: u32,
    pub symbol: u32,
    pub reloc_type: RelocationType,
    pub addend: i32,
}

/// Relocation table
#[derive(Debug, Clone)]
pub struct RelocationTable {
    pub relocations: Vec<Relocation>,
}

impl RelocationTable {
    pub fn new() -> Self {
        Self {
            relocations: Vec::new(),
        }
    }

    pub fn add_relocation(&mut self, relocation: Relocation) {
        self.relocations.push(relocation);
    }

    /// Apply relocations to a section
    pub fn apply_to_section(
        &self,
        section_data: &mut [u8],
        base_address: u32,
    ) -> Result<(), RelocationError> {
        for reloc in &self.relocations {
            self.apply_relocation(reloc, section_data, base_address)?;
        }
        Ok(())
    }

    fn apply_relocation(
        &self,
        reloc: &Relocation,
        section_data: &mut [u8],
        _base_address: u32,
    ) -> Result<(), RelocationError> {
        let offset = reloc.offset as usize;
        
        if offset + 4 > section_data.len() {
            return Err(RelocationError::InvalidOffset(reloc.offset));
        }

        match reloc.reloc_type {
            RelocationType::Hi16 => {
                // TODO: Apply HI16 relocation
            }
            RelocationType::Lo16 => {
                // TODO: Apply LO16 relocation
            }
            RelocationType::Pc16 => {
                // TODO: Apply PC16 relocation
            }
            _ => {
                return Err(RelocationError::UnsupportedType(reloc.reloc_type));
            }
        }

        Ok(())
    }
}

impl Default for RelocationTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RelocationError {
    #[error("Invalid relocation offset: {0}")]
    InvalidOffset(u32),
    #[error("Unsupported relocation type: {0:?}")]
    UnsupportedType(RelocationType),
}
