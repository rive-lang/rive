//! Core code generator implementation.

use super::{inline, types};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Error, Result};
use rive_ir::{RirBlock, RirFunction, RirModule};

/// Loop context for tracking result variables in loop expressions.
#[derive(Debug, Clone)]
struct LoopContext {
    /// Name of the result variable for this loop (e.g., "__for_result")
    result_var: String,
}

/// Code generator for Rive programs.
pub struct CodeGenerator {
    /// Stack of loop contexts for rewriting break statements
    loop_stack: Vec<LoopContext>,
    /// Type registry for type lookups during codegen
    pub(crate) type_registry: rive_core::type_system::TypeRegistry,
}

impl CodeGenerator {
    /// Creates a new code generator.
    pub fn new() -> Self {
        Self {
            loop_stack: Vec::new(),
            type_registry: rive_core::type_system::TypeRegistry::new(),
        }
    }

    /// Enters a loop context with a result variable.
    pub(crate) fn enter_loop_context(&mut self, result_var: Option<String>) {
        if let Some(var) = result_var {
            self.loop_stack.push(LoopContext { result_var: var });
        }
    }

    /// Exits the current loop context.
    pub(crate) fn exit_loop_context(&mut self) {
        self.loop_stack.pop();
    }

    /// Gets the current loop's result variable name, if in a loop context.
    pub(crate) fn current_loop_result_var(&self) -> Option<&str> {
        self.loop_stack.last().map(|ctx| ctx.result_var.as_str())
    }

    /// Generates a loop (for/while/loop) as a statement (no return value).
    pub(crate) fn generate_loop_stmt(
        &mut self,
        expr: &rive_ir::RirExpression,
    ) -> Result<TokenStream> {
        use crate::generator::control_flow::ForLoopParams;

        match expr {
            rive_ir::RirExpression::For {
                variable,
                start,
                end,
                inclusive,
                body,
                label,
                ..
            } => {
                let params = ForLoopParams {
                    variable,
                    start,
                    end,
                    inclusive: *inclusive,
                    body,
                    label,
                };
                self.generate_for(params)
            }
            rive_ir::RirExpression::While {
                condition,
                body,
                label,
                ..
            } => self.generate_while(condition, body, label),
            rive_ir::RirExpression::Loop { body, label, .. } => self.generate_loop(body, label),
            _ => unreachable!("generate_loop_stmt called on non-loop expression"),
        }
    }

    /// Generates Rust code from a RIR module.
    pub fn generate(&mut self, module: &RirModule) -> Result<String> {
        // Copy the type registry from the module
        self.type_registry = module.type_registry.clone();

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
            // Special case: if final_expr is a loop without break value, treat it as a statement
            // This prevents generating { let __result = None; for ... {} __result } wrapper
            if final_expr.is_loop() {
                let loop_stmt = self.generate_loop_stmt(final_expr)?;
                Ok(quote! {
                    #(#statements)*
                    #loop_stmt
                })
            } else {
                let expr = self.generate_expression(final_expr)?;
                Ok(quote! {
                    #(#statements)*
                    #expr
                })
            }
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
