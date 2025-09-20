//! Bitfield manipulation utilities

/// Extract a bitfield from a value
pub fn extract_bits(value: u32, start: u8, len: u8) -> u32 {
    let mask = (1u32 << len) - 1;
    (value >> start) & mask
}

/// Insert bits into a value
pub fn insert_bits(value: u32, bits: u32, start: u8, len: u8) -> u32 {
    let mask = (1u32 << len) - 1;
    let cleared = value & !(mask << start);
    cleared | ((bits & mask) << start)
}

/// Sign extend a value from a given bit width
pub fn sign_extend(value: u32, bits: u8) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

/// Zero extend a value to 32 bits
pub fn zero_extend(value: u32, _bits: u8) -> u32 {
    value
}

/// Check if a bit is set
pub fn is_bit_set(value: u32, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}

/// Set a bit
pub fn set_bit(value: u32, bit: u8) -> u32 {
    value | (1 << bit)
}

/// Clear a bit
pub fn clear_bit(value: u32, bit: u8) -> u32 {
    value & !(1 << bit)
}

/// Toggle a bit
pub fn toggle_bit(value: u32, bit: u8) -> u32 {
    value ^ (1 << bit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bits() {
        assert_eq!(extract_bits(0b11010110, 2, 3), 0b101);
        assert_eq!(extract_bits(0xDEADBEEF, 16, 16), 0xDEAD);
    }

    #[test]
    fn test_insert_bits() {
        assert_eq!(insert_bits(0b11110000, 0b101, 2, 3), 0b11110100);
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0xFF, 8), -1);
        assert_eq!(sign_extend(0x7F, 8), 127);
    }
}
