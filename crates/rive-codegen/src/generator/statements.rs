//! Statement code generation.

use super::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::Result;
use rive_ir::RirStatement;

impl CodeGenerator {
    /// Generates code for a RIR statement.
    pub(crate) fn generate_statement(&mut self, stmt: &RirStatement) -> Result<TokenStream> {
        match stmt {
            RirStatement::Let {
                name,
                is_mutable,
                value,
                ..
            } => {
                let var_name = format_ident!("{}", name);
                let expr = self.generate_expression(value)?;

                if *is_mutable {
                    Ok(quote! { let mut #var_name = #expr; })
                } else {
                    Ok(quote! { let #var_name = #expr; })
                }
            }
            RirStatement::Assign { name, value, .. } => {
                let var_name = format_ident!("{}", name);
                let expr = self.generate_expression(value)?;
                Ok(quote! { #var_name = #expr; })
            }
            RirStatement::AssignIndex {
                array,
                index,
                value,
                ..
            } => {
                let array_name = format_ident!("{}", array);
                let index_expr = self.generate_expression(index)?;
                let value_expr = self.generate_expression(value)?;
                Ok(quote! { #array_name[#index_expr] = #value_expr; })
            }
            RirStatement::Expression { expr, .. } => {
                let expression = self.generate_expression(expr)?;
                Ok(quote! { #expression; })
            }
            RirStatement::Return { value, .. } => {
                if let Some(expr) = value {
                    let generated_expr = self.generate_expression(expr)?;
                    Ok(quote! { return #generated_expr; })
                } else {
                    Ok(quote! { return; })
                }
            }
            RirStatement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
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
            RirStatement::Block { block, .. } => {
                let body = self.generate_block(block)?;
                Ok(quote! { { #body } })
            }
            RirStatement::While {
                condition,
                body,
                label,
                ..
            } => self.generate_while(condition, body, label),
            RirStatement::For {
                variable,
                start,
                end,
                inclusive,
                body,
                label,
                ..
            } => self.generate_for(variable, start, end, *inclusive, body, label),
            RirStatement::Loop { body, label, .. } => self.generate_loop(body, label),
            RirStatement::Break { label, value, .. } => self.generate_break(label, value),
            RirStatement::Continue { label, .. } => self.generate_continue(label),
            RirStatement::Match {
                scrutinee, arms, ..
            } => self.generate_match_stmt(scrutinee, arms),
        }
    }
}
