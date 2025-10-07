//! Nullable type operation code generation.
//!
//! This module handles null-related operations:
//! - Null literal: `null` → `None`
//! - Elvis operator: `value ?: fallback` → `value.unwrap_or(fallback)`
//! - Safe call: `object?.method()` → `object.and_then(|obj| ...)`
//! - Wrap optional: `Some(value)`

use super::super::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::quote;
use rive_core::Result;
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for null literal.
    ///
    /// # Example
    /// `null` → `None`
    pub(crate) fn generate_null_literal(&self) -> Result<TokenStream> {
        Ok(quote! { None })
    }

    /// Generates code for wrapping a value in Optional.
    ///
    /// # Example
    /// `Some(42)` → `Some(42)`
    pub(crate) fn generate_wrap_optional(&mut self, value: &RirExpression) -> Result<TokenStream> {
        let value_expr = self.generate_expression(value)?;
        Ok(quote! { Some(#value_expr) })
    }

    /// Generates code for Elvis operator (null-coalescing).
    ///
    /// # Example
    /// `value ?: fallback` compiles to:
    /// - `value.unwrap_or(fallback)` if fallback is a simple value
    /// - `value.unwrap_or_else(|| fallback)` if fallback is a complex expression
    pub(crate) fn generate_elvis(
        &mut self,
        value: &RirExpression,
        fallback: &RirExpression,
    ) -> Result<TokenStream> {
        let value_expr = self.generate_expression(value)?;
        let fallback_expr = self.generate_expression(fallback)?;

        // Check if fallback is a simple literal or variable
        // If so, use unwrap_or, otherwise use unwrap_or_else
        let is_simple = matches!(
            fallback,
            RirExpression::IntLiteral { .. }
                | RirExpression::FloatLiteral { .. }
                | RirExpression::StringLiteral { .. }
                | RirExpression::BoolLiteral { .. }
                | RirExpression::Variable { .. }
                | RirExpression::NullLiteral { .. }
        );

        if is_simple {
            Ok(quote! { #value_expr.unwrap_or(#fallback_expr) })
        } else {
            Ok(quote! { #value_expr.unwrap_or_else(|| #fallback_expr) })
        }
    }

    /// Generates code for Safe Call operator.
    ///
    /// # Example
    /// `object?.method()` compiles to:
    /// - `object.and_then(|obj| /* rewrite method() to use obj */)`
    ///
    /// # Note
    /// Currently, we use a simplified approach. The call expression
    /// is evaluated independently, but it should reference the object.
    /// A more sophisticated approach would rewrite the call to use
    /// the unwrapped object value.
    pub(crate) fn generate_safe_call(
        &mut self,
        object: &RirExpression,
        call: &RirExpression,
    ) -> Result<TokenStream> {
        let object_expr = self.generate_expression(object)?;
        let call_expr = self.generate_expression(call)?;

        // Generate: object.map(|_| call)
        // In a real implementation, we'd need to rewrite the call to use the unwrapped object
        // For now, we assume the call is self-contained
        Ok(quote! { #object_expr.and_then(|_obj| Some(#call_expr)) })
    }
}
