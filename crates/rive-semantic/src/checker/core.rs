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

    /// Creates a new type checker with an existing symbol table.
    ///
    /// This is useful when you need to use a specific TypeRegistry
    /// (e.g., from the parser) instead of creating a new one.
    pub fn with_symbols(symbols: SymbolTable) -> Self {
        Self {
            symbols,
            current_function_return_type: None,
            loop_stack: Vec::new(),
        }
    }

    /// Consumes the type checker and returns the type registry.
    ///
    /// This is useful for extracting the type registry after semantic analysis
    /// so it can be passed to subsequent compilation stages.
    pub fn into_type_registry(self) -> rive_core::type_system::TypeRegistry {
        self.symbols.into_type_registry()
    }

    /// Checks if a type is nullable and returns the inner type if so.
    ///
    /// # Returns
    /// - `Some(inner_type)` if the type is `T?`
    /// - `None` if the type is not nullable
    pub(crate) fn get_nullable_inner(&self, type_id: TypeId) -> Option<TypeId> {
        use rive_core::type_system::TypeKind;

        let type_meta = self.symbols.type_registry().get(type_id)?;
        match type_meta.kind {
            TypeKind::Optional { inner } => Some(inner),
            _ => None,
        }
    }

    /// Checks if a type is nullable (T?).
    #[allow(dead_code)] // Used in future tasks
    pub(crate) fn is_nullable(&self, type_id: TypeId) -> bool {
        self.get_nullable_inner(type_id).is_some()
    }

    /// Gets or creates a nullable version of the given type.
    /// If the type is already nullable, returns it as-is.
    pub(crate) fn get_or_create_nullable(&mut self, type_id: TypeId) -> TypeId {
        use rive_core::type_system::TypeKind;

        // Check if already nullable
        if let Some(meta) = self.symbols.type_registry().get(type_id)
            && matches!(meta.kind, TypeKind::Optional { .. })
        {
            return type_id; // Already nullable
        }

        // Create nullable version
        self.symbols.type_registry_mut().create_optional(type_id)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
