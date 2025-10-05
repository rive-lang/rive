//! Control flow code generation.

use super::{core::CodeGenerator, labels};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Result, type_system::TypeId};
use rive_ir::{RirBlock, RirExpression};

/// Parameters for generating a for loop.
pub(crate) struct ForLoopParams<'a> {
    pub variable: &'a str,
    pub start: &'a RirExpression,
    pub end: &'a RirExpression,
    pub inclusive: bool,
    pub body: &'a RirBlock,
    pub label: &'a Option<String>,
}

impl CodeGenerator {
    /// Generates code for an if statement/expression.
    /// Works for both statements (optional else) and expressions (required else).
    pub(crate) fn generate_if(
        &mut self,
        condition: &RirExpression,
        then_block: &RirBlock,
        else_block: Option<&RirBlock>,
    ) -> Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let then_body = self.generate_block(then_block)?;

        if let Some(else_blk) = else_block {
            let else_body = self.generate_block(else_blk)?;
            Ok(quote! {
                if #cond {
                    #then_body
                } else {
                    #else_body
                }
            })
        } else {
            Ok(quote! {
                if #cond {
                    #then_body
                }
            })
        }
    }

    /// Generates code for an if expression (must have else).
    pub(crate) fn generate_if_expr(
        &mut self,
        condition: &RirExpression,
        then_block: &RirBlock,
        else_block: &RirBlock,
    ) -> Result<TokenStream> {
        self.generate_if(condition, then_block, Some(else_block))
    }

    /// Generates code for a while loop statement.
    pub(crate) fn generate_while(
        &mut self,
        condition: &RirExpression,
        body: &RirBlock,
        label: &Option<String>,
    ) -> Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let body_stmts = self.generate_block(body)?;

        Ok(labels::with_label(label, || {
            quote! {
                while #cond {
                    #body_stmts
                }
            }
        }))
    }

    /// Generates code for a for loop statement.
    pub(crate) fn generate_for(&mut self, params: ForLoopParams<'_>) -> Result<TokenStream> {
        let var = format_ident!("{}", params.variable);
        let start_expr = self.generate_expression(params.start)?;
        let end_expr = self.generate_expression(params.end)?;
        let body_stmts = self.generate_block(params.body)?;
        let range = labels::generate_range(&start_expr, &end_expr, params.inclusive);

        Ok(labels::with_label(params.label, || {
            quote! {
                for #var in #range {
                    #body_stmts
                }
            }
        }))
    }

    /// Generates code for an infinite loop statement.
    pub(crate) fn generate_loop(
        &mut self,
        body: &RirBlock,
        label: &Option<String>,
    ) -> Result<TokenStream> {
        let body_stmts = self.generate_block(body)?;

        Ok(labels::with_label(label, || {
            quote! {
                loop {
                    #body_stmts
                }
            }
        }))
    }

    /// Generates code for a break statement.
    pub(crate) fn generate_break(
        &mut self,
        label: &Option<String>,
        value: &Option<Box<RirExpression>>,
    ) -> Result<TokenStream> {
        // Check if we're in a loop expression context (for/while with result variable)
        if let Some(result_var) = self.current_loop_result_var() {
            let result_ident = format_ident!("{}", result_var);

            // Generate: __result = Some(value); break label;
            if let Some(expr) = value {
                let val_expr = self.generate_expression(expr)?;
                let break_stmt = labels::generate_break_stmt(label, &None);
                Ok(quote! {
                    {
                        #result_ident = Some(#val_expr);
                        #break_stmt
                    }
                })
            } else {
                // No value, just break (result stays None)
                Ok(labels::generate_break_stmt(label, &None))
            }
        } else {
            // In loop expression context, wrap value in Some/None
            let value_token = if let Some(expr) = value {
                let val_expr = self.generate_expression(expr)?;
                Some(quote! { Some(#val_expr) })
            } else {
                Some(quote! { None })
            };

            Ok(labels::generate_break_stmt(label, &value_token))
        }
    }

    /// Generates code for a continue statement.
    pub(crate) fn generate_continue(&mut self, label: &Option<String>) -> Result<TokenStream> {
        Ok(labels::generate_continue_stmt(label))
    }

    /// Wraps a loop with result variable for expression context.
    /// This helper reduces duplication across while/for/loop expression generators.
    fn wrap_loop_as_expression<F>(
        &mut self,
        result_var_name: &str,
        loop_body_gen: F,
    ) -> Result<TokenStream>
    where
        F: FnOnce(&mut Self) -> Result<TokenStream>,
    {
        let result_var = format_ident!("{}", result_var_name);
        self.enter_loop_context(Some(result_var_name.to_string()));
        let loop_code = loop_body_gen(self)?;
        self.exit_loop_context();

        Ok(quote! {
            {
                let mut #result_var = None;
                #loop_code
                #result_var
            }
        })
    }

    /// Generates code for a while loop expression.
    /// Returns Option<T> where T is the break value type, or Option<Unit> if no break with value.
    pub(crate) fn generate_while_expr(
        &mut self,
        condition: &RirExpression,
        body: &RirBlock,
        label: &Option<String>,
        _result_type: TypeId,
    ) -> Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let label_clone = label.clone();

        self.wrap_loop_as_expression("__while_result", |generator| {
            let body_stmts = generator.generate_block(body)?;
            let while_loop = quote! {
                while #cond {
                    #body_stmts
                }
            };

            Ok(labels::with_loop_label(&label_clone, while_loop))
        })
    }

    /// Generates code for a for loop expression.
    /// Returns Option<T> where T is the break value type, or Option<Unit> if no break with value.
    pub(crate) fn generate_for_expr(
        &mut self,
        params: ForLoopParams<'_>,
        _result_type: TypeId,
    ) -> Result<TokenStream> {
        let var = format_ident!("{}", params.variable);
        let start_expr = self.generate_expression(params.start)?;
        let end_expr = self.generate_expression(params.end)?;
        let range = labels::generate_range(&start_expr, &end_expr, params.inclusive);
        let label_clone = params.label.clone();

        self.wrap_loop_as_expression("__for_result", |generator| {
            let body_stmts = generator.generate_block(params.body)?;
            let for_loop = quote! {
                for #var in #range {
                    #body_stmts
                }
            };

            Ok(labels::with_loop_label(&label_clone, for_loop))
        })
    }

    /// Generates code for an infinite loop expression.
    /// Returns Option<T> where T is the break value type, or Option<Unit> if no break with value.
    pub(crate) fn generate_loop_expr(
        &mut self,
        body: &RirBlock,
        label: &Option<String>,
    ) -> Result<TokenStream> {
        let label_clone = label.clone();

        self.wrap_loop_as_expression("__loop_result", |generator| {
            let body_stmts = generator.generate_block(body)?;

            Ok(labels::with_label(&label_clone, || {
                quote! {
                    loop {
                        #body_stmts
                    }
                }
            }))
        })
    }

    /// Generates code for a block expression.
    pub(crate) fn generate_block_expr(
        &mut self,
        block: &RirBlock,
        result: &Option<Box<RirExpression>>,
    ) -> Result<TokenStream> {
        let stmts: Result<Vec<_>> = block
            .statements
            .iter()
            .map(|stmt| self.generate_statement(stmt))
            .collect();

        let stmts = stmts?;

        if let Some(result_expr) = result {
            let result_code = self.generate_expression(result_expr)?;
            Ok(quote! {
                {
                    #(#stmts)*
                    #result_code
                }
            })
        } else {
            Ok(quote! {
                {
                    #(#stmts)*
                }
            })
        }
    }
}
