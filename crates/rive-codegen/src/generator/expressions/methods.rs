//! Method call code generation.
//!
//! This module handles method call generation and dispatch:
//! - List methods (len, append, get, etc.)
//! - Map methods (len, get, insert, etc.)
//! - Primitive type methods (to_float, to_int, trim, etc.)
//! - Field access (tuple indexing)

mod list;
mod map;
mod primitives;

use super::super::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::format_ident;
use rive_core::{Result, type_system::TypeId};
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for a method call.
    ///
    /// Dispatches to the appropriate method generator based on the object type.
    pub(crate) fn generate_method_call(
        &mut self,
        object: &RirExpression,
        method: &str,
        arguments: &[RirExpression],
        _return_type: TypeId,
    ) -> Result<TokenStream> {
        let object_expr = self.generate_expression(object)?;
        let arg_exprs: Result<Vec<_>> = arguments
            .iter()
            .map(|arg| self.generate_expression(arg))
            .collect();
        let arg_exprs = arg_exprs?;

        let object_type = object.type_id();

        // Check for composite types first (List, Map)
        if self.is_list_type(object_type) {
            return list::generate(object_expr, method, &arg_exprs);
        }
        if self.is_map_type(object_type) {
            return map::generate(object_expr, method, &arg_exprs);
        }

        // Primitive types
        primitives::generate(object_type, object_expr, method, &arg_exprs)
    }

    /// Generates code for field access (tuple indexing).
    ///
    /// # Example
    /// `t.0` → `t.0`
    pub(crate) fn generate_field_access(
        &mut self,
        object: &RirExpression,
        field: &str,
    ) -> Result<TokenStream> {
        let object_expr = self.generate_expression(object)?;
        
        // Try to parse as tuple index first (numeric)
        if let Ok(field_index) = field.parse::<usize>() {
            // Tuple field access
            let index = proc_macro2::Literal::usize_unsuffixed(field_index);
            Ok(quote::quote! { #object_expr.#index })
        } else {
            // Struct field access (named field)
            let field_ident = format_ident!("{}", field);
            Ok(quote::quote! { #object_expr.#field_ident })
        }
    }

    /// Checks if a type is a List type.
    fn is_list_type(&self, type_id: TypeId) -> bool {
        use rive_core::type_system::TypeKind;
        if let Some(meta) = self.type_registry.get(type_id) {
            matches!(meta.kind, TypeKind::List { .. })
        } else {
            eprintln!("Warning: TypeId {:?} not found in registry", type_id);
            false
        }
    }

    /// Checks if a type is a Map type.
    fn is_map_type(&self, type_id: TypeId) -> bool {
        use rive_core::type_system::TypeKind;
        self.type_registry
            .get(type_id)
            .is_some_and(|meta| matches!(meta.kind, TypeKind::Map { .. }))
    }
}
