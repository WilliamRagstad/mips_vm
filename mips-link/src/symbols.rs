//! Symbol table management

use std::collections::HashMap;

/// Symbol binding types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolBinding {
    Local = 0,
    Global = 1,
    Weak = 2,
}

/// Symbol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    NoType = 0,
    Object = 1,
    Function = 2,
    Section = 3,
    File = 4,
}

/// Symbol entry
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub value: u32,
    pub size: u32,
    pub binding: SymbolBinding,
    pub symbol_type: SymbolType,
    pub section: Option<u16>,
}

/// Symbol table
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    undefined: Vec<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            undefined: Vec::new(),
        }
    }

    /// Add a symbol definition
    pub fn define_symbol(&mut self, symbol: Symbol) -> Result<(), SymbolError> {
        if self.symbols.contains_key(&symbol.name) {
            return Err(SymbolError::MultipleDefinition(symbol.name.clone()));
        }
        
        self.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Mark a symbol as undefined (needs to be resolved)
    pub fn add_undefined(&mut self, name: String) {
        if !self.symbols.contains_key(&name) && !self.undefined.contains(&name) {
            self.undefined.push(name);
        }
    }

    /// Get a symbol by name
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Get all undefined symbols
    pub fn undefined_symbols(&self) -> &[String] {
        &self.undefined
    }

    /// Check if all symbols are resolved
    pub fn is_fully_resolved(&self) -> bool {
        self.undefined.is_empty()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SymbolError {
    #[error("Multiple definition of symbol: {0}")]
    MultipleDefinition(String),
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),
}
