//! Core type checker structure.

use crate::checker::loops::LoopContext;
use crate::symbol_table::SymbolTable;
use rive_core::type_system::TypeId;

/// Type checker for Rive programs.
///
/// Performs type checking and semantic validation on AST nodes,
/// ensuring type safety and proper variable usage.
pub struct TypeChecker {
    /// Symbol table for tracking variables and functions
    pub(crate) symbols: SymbolTable,
    /// The expected return type of the current function
    pub(crate) current_function_return_type: Option<TypeId>,
    /// Stack of loop contexts for break/continue validation
    pub(crate) loop_stack: Vec<LoopContext>,
}

impl TypeChecker {
    /// Creates a new type checker.
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            current_function_return_type: None,
            loop_stack: Vec::new(),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
