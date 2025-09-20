//! High-performance symbol table using hashbrown and ahash

use hashbrown::HashMap;
use ahash::RandomState;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbolError {
    #[error("Symbol '{0}' not found")]
    NotFound(String),
    #[error("Symbol '{0}' already exists")]
    AlreadyExists(String),
    #[error("Invalid symbol name: '{0}'")]
    InvalidName(String),
}

/// Symbol types in the program
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    /// Function entry point
    Function,
    /// Data label
    Data,
    /// Generic label
    Label,
    /// External symbol (for linking)
    External,
}

/// Information about a symbol
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,
    /// Address in memory
    pub address: u32,
    /// Symbol type
    pub symbol_type: SymbolType,
    /// Size in bytes (if known)
    pub size: Option<u32>,
    /// Whether the symbol is global
    pub global: bool,
}

/// High-performance symbol table using hashbrown HashMap with ahash
pub struct SymbolTable {
    /// Main symbol table: name -> symbol info
    symbols: HashMap<String, SymbolInfo, RandomState>,
    /// Reverse mapping: address -> symbol name for fast lookups
    address_map: HashMap<u32, String, RandomState>,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::with_hasher(RandomState::new()),
            address_map: HashMap::with_hasher(RandomState::new()),
        }
    }

    /// Create a symbol table with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            symbols: HashMap::with_capacity_and_hasher(capacity, RandomState::new()),
            address_map: HashMap::with_capacity_and_hasher(capacity, RandomState::new()),
        }
    }

    /// Add a symbol to the table
    pub fn add_symbol(
        &mut self, 
        name: String, 
        address: u32, 
        symbol_type: SymbolType,
        size: Option<u32>,
        global: bool,
    ) -> Result<(), SymbolError> {
        if self.symbols.contains_key(&name) {
            return Err(SymbolError::AlreadyExists(name));
        }

        if name.is_empty() {
            return Err(SymbolError::InvalidName(name));
        }

        let symbol_info = SymbolInfo {
            name: name.clone(),
            address,
            symbol_type,
            size,
            global,
        };

        self.symbols.insert(name.clone(), symbol_info);
        self.address_map.insert(address, name);

        Ok(())
    }

    /// Get symbol information by name
    pub fn get_symbol(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        self.symbols.get(name).ok_or_else(|| SymbolError::NotFound(name.to_string()))
    }

    /// Get symbol address by name
    pub fn get_address(&self, name: &str) -> Result<u32, SymbolError> {
        self.get_symbol(name).map(|info| info.address)
    }

    /// Get symbol name by address
    pub fn get_symbol_at_address(&self, address: u32) -> Option<&str> {
        self.address_map.get(&address).map(|s| s.as_str())
    }

    /// Check if a symbol exists
    pub fn contains_symbol(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Remove a symbol
    pub fn remove_symbol(&mut self, name: &str) -> Result<SymbolInfo, SymbolError> {
        if let Some(symbol_info) = self.symbols.remove(name) {
            self.address_map.remove(&symbol_info.address);
            Ok(symbol_info)
        } else {
            Err(SymbolError::NotFound(name.to_string()))
        }
    }

    /// Get all symbols of a specific type
    pub fn symbols_by_type(&self, symbol_type: SymbolType) -> Vec<&SymbolInfo> {
        self.symbols
            .values()
            .filter(|info| info.symbol_type == symbol_type)
            .collect()
    }

    /// Get all global symbols
    pub fn global_symbols(&self) -> Vec<&SymbolInfo> {
        self.symbols
            .values()
            .filter(|info| info.global)
            .collect()
    }

    /// Find symbols within an address range
    pub fn symbols_in_range(&self, start: u32, end: u32) -> Vec<&SymbolInfo> {
        self.symbols
            .values()
            .filter(|info| info.address >= start && info.address <= end)
            .collect()
    }

    /// Clear all symbols
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.address_map.clear();
    }

    /// Get the number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Get an iterator over all symbols
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolInfo)> {
        self.symbols.iter()
    }

    /// Get an iterator over symbol names
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.symbols.keys()
    }

    /// Get an iterator over symbol infos
    pub fn infos(&self) -> impl Iterator<Item = &SymbolInfo> {
        self.symbols.values()
    }

    /// Add common MIPS runtime symbols
    pub fn add_runtime_symbols(&mut self) {
        // Standard MIPS runtime entry points
        let _ = self.add_symbol(
            "_start".to_string(),
            0x00400000,
            SymbolType::Function,
            None,
            true,
        );

        // System call table base
        let _ = self.add_symbol(
            "__syscall_table".to_string(),
            0x80000000,
            SymbolType::Data,
            Some(256 * 4), // 256 syscalls * 4 bytes each
            false,
        );
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolTable")
            .field("symbol_count", &self.len())
            .field("symbols", &self.symbols)
            .finish()
    }
}

impl fmt::Display for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Symbol Table ({} symbols):", self.len())?;
        
        let mut symbols: Vec<_> = self.symbols.values().collect();
        symbols.sort_by_key(|s| s.address);

        for symbol in symbols {
            write!(f, "  0x{:08x} {:<15} {:?}", 
                   symbol.address, 
                   symbol.name,
                   symbol.symbol_type)?;
            
            if let Some(size) = symbol.size {
                write!(f, " (size: {} bytes)", size)?;
            }
            
            if symbol.global {
                write!(f, " [GLOBAL]")?;
            }
            
            writeln!(f)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_add_and_get_symbol() {
        let mut table = SymbolTable::new();
        
        table.add_symbol(
            "main".to_string(),
            0x00400000,
            SymbolType::Function,
            Some(100),
            true,
        ).unwrap();

        let symbol = table.get_symbol("main").unwrap();
        assert_eq!(symbol.name, "main");
        assert_eq!(symbol.address, 0x00400000);
        assert_eq!(symbol.symbol_type, SymbolType::Function);
        assert_eq!(symbol.size, Some(100));
        assert!(symbol.global);

        assert_eq!(table.get_address("main").unwrap(), 0x00400000);
        assert_eq!(table.get_symbol_at_address(0x00400000), Some("main"));
    }

    #[test]
    fn test_duplicate_symbol() {
        let mut table = SymbolTable::new();
        
        table.add_symbol(
            "test".to_string(),
            0x1000,
            SymbolType::Label,
            None,
            false,
        ).unwrap();

        let result = table.add_symbol(
            "test".to_string(),
            0x2000,
            SymbolType::Label,
            None,
            false,
        );

        assert!(result.is_err());
        match result {
            Err(SymbolError::AlreadyExists(name)) => assert_eq!(name, "test"),
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[test]
    fn test_symbol_not_found() {
        let table = SymbolTable::new();
        
        let result = table.get_symbol("nonexistent");
        assert!(result.is_err());
        match result {
            Err(SymbolError::NotFound(name)) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_symbols_by_type() {
        let mut table = SymbolTable::new();
        
        table.add_symbol("func1".to_string(), 0x1000, SymbolType::Function, None, false).unwrap();
        table.add_symbol("func2".to_string(), 0x2000, SymbolType::Function, None, false).unwrap();
        table.add_symbol("data1".to_string(), 0x3000, SymbolType::Data, None, false).unwrap();

        let functions = table.symbols_by_type(SymbolType::Function);
        assert_eq!(functions.len(), 2);

        let data = table.symbols_by_type(SymbolType::Data);
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn test_remove_symbol() {
        let mut table = SymbolTable::new();
        
        table.add_symbol("test".to_string(), 0x1000, SymbolType::Label, None, false).unwrap();
        assert!(table.contains_symbol("test"));

        let removed = table.remove_symbol("test").unwrap();
        assert_eq!(removed.name, "test");
        assert!(!table.contains_symbol("test"));
        assert_eq!(table.get_symbol_at_address(0x1000), None);
    }
}
