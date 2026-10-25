//! Blocking console adapter for the legacy MARS-style MMIO example.
use mips_vm::{address::Address, memory::MemorySegment};
use std::io::{Read, Write};
fn read(address: Address) -> u8 {
    match address.unwrap() - 0xffff0000 {
        0 | 8 => 1, // ready; receiver data reads block until input or EOF
        4 => {
            let mut byte = [0];
            let _ = std::io::stdin().read(&mut byte);
            byte[0]
        }
        _ => 0,
    }
}
fn write(address: Address, value: u8) {
    if address.unwrap() == 0xffff000c {
        let _ = std::io::stdout().write_all(&[value]);
        let _ = std::io::stdout().flush();
    }
}
pub fn console() -> Vec<MemorySegment> {
    vec![MemorySegment::device(
        Address::new(0xffff0000),
        Address::new(0xffff0010),
        read,
        write,
    )]
}
