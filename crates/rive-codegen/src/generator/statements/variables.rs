//! Variable statement generation (let, assign, assign_index).

use crate::generator::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::Result;
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for a `let` statement.
    pub(crate) fn generate_let(
        &mut self,
        name: &str,
        is_mutable: bool,
        value: &RirExpression,
    ) -> Result<TokenStream> {
        let var_name = format_ident!("{}", name);
        let expr = self.generate_expression(value)?;

        if is_mutable {
            Ok(quote! { let mut #var_name = #expr; })
        } else {
            Ok(quote! { let #var_name = #expr; })
        }
    }

    /// Generates code for an assignment statement.
    pub(crate) fn generate_assign(
        &mut self,
        name: &str,
        value: &RirExpression,
    ) -> Result<TokenStream> {
        let var_name = format_ident!("{}", name);
        let expr = self.generate_expression(value)?;
        Ok(quote! { #var_name = #expr; })
    }

    /// Generates code for an array/index assignment.
    pub(crate) fn generate_assign_index(
        &mut self,
        array: &str,
        index: &RirExpression,
        value: &RirExpression,
    ) -> Result<TokenStream> {
        let array_name = format_ident!("{}", array);
        let index_expr = self.generate_expression(index)?;
        let value_expr = self.generate_expression(value)?;
        Ok(quote! { #array_name[#index_expr] = #value_expr; })
    }
}
