//! DWARF writer using gimli::write

use gimli::write::{
    DwarfUnit, EndianVec, LineProgram, Sections, Writer, 
    FileId, DirectoryId, Address, LineString, AttributeValue,
    UnitEntryId, Attribute
};
use gimli::{Encoding, Format, RunTimeEndian, constants};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DwarfWriteError {
    #[error("DWARF generation failed: {0}")]
    Generation(#[from] gimli::write::Error),
    #[error("Invalid source file: {0}")]
    InvalidSourceFile(String),
}

/// DWARF debug information writer
pub struct DwarfWriter {
    encoding: Encoding,
    units: Vec<DwarfUnit>,
    line_programs: Vec<LineProgram>,
    file_map: HashMap<String, FileId>,
    dir_map: HashMap<String, DirectoryId>,
}

impl DwarfWriter {
    /// Create a new DWARF writer
    pub fn new() -> Self {
        let encoding = Encoding {
            format: Format::Dwarf32,
            version: 4,
            address_size: 4, // 32-bit MIPS
        };

        Self {
            encoding,
            units: Vec::new(),
            line_programs: Vec::new(),
            file_map: HashMap::new(),
            dir_map: HashMap::new(),
        }
    }

    /// Start a new compilation unit
    pub fn add_compilation_unit(&mut self, name: &str, comp_dir: &str) -> Result<usize, DwarfWriteError> {
        let mut unit = DwarfUnit::new(self.encoding);

        // Add compilation unit DIE
        let comp_unit_id = unit.root();
        
        // Set DW_TAG_compile_unit
        unit.get_mut(comp_unit_id).set(
            constants::DW_AT_name,
            AttributeValue::StringRef(LineString::new(name.as_bytes(), &self.encoding, 0)?),
        );
        
        unit.get_mut(comp_unit_id).set(
            constants::DW_AT_comp_dir,
            AttributeValue::StringRef(LineString::new(comp_dir.as_bytes(), &self.encoding, 0)?),
        );

        unit.get_mut(comp_unit_id).set(
            constants::DW_AT_producer,
            AttributeValue::StringRef(LineString::new(b"mips-asm", &self.encoding, 0)?),
        );

        unit.get_mut(comp_unit_id).set(
            constants::DW_AT_language,
            AttributeValue::Data1(constants::DW_LANG_Mips_Assembler as u8),
        );

        let unit_id = self.units.len();
        self.units.push(unit);

        // Create corresponding line program
        let mut line_program = LineProgram::new(
            self.encoding,
            constants::DW_LNS_copy,
            constants::DW_LNE_end_sequence,
            DirectoryId::new(0),
        );

        // Add current directory
        if !self.dir_map.contains_key(comp_dir) {
            let dir_id = line_program.add_directory(LineString::new(comp_dir.as_bytes(), &self.encoding, 0)?)?;
            self.dir_map.insert(comp_dir.to_string(), dir_id);
        }

        self.line_programs.push(line_program);

        Ok(unit_id)
    }

    /// Add a source file to the line program
    pub fn add_file(&mut self, unit_id: usize, filename: &str, directory: &str) -> Result<FileId, DwarfWriteError> {
        if unit_id >= self.line_programs.len() {
            return Err(DwarfWriteError::InvalidSourceFile("Invalid unit ID".to_string()));
        }

        let key = format!("{}/{}", directory, filename);
        if let Some(&file_id) = self.file_map.get(&key) {
            return Ok(file_id);
        }

        let line_program = &mut self.line_programs[unit_id];

        // Get or create directory
        let dir_id = if let Some(&dir_id) = self.dir_map.get(directory) {
            dir_id
        } else {
            let dir_id = line_program.add_directory(LineString::new(directory.as_bytes(), &self.encoding, 0)?)?;
            self.dir_map.insert(directory.to_string(), dir_id);
            dir_id
        };

        // Add file
        let file_id = line_program.add_file(
            LineString::new(filename.as_bytes(), &self.encoding, 0)?,
            dir_id,
            None, // timestamp
        )?;

        self.file_map.insert(key, file_id);
        Ok(file_id)
    }

    /// Add a line number entry
    pub fn add_line_entry(
        &mut self,
        unit_id: usize,
        address: u64,
        file_id: FileId,
        line: u32,
        column: u32,
    ) -> Result<(), DwarfWriteError> {
        if unit_id >= self.line_programs.len() {
            return Err(DwarfWriteError::InvalidSourceFile("Invalid unit ID".to_string()));
        }

        let line_program = &mut self.line_programs[unit_id];
        
        // Generate the line number program
        line_program.row().address_offset = address;
        line_program.row().file = file_id;
        line_program.row().line = line;
        line_program.row().column = column;
        line_program.generate_row()?;

        Ok(())
    }

    /// Add a function DIE
    pub fn add_function(
        &mut self,
        unit_id: usize,
        name: &str,
        low_pc: u64,
        high_pc: u64,
    ) -> Result<UnitEntryId, DwarfWriteError> {
        if unit_id >= self.units.len() {
            return Err(DwarfWriteError::InvalidSourceFile("Invalid unit ID".to_string()));
        }

        let unit = &mut self.units[unit_id];
        let root = unit.root();
        
        // Create function DIE as child of compilation unit
        let func_id = unit.add(root, constants::DW_TAG_subprogram);
        
        unit.get_mut(func_id).set(
            constants::DW_AT_name,
            AttributeValue::StringRef(LineString::new(name.as_bytes(), &self.encoding, 0)?),
        );
        
        unit.get_mut(func_id).set(
            constants::DW_AT_low_pc,
            AttributeValue::Address(Address::Constant(low_pc)),
        );
        
        unit.get_mut(func_id).set(
            constants::DW_AT_high_pc,
            AttributeValue::Udata(high_pc - low_pc),
        );

        Ok(func_id)
    }

    /// Generate all DWARF sections
    pub fn generate_sections(&mut self) -> Result<DwarfSections, DwarfWriteError> {
        let mut sections = Sections::default();
        let endian = RunTimeEndian::Little;

        // Write units and line programs
        for (unit, line_program) in self.units.iter_mut().zip(self.line_programs.iter_mut()) {
            let mut buf = EndianVec::new(endian);
            unit.write(&mut sections, &mut buf)?;
            
            // Write line program
            let mut line_buf = EndianVec::new(endian);
            line_program.write(&mut line_buf, &self.encoding, &sections.debug_str)?;
            sections.debug_line.extend(line_buf);
        }

        // Convert to our own format
        let mut debug_info = Vec::new();
        sections.debug_info.write(&mut debug_info, endian)?;

        let mut debug_abbrev = Vec::new();
        sections.debug_abbrev.write(&mut debug_abbrev, endian)?;

        let mut debug_str = Vec::new();
        sections.debug_str.write(&mut debug_str, endian)?;

        let mut debug_line = Vec::new();
        sections.debug_line.write(&mut debug_line, endian)?;

        Ok(DwarfSections {
            debug_info,
            debug_abbrev,
            debug_str,
            debug_line,
        })
    }
}

impl Default for DwarfWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Generated DWARF sections
#[derive(Debug, Clone)]
pub struct DwarfSections {
    pub debug_info: Vec<u8>,
    pub debug_abbrev: Vec<u8>,
    pub debug_str: Vec<u8>,
    pub debug_line: Vec<u8>,
}

impl DwarfSections {
    /// Get section data by name
    pub fn get_section(&self, name: &str) -> Option<&[u8]> {
        match name {
            ".debug_info" => Some(&self.debug_info),
            ".debug_abbrev" => Some(&self.debug_abbrev),
            ".debug_str" => Some(&self.debug_str),
            ".debug_line" => Some(&self.debug_line),
            _ => None,
        }
    }
}
