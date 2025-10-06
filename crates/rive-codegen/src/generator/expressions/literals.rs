//! Literal expression code generation.
//!
//! This module handles generation of primitive literals:
//! - Unit `()`
//! - Integer literals
//! - Float literals
//! - String literals
//! - Boolean literals
//! - Variable references
//! - Array literals

use super::super::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::Result;
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for unit literal `()`.
    pub(crate) fn generate_unit(&self) -> Result<TokenStream> {
        Ok(quote! { () })
    }

    /// Generates code for an integer literal.
    pub(crate) fn generate_int_literal(&self, value: i64) -> Result<TokenStream> {
        let lit = proc_macro2::Literal::i64_unsuffixed(value);
        Ok(quote! { #lit })
    }

    /// Generates code for a float literal.
    pub(crate) fn generate_float_literal(&self, value: f64) -> Result<TokenStream> {
        let lit = proc_macro2::Literal::f64_suffixed(value);
        Ok(quote! { #lit })
    }

    /// Generates code for a string literal.
    pub(crate) fn generate_string_literal(&self, value: &str) -> Result<TokenStream> {
        let lit = proc_macro2::Literal::string(value);
        Ok(quote! { #lit })
    }

    /// Generates code for a boolean literal.
    pub(crate) fn generate_bool_literal(&self, value: bool) -> Result<TokenStream> {
        Ok(quote! { #value })
    }

    /// Generates code for a variable reference.
    pub(crate) fn generate_variable(&self, name: &str) -> Result<TokenStream> {
        let var_name = format_ident!("{}", name);
        Ok(quote! { #var_name })
    }

    /// Generates code for an array literal.
    pub(crate) fn generate_array_literal(
        &mut self,
        elements: &[RirExpression],
    ) -> Result<TokenStream> {
        let elems = elements
            .iter()
            .map(|elem| self.generate_expression(elem))
            .collect::<Result<Vec<_>>>()?;

        Ok(quote! { [#(#elems),*] })
    }

    /// Generates code for array indexing.
    pub(crate) fn generate_index(
        &mut self,
        array: &RirExpression,
        index: &RirExpression,
    ) -> Result<TokenStream> {
        let array_expr = self.generate_expression(array)?;
        let index_expr = self.generate_expression(index)?;
        Ok(quote! { #array_expr[#index_expr] })
    }
}
