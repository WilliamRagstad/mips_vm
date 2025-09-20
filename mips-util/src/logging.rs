//! Logging utilities

pub use log::{debug, error, info, trace, warn};

/// Initialize logging with default configuration
pub fn init_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// Initialize logging with specified level
pub fn init_logging_with_level(level: log::LevelFilter) {
    env_logger::Builder::from_default_env()
        .filter_level(level)
        .init();
}

/// Initialize logging for debug builds
pub fn init_debug_logging() {
    #[cfg(debug_assertions)]
    init_logging_with_level(log::LevelFilter::Debug);
    
    #[cfg(not(debug_assertions))]
    init_logging_with_level(log::LevelFilter::Info);
}

/// Log a hexdump of data
pub fn hexdump(data: &[u8], address: u32) {
    for (i, chunk) in data.chunks(16).enumerate() {
        let addr = address + (i * 16) as u32;
        let hex: String = chunk
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(" ");
        
        let ascii: String = chunk
            .iter()
            .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
            .collect();
        
        debug!("{:08x}: {:48} |{}|", addr, hex, ascii);
    }
}
