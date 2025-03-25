use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use colorful::{Color, Colorful};

/// Represents a memory address in a MIPS32 virtual machine.
///
/// Memory addresses are 32-bit values. This struct provides various
/// utility methods to work with these addresses, including conversion to and
/// from little-endian byte arrays, calculating page numbers, and formatting
/// the address for display.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Address(u32);

impl Address {
    /// Creates a new `Address` from a 32-bit unsigned integer.
    ///
    /// # Arguments
    ///
    /// * `address` - A 32-bit unsigned integer representing the memory address.
    ///
    /// # Returns
    ///
    /// A new `Address` instance.
    pub const fn new(address: u32) -> Address {
        Address(address)
    }

    pub fn from_page_number(page_number: u32) -> Address {
        Address(page_number << 12)
    }

    /// Returns the 32-bit unsigned integer value of the address.
    ///
    /// # Returns
    ///
    /// The 32-bit unsigned integer value of the address.
    pub fn unwrap(&self) -> u32 {
        self.0
    }

    /// Calculates the page number of the address.
    ///
    /// # Returns
    ///
    /// The page number of the address, assuming a page size of 4KB (12 bits).
    pub fn page_number(&self) -> u32 {
        self.0 >> 12
    }

    /// Calculates the offset of the address within the page.
    ///
    /// # Returns
    ///
    /// The offset of the address within the page, assuming a page size of 4KB (12 bits).
    pub fn page_offset(&self) -> u32 {
        self.0 & 0xFFF
    }

    /// Creates an `Address` from a little-endian byte array.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A 4-element array of bytes in little-endian order.
    ///
    /// # Returns
    ///
    /// A new `Address` instance.
    pub fn from_le_bytes(bytes: [u8; 4]) -> Address {
        Address::new(u32::from_le_bytes(bytes))
    }

    /// Converts the address to a little-endian byte array.
    ///
    /// # Returns
    ///
    /// A 4-element array of bytes representing the address in little-endian order.
    pub fn to_le_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Returns a new `Address` offset by a given number of bytes.
    ///
    /// # Arguments
    ///
    /// * `offset` - The number of bytes to offset the address by. Can be negative.
    ///
    /// # Returns
    ///
    /// A new `Address` instance offset by the given number of bytes.
    pub fn offset(&self, offset: i32) -> Address {
        Address((self.0 as i32 + offset) as u32)
    }

    /// Returns a string representation of the address in hexadecimal format.
    ///
    /// # Returns
    ///
    /// A string representing the address in hexadecimal format.
    pub fn show(&self) -> String {
        format!("0x{:08x}", self.0)
    }

    /// Returns a colored string representation of the address in hexadecimal format.
    ///
    /// # Returns
    ///
    /// A colored string representing the address in hexadecimal format.
    pub fn show_color(&self) -> String {
        format!("0x{:08x}", self.0)
            .color(Color::LightBlue)
            .to_string()
    }
}

impl From<i32> for Address {
    fn from(address: i32) -> Self {
        if address < 0 {
            panic!("Address can not be negative!")
        }
        Address::new(address as u32)
    }
}

impl From<u32> for Address {
    fn from(address: u32) -> Address {
        Address::new(address)
    }
}

impl From<usize> for Address {
    fn from(address: usize) -> Address {
        Address::new(address as u32)
    }
}

impl Add for Address {
    type Output = Address;

    fn add(self, other: Address) -> Address {
        Address::new(self.0 + other.0)
    }
}

impl Add<u32> for Address {
    type Output = Address;

    fn add(self, rhs: u32) -> Self::Output {
        Address::new(self.0 + rhs)
    }
}

impl Add<u16> for Address {
    type Output = Address;

    fn add(self, rhs: u16) -> Self::Output {
        Address::new(self.0 + rhs as u32)
    }
}

impl Add<i32> for Address {
    type Output = Address;

    fn add(self, rhs: i32) -> Self::Output {
        Address::new(self.0 + rhs as u32)
    }
}

impl Add<usize> for Address {
    type Output = Address;

    fn add(self, rhs: usize) -> Self::Output {
        Address::new(self.0 + rhs as u32)
    }
}

impl AddAssign for Address {
    fn add_assign(&mut self, rhs: Address) {
        self.0 += rhs.0;
    }
}

impl AddAssign<i32> for Address {
    fn add_assign(&mut self, rhs: i32) {
        if rhs < 0 {
            panic!("Address offset can not be negative!")
        }
        self.0 += rhs as u32;
    }
}

impl AddAssign<u32> for Address {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl AddAssign<usize> for Address {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs as u32
    }
}

impl Sub for Address {
    type Output = u32;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Sub<u32> for Address {
    type Output = Address;

    fn sub(self, rhs: u32) -> Self::Output {
        Address::new(self.0 - rhs)
    }
}

impl Sub<i32> for Address {
    type Output = Address;

    fn sub(self, rhs: i32) -> Self::Output {
        Address::new(self.0 - rhs as u32)
    }
}

impl Sub<usize> for Address {
    type Output = Address;

    fn sub(self, rhs: usize) -> Self::Output {
        Address::new(self.0 - rhs as u32)
    }
}

impl SubAssign for Address {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl SubAssign<u32> for Address {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 -= rhs
    }
}

impl SubAssign<i32> for Address {
    fn sub_assign(&mut self, rhs: i32) {
        self.0 -= rhs as u32
    }
}

impl SubAssign<usize> for Address {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs as u32
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}
