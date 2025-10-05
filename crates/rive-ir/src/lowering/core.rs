//! Core AST lowering structure and symbol management.

use rive_core::type_system::{TypeId, TypeRegistry};
use std::collections::HashMap;

/// Symbol information during lowering.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub type_id: TypeId,
    #[allow(dead_code)] // Will be used in future for mutability checks
    pub mutable: bool,
}

/// Converts AST to RIR with type information.
pub struct AstLowering {
    pub(crate) type_registry: TypeRegistry,
    /// Local symbol table for tracking variables and their types
    pub(crate) symbols: Vec<HashMap<String, SymbolInfo>>,
    /// Function signatures for function calls
    pub(crate) functions: HashMap<String, (Vec<TypeId>, TypeId)>,
    /// Current loop nesting depth
    pub(crate) loop_depth: usize,
    /// Stack of loop labels for break/continue
    pub(crate) loop_labels: Vec<Option<String>>,
}

impl AstLowering {
    /// Creates a new AST lowering instance.
    #[must_use]
    pub fn new(type_registry: TypeRegistry) -> Self {
        Self {
            type_registry,
            symbols: vec![HashMap::new()], // Global scope
            functions: HashMap::new(),
            loop_depth: 0,
            loop_labels: Vec::new(),
        }
    }

    /// Enters a new scope.
    pub(crate) fn enter_scope(&mut self) {
        self.symbols.push(HashMap::new());
    }

    /// Exits the current scope.
    pub(crate) fn exit_scope(&mut self) {
        self.symbols.pop();
    }

    /// Defines a variable in the current scope.
    pub(crate) fn define_variable(&mut self, name: String, type_id: TypeId, mutable: bool) {
        if let Some(scope) = self.symbols.last_mut() {
            scope.insert(name, SymbolInfo { type_id, mutable });
        }
    }

    /// Looks up a variable in all scopes.
    pub(crate) fn lookup_variable(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.symbols.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Defines a function signature.
    pub(crate) fn define_function(
        &mut self,
        name: String,
        params: Vec<TypeId>,
        return_type: TypeId,
    ) {
        self.functions.insert(name, (params, return_type));
    }

    /// Looks up a function signature.
    pub(crate) fn lookup_function(&self, name: &str) -> Option<&(Vec<TypeId>, TypeId)> {
        self.functions.get(name)
    }

    /// Enters a new loop scope and returns the label for this loop.
    pub(crate) fn enter_loop(&mut self) -> Option<String> {
        self.loop_depth += 1;

        // Generate label without the ' prefix (will be added in codegen)
        let label = Some(format!("loop_{}", self.loop_depth));

        self.loop_labels.push(label.clone());
        label
    }

    /// Exits the current loop scope.
    pub(crate) fn exit_loop(&mut self) {
        self.loop_labels.pop();
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }

    /// Gets the loop label at the specified depth (1 = current loop).
    pub(crate) fn get_loop_label(&self, depth: usize) -> rive_core::Result<Option<String>> {
        if depth == 0 || depth > self.loop_labels.len() {
            return Err(rive_core::Error::Semantic(format!(
                "Invalid loop depth: {depth}"
            )));
        }

        // depth = 1 means current loop (last in stack)
        // depth = 2 means parent loop (second from last)
        let index = self.loop_labels.len() - depth;
        Ok(self.loop_labels[index].clone())
    }

    /// Checks if a type is nullable and returns the inner type if so.
    ///
    /// # Returns
    /// - `Some(inner_type)` if the type is `T?`
    /// - `None` if the type is not nullable
    pub(crate) fn get_nullable_inner(&self, type_id: TypeId) -> Option<TypeId> {
        use rive_core::type_system::TypeKind;

        let type_meta = self.type_registry.get(type_id)?;
        match type_meta.kind {
            TypeKind::Optional { inner } => Some(inner),
            _ => None,
        }
    }
}
