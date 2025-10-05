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
    /// If user_label is provided, uses that; otherwise generates an automatic label.
    pub(crate) fn enter_loop(&mut self, user_label: Option<String>) -> Option<String> {
        self.loop_depth += 1;

        // Use user-provided label or generate automatic label
        let label = user_label.or_else(|| Some(format!("loop_{}", self.loop_depth)));

        self.loop_labels.push(label.clone());
        label
    }

    /// Exits the current loop scope.
    pub(crate) fn exit_loop(&mut self) {
        self.loop_labels.pop();
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }

    /// Gets the loop label - either from user-provided label name or defaults to innermost loop.
    pub(crate) fn resolve_loop_label(
        &self,
        user_label: Option<String>,
    ) -> rive_core::Result<Option<String>> {
        if let Some(label_name) = user_label {
            // Find the label in the stack
            for loop_label in self.loop_labels.iter().rev() {
                if loop_label.as_ref() == Some(&label_name) {
                    return Ok(Some(label_name));
                }
            }
            // Label not found
            return Err(rive_core::Error::Semantic(format!(
                "Label '{label_name}' not found"
            )));
        }

        // No user label, use innermost loop
        if self.loop_labels.is_empty() {
            return Err(rive_core::Error::Semantic(
                "Break/continue outside of loop".to_string(),
            ));
        }

        Ok(self.loop_labels.last().unwrap().clone())
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
