//! Symbol table for tracking variables and functions during semantic analysis.

use rive_core::types::Type;
use std::collections::HashMap;

/// Represents a symbol in the symbol table.
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// The name of the symbol
    pub name: String,
    /// The type of the symbol
    pub symbol_type: Type,
    /// Whether the symbol is mutable
    pub mutable: bool,
    /// Whether the symbol has been initialized
    pub initialized: bool,
}

impl Symbol {
    /// Creates a new symbol.
    pub fn new(name: String, symbol_type: Type, mutable: bool) -> Self {
        Self {
            name,
            symbol_type,
            mutable,
            initialized: true,
        }
    }
}

/// Symbol table for managing variable and function scopes.
///
/// The symbol table uses a stack of scopes to handle nested blocks.
/// Each scope maps symbol names to their definitions.
#[derive(Debug)]
pub struct SymbolTable {
    /// Stack of scopes, with the current scope at the top
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    /// Creates a new symbol table with a global scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Enters a new scope.
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exits the current scope.
    ///
    /// # Panics
    /// Panics if attempting to exit the global scope.
    pub fn exit_scope(&mut self) {
        if self.scopes.len() <= 1 {
            panic!("Cannot exit global scope");
        }
        self.scopes.pop();
    }

    /// Defines a new symbol in the current scope.
    ///
    /// # Arguments
    /// * `symbol` - The symbol to define
    ///
    /// # Returns
    /// * `Ok(())` if the symbol was defined successfully
    /// * `Err` if a symbol with the same name already exists in the current scope
    pub fn define(&mut self, symbol: Symbol) -> Result<(), String> {
        let current_scope = self.scopes.last_mut().unwrap();

        if current_scope.contains_key(&symbol.name) {
            return Err(format!(
                "Symbol '{}' is already defined in this scope",
                symbol.name
            ));
        }

        current_scope.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Looks up a symbol by name, searching from the current scope upwards.
    ///
    /// # Arguments
    /// * `name` - The name of the symbol to find
    ///
    /// # Returns
    /// * `Some(&Symbol)` if the symbol was found
    /// * `None` if the symbol was not found in any scope
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Looks up a symbol by name (mutable version).
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Returns the current scope depth (0 = global scope).
    pub fn depth(&self) -> usize {
        self.scopes.len() - 1
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_basic() {
        let mut table = SymbolTable::new();

        let symbol = Symbol::new("x".to_string(), Type::Int, false);
        assert!(table.define(symbol).is_ok());

        let found = table.lookup("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "x");
        assert_eq!(found.unwrap().symbol_type, Type::Int);
    }

    #[test]
    fn test_symbol_table_scopes() {
        let mut table = SymbolTable::new();

        // Define in global scope
        let global_symbol = Symbol::new("x".to_string(), Type::Int, false);
        table.define(global_symbol).unwrap();

        // Enter new scope
        table.enter_scope();

        // Define in local scope (shadows global)
        let local_symbol = Symbol::new("x".to_string(), Type::Text, false);
        table.define(local_symbol).unwrap();

        // Lookup finds local definition
        let found = table.lookup("x").unwrap();
        assert_eq!(found.symbol_type, Type::Text);

        // Exit scope
        table.exit_scope();

        // Lookup finds global definition again
        let found = table.lookup("x").unwrap();
        assert_eq!(found.symbol_type, Type::Int);
    }

    #[test]
    fn test_symbol_table_duplicate_error() {
        let mut table = SymbolTable::new();

        let symbol1 = Symbol::new("x".to_string(), Type::Int, false);
        table.define(symbol1).unwrap();

        let symbol2 = Symbol::new("x".to_string(), Type::Text, false);
        assert!(table.define(symbol2).is_err());
    }

    #[test]
    fn test_symbol_table_undefined() {
        let table = SymbolTable::new();
        assert!(table.lookup("undefined").is_none());
    }
}
