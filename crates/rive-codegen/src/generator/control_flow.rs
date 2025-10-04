//! Control flow code generation.

use super::{core::CodeGenerator, labels, utils};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Result, type_system::TypeId};
use rive_ir::{RirBlock, RirExpression, RirPattern};

impl CodeGenerator {
    /// Generates code for an if expression (must have else).
    pub(crate) fn generate_if_expr(
        &mut self,
        condition: &RirExpression,
        then_block: &RirBlock,
        else_block: &RirBlock,
    ) -> Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let then_body = self.generate_block(then_block)?;
        let else_body = self.generate_block(else_block)?;

        Ok(quote! {
            if #cond {
                #then_body
            } else {
                #else_body
            }
        })
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
    pub(crate) fn generate_for(
        &mut self,
        variable: &str,
        start: &RirExpression,
        end: &RirExpression,
        inclusive: bool,
        body: &RirBlock,
        label: &Option<String>,
    ) -> Result<TokenStream> {
        let var = format_ident!("{variable}");
        let start_expr = self.generate_expression(start)?;
        let end_expr = self.generate_expression(end)?;
        let body_stmts = self.generate_block(body)?;
        let range = labels::generate_range(&start_expr, &end_expr, inclusive);

        Ok(labels::with_label(label, || {
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
        let value_token = value
            .as_ref()
            .map(|expr| self.generate_expression(expr))
            .transpose()?;

        Ok(labels::generate_break_stmt(label, &value_token))
    }

    /// Generates code for a continue statement.
    pub(crate) fn generate_continue(&mut self, label: &Option<String>) -> Result<TokenStream> {
        Ok(labels::generate_continue_stmt(label))
    }

    /// Generates code for a match expression.
    pub(crate) fn generate_match_expr(
        &mut self,
        scrutinee: &RirExpression,
        arms: &[(RirPattern, Box<RirExpression>)],
    ) -> Result<TokenStream> {
        let val = self.generate_expression(scrutinee)?;
        let match_val = self.prepare_match_value(val, scrutinee.type_id());

        let match_arms: Result<Vec<_>> = arms
            .iter()
            .map(|(pattern, body)| {
                let pat = self.generate_pattern(pattern)?;
                let body_expr = self.generate_expression(body)?;
                Ok(quote! { #pat => #body_expr })
            })
            .collect();

        let match_arms = match_arms?;

        Ok(quote! {
            match #match_val {
                #(#match_arms),*
            }
        })
    }

    /// Generates code for a match statement.
    pub(crate) fn generate_match_stmt(
        &mut self,
        scrutinee: &RirExpression,
        arms: &[(RirPattern, rive_ir::RirBlock)],
    ) -> Result<TokenStream> {
        let val = self.generate_expression(scrutinee)?;
        let match_val = self.prepare_match_value(val, scrutinee.type_id());

        let match_arms: Result<Vec<_>> = arms
            .iter()
            .map(|(pattern, body)| {
                let pat = self.generate_pattern(pattern)?;
                let body_stmts = self.generate_block(body)?;
                Ok(quote! {
                    #pat => {
                        #body_stmts
                    }
                })
            })
            .collect();

        let match_arms = match_arms?;

        Ok(quote! {
            match #match_val {
                #(#match_arms),*
            }
        })
    }

    /// Prepares a value for matching (converts Text to &str).
    fn prepare_match_value(&self, val: TokenStream, type_id: TypeId) -> TokenStream {
        if type_id == TypeId::TEXT {
            quote! { (#val).as_str() }
        } else {
            val
        }
    }

    /// Generates code for a pattern.
    fn generate_pattern(&mut self, pattern: &RirPattern) -> Result<TokenStream> {
        Ok(match pattern {
            RirPattern::IntLiteral { value, .. } => {
                let lit = proc_macro2::Literal::i64_unsuffixed(*value);
                quote! { #lit }
            }
            RirPattern::FloatLiteral { value, .. } => {
                let lit = proc_macro2::Literal::f64_unsuffixed(*value);
                quote! { #lit }
            }
            RirPattern::StringLiteral { value, .. } => {
                let lit = proc_macro2::Literal::string(value);
                quote! { #lit }
            }
            RirPattern::BoolLiteral { value, .. } => {
                quote! { #value }
            }
            RirPattern::Wildcard { .. } => {
                quote! { _ }
            }
            RirPattern::RangePattern {
                start,
                end,
                inclusive,
                ..
            } => {
                let start_expr = self.generate_expression(start)?;
                let end_expr = self.generate_expression(end)?;
                labels::generate_range(&start_expr, &end_expr, *inclusive)
            }
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

    /// Generates code for a while loop expression.
    pub(crate) fn generate_while_expr(
        &mut self,
        condition: &RirExpression,
        body: &RirBlock,
        label: &Option<String>,
        result_type: TypeId,
    ) -> Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let body_stmts = self.generate_block(body)?;
        let default_value = utils::generate_default_value(result_type);

        let break_stmt = labels::generate_break_stmt(label, &default_value);

        Ok(labels::with_label(label, || {
            quote! {
                loop {
                    if !(#cond) {
                        #break_stmt;
                    }
                    #body_stmts
                }
            }
        }))
    }

    /// Generates code for a for loop expression.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn generate_for_expr(
        &mut self,
        variable: &str,
        start: &RirExpression,
        end: &RirExpression,
        inclusive: bool,
        body: &RirBlock,
        label: &Option<String>,
        result_type: TypeId,
    ) -> Result<TokenStream> {
        let var = format_ident!("{variable}");
        let start_expr = self.generate_expression(start)?;
        let end_expr = self.generate_expression(end)?;
        let body_stmts = self.generate_block(body)?;
        let default_value = utils::generate_default_value(result_type);
        let range = labels::generate_range_iterator(&start_expr, &end_expr, inclusive);

        let break_stmt = labels::generate_break_stmt(label, &default_value);

        let loop_body = labels::with_label(label, || {
            quote! {
                loop {
                    let #var = match iter.next() {
                        Some(val) => val,
                        None => { #break_stmt; }
                    };
                    #body_stmts
                }
            }
        });

        Ok(quote! {
            {
                let mut iter = #range;
                #loop_body
            }
        })
    }

    /// Generates code for an infinite loop expression.
    pub(crate) fn generate_loop_expr(
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
}
