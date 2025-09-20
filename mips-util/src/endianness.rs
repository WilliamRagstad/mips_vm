//! Endianness utilities

/// Byte order for data serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    /// Convert a u32 to bytes with the specified endianness
    pub fn to_bytes(self, value: u32) -> [u8; 4] {
        match self {
            Endianness::Little => value.to_le_bytes(),
            Endianness::Big => value.to_be_bytes(),
        }
    }

    /// Convert bytes to u32 with the specified endianness
    pub fn from_bytes(self, bytes: [u8; 4]) -> u32 {
        match self {
            Endianness::Little => u32::from_le_bytes(bytes),
            Endianness::Big => u32::from_be_bytes(bytes),
        }
    }

    /// Convert a u16 to bytes with the specified endianness
    pub fn to_bytes_u16(self, value: u16) -> [u8; 2] {
        match self {
            Endianness::Little => value.to_le_bytes(),
            Endianness::Big => value.to_be_bytes(),
        }
    }

    /// Convert bytes to u16 with the specified endianness
    pub fn from_bytes_u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            Endianness::Little => u16::from_le_bytes(bytes),
            Endianness::Big => u16::from_be_bytes(bytes),
        }
    }
}

/// Get the native endianness of the current platform
pub fn native_endianness() -> Endianness {
    if cfg!(target_endian = "little") {
        Endianness::Little
    } else {
        Endianness::Big
    }
}
