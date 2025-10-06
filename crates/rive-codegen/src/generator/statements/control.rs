//! Control flow statement helper (return statement).

use crate::generator::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::quote;
use rive_core::Result;
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for a `return` statement.
    pub(crate) fn generate_return(&mut self, value: Option<&RirExpression>) -> Result<TokenStream> {
        if let Some(expr) = value {
            let generated_expr = self.generate_expression(expr)?;
            Ok(quote! { return #generated_expr; })
        } else {
            Ok(quote! { return; })
        }
    }
}
