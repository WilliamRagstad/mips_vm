//! Minimal DWARF v4 writer for debug information

pub mod abbrev;
pub mod info;
pub mod line;
pub mod str_section;
pub mod writer;

pub use abbrev::*;
pub use info::*;
pub use line::*;
pub use str_section::*;
pub use writer::*;

/// DWARF version 4
pub const DWARF_VERSION: u16 = 4;

/// DWARF section names
pub const DEBUG_LINE: &str = ".debug_line";
pub const DEBUG_INFO: &str = ".debug_info";
pub const DEBUG_ABBREV: &str = ".debug_abbrev";
pub const DEBUG_STR: &str = ".debug_str";
