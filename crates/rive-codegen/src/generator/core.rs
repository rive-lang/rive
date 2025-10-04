//! Core code generator implementation.

use super::{inline, types};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Error, Result};
use rive_ir::{RirBlock, RirFunction, RirModule};

/// Code generator for Rive programs.
pub struct CodeGenerator;

impl CodeGenerator {
    /// Creates a new code generator.
    pub fn new() -> Self {
        Self
    }

    /// Generates Rust code from a RIR module.
    pub fn generate(&mut self, module: &RirModule) -> Result<String> {
        let items: Result<Vec<_>> = module
            .functions
            .iter()
            .map(|function| self.generate_function(function))
            .collect();

        let items = items?;

        let tokens = quote! {
            #(#items)*
        };

        let syntax_tree = syn::parse2::<syn::File>(tokens)
            .map_err(|e| Error::Codegen(format!("Failed to parse generated code: {e}")))?;

        Ok(prettyplease::unparse(&syntax_tree))
    }

    /// Generates code for a RIR function.
    pub(crate) fn generate_function(&mut self, function: &RirFunction) -> Result<TokenStream> {
        let name = format_ident!("{}", function.name);
        let params = self.generate_parameters(&function.parameters)?;
        let return_type = types::generate_return_type(function.return_type);
        let body = self.generate_block(&function.body)?;

        if inline::should_inline_function(function) {
            Ok(quote! {
                #[inline]
                fn #name(#(#params),*) #return_type {
                    #body
                }
            })
        } else {
            Ok(quote! {
                fn #name(#(#params),*) #return_type {
                    #body
                }
            })
        }
    }

    /// Generates function parameters.
    pub(crate) fn generate_parameters(
        &self,
        params: &[rive_ir::RirParameter],
    ) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|param| {
                let name = format_ident!("{}", param.name);
                let ty = types::rust_type(param.type_id, param.memory_strategy)?;
                Ok(quote! { #name: #ty })
            })
            .collect()
    }

    /// Generates code for a RIR block.
    pub(crate) fn generate_block(&mut self, block: &RirBlock) -> Result<TokenStream> {
        let statements: Result<Vec<_>> = block
            .statements
            .iter()
            .map(|stmt| self.generate_statement(stmt))
            .collect();

        let statements = statements?;

        if let Some(final_expr) = &block.final_expr {
            let expr = self.generate_expression(final_expr)?;
            Ok(quote! {
                #(#statements)*
                #expr
            })
        } else {
            Ok(quote! {
                #(#statements)*
            })
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
