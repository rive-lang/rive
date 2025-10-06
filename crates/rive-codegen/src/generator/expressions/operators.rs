//! Binary and unary operator code generation.
//!
//! This module handles:
//! - Binary operations (arithmetic, comparison, logical)
//! - Unary operations (negation, logical not)
//! - Operator precedence and parenthesization

use super::super::{core::CodeGenerator, utils};
use proc_macro2::TokenStream;
use quote::quote;
use rive_core::{Result, type_system::TypeId};
use rive_ir::{BinaryOp, RirExpression, UnaryOp};

impl CodeGenerator {
    /// Generates code for a binary operation.
    pub(crate) fn generate_binary(
        &mut self,
        op: &BinaryOp,
        left: &RirExpression,
        right: &RirExpression,
        result_type: TypeId,
    ) -> Result<TokenStream> {
        // Special handling for string concatenation
        if *op == BinaryOp::Add && result_type == TypeId::TEXT {
            let left_expr = self.generate_expression(left)?;
            let right_expr = self.generate_expression(right)?;
            return Ok(quote! { format!("{}{}", #left_expr, #right_expr) });
        }

        let left_expr = self.generate_binary_operand(left, op, true)?;
        let right_expr = self.generate_binary_operand(right, op, false)?;
        let operator = utils::binary_op_token(op);

        Ok(quote! { #left_expr #operator #right_expr })
    }

    /// Generates code for a binary operation operand, adding parentheses if needed.
    fn generate_binary_operand(
        &mut self,
        operand: &RirExpression,
        parent_op: &BinaryOp,
        is_left: bool,
    ) -> Result<TokenStream> {
        if let RirExpression::Binary { op: child_op, .. } = operand {
            let parent_prec = utils::operator_precedence(parent_op);
            let child_prec = utils::operator_precedence(child_op);

            let needs_parens = child_prec < parent_prec
                || (child_prec == parent_prec
                    && !is_left
                    && !utils::is_right_associative(parent_op));

            let expr = self.generate_expression(operand)?;
            if needs_parens {
                return Ok(quote! { (#expr) });
            }
            return Ok(expr);
        }

        self.generate_expression(operand)
    }

    /// Generates code for a unary operation.
    pub(crate) fn generate_unary(
        &mut self,
        op: &UnaryOp,
        operand: &RirExpression,
    ) -> Result<TokenStream> {
        use quote::TokenStreamExt;

        // For literals, directly generate negative/not literal without parentheses
        // This avoids unwanted spaces like "- 1" and makes output cleaner
        match (op, operand) {
            (UnaryOp::Negate, RirExpression::IntLiteral { value, .. }) => {
                let lit = proc_macro2::Literal::i64_unsuffixed(-value);
                Ok(quote! { #lit })
            }
            (UnaryOp::Negate, RirExpression::FloatLiteral { value, .. }) => {
                let lit = proc_macro2::Literal::f64_suffixed(-value);
                Ok(quote! { #lit })
            }
            // For all other cases, generate operator without spaces using TokenStream
            _ => {
                let operand_expr = self.generate_expression(operand)?;
                let op_char = match op {
                    UnaryOp::Negate => '-',
                    UnaryOp::Not => '!',
                };

                let mut tokens = proc_macro2::TokenStream::new();
                tokens.append(proc_macro2::Punct::new(
                    op_char,
                    proc_macro2::Spacing::Alone,
                ));
                tokens.extend(operand_expr);
                Ok(tokens)
            }
        }
    }
}
