//! Statement code generation.
//!
//! This module handles generation of Rive statements.

mod control;
mod variables;

use super::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::quote;
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
            } => self.generate_let(name, *is_mutable, value),

            RirStatement::Assign { name, value, .. } => self.generate_assign(name, value),

            RirStatement::AssignIndex {
                array,
                index,
                value,
                ..
            } => self.generate_assign_index(array, index, value),

            RirStatement::Expression { expr, .. } => {
                // Special handling: if expression is a loop, generate as statement without return value
                if expr.is_loop() {
                    self.generate_loop_stmt(expr)
                } else {
                    let expression = self.generate_expression(expr)?;
                    Ok(quote! { #expression; })
                }
            }

            RirStatement::Return { value, .. } => self.generate_return(value.as_deref()),

            RirStatement::If {
                condition,
                then_block,
                else_block,
                ..
            } => self.generate_if(condition, then_block, else_block.as_ref()),

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
            } => {
                use crate::generator::control_flow::ForLoopParams;
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

            RirStatement::Loop { body, label, .. } => self.generate_loop(body, label),

            RirStatement::Break { label, value, .. } => self.generate_break(label, value),

            RirStatement::Continue { label, .. } => self.generate_continue(label),

            RirStatement::Match {
                scrutinee, arms, ..
            } => self.generate_match_stmt(scrutinee, arms),
        }
    }
}
