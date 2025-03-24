use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use colorful::{Color, Colorful};

/// Represents an address in a MIPS program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Address(u32);

impl Address {
    pub fn new(address: u32) -> Address {
        Address(address)
    }

    pub fn from_le_bytes(bytes: [u8; 4]) -> Address {
        Address::new(u32::from_le_bytes(bytes))
    }

    pub fn to_le_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn offset(&self, offset: i32) -> Address {
        Address((self.0 as i32 + offset) as u32)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn show(&self) -> String {
        format!("0x{:08x}", self.0)
    }

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
