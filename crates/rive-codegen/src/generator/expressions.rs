//! Expression code generation.

use super::{core::CodeGenerator, utils};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Error, Result, type_system::TypeId};
use rive_ir::{BinaryOp, RirExpression, UnaryOp};

impl CodeGenerator {
    /// Generates code for a RIR expression.
    pub(crate) fn generate_expression(&mut self, expr: &RirExpression) -> Result<TokenStream> {
        match expr {
            RirExpression::Unit { .. } => Ok(quote! { () }),
            RirExpression::IntLiteral { value, .. } => {
                let lit = proc_macro2::Literal::i64_unsuffixed(*value);
                Ok(quote! { #lit })
            }
            RirExpression::FloatLiteral { value, .. } => {
                let lit = proc_macro2::Literal::f64_unsuffixed(*value);
                Ok(quote! { #lit })
            }
            RirExpression::StringLiteral { value, .. } => {
                let lit = proc_macro2::Literal::string(value);
                Ok(quote! { #lit.to_string() })
            }
            RirExpression::BoolLiteral { value, .. } => Ok(quote! { #value }),
            RirExpression::Variable { name, .. } => {
                let var_name = format_ident!("{}", name);
                Ok(quote! { #var_name })
            }
            RirExpression::Binary {
                op,
                left,
                right,
                result_type,
                ..
            } => self.generate_binary(op, left, right, *result_type),
            RirExpression::Unary { op, operand, .. } => self.generate_unary(op, operand),
            RirExpression::Call {
                function,
                arguments,
                ..
            } => self.generate_call(function, arguments),
            RirExpression::ArrayLiteral { elements, .. } => self.generate_array_literal(elements),
            RirExpression::Index { array, index, .. } => self.generate_index(array, index),
            // Control flow expressions
            RirExpression::If {
                condition,
                then_block,
                else_block,
                ..
            } => self.generate_if_expr(condition, then_block, else_block),
            RirExpression::Match {
                scrutinee, arms, ..
            } => self.generate_match_expr(scrutinee, arms),
            RirExpression::Block { block, result, .. } => self.generate_block_expr(block, result),
            RirExpression::While {
                condition,
                body,
                label,
                result_type,
                ..
            } => self.generate_while_expr(condition, body, label, *result_type),
            RirExpression::For {
                variable,
                start,
                end,
                inclusive,
                body,
                label,
                result_type,
                ..
            } => {
                self.generate_for_expr(variable, start, end, *inclusive, body, label, *result_type)
            }
            RirExpression::Loop { body, label, .. } => self.generate_loop_expr(body, label),

            // Nullable operations
            RirExpression::NullLiteral { .. } => Ok(quote! { None }),
            RirExpression::Elvis {
                value, fallback, ..
            } => self.generate_elvis(value, fallback),
            RirExpression::SafeCall { object, call, .. } => self.generate_safe_call(object, call),
            RirExpression::WrapOptional { value, .. } => {
                let value_expr = self.generate_expression(value)?;
                Ok(quote! { Some(#value_expr) })
            }
        }
    }

    /// Generates code for a binary operation.
    fn generate_binary(
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
    fn generate_unary(&mut self, op: &UnaryOp, operand: &RirExpression) -> Result<TokenStream> {
        let operand_expr = self.generate_expression(operand)?;
        let operator = match op {
            UnaryOp::Negate => quote! { - },
            UnaryOp::Not => quote! { ! },
        };
        Ok(quote! { (#operator #operand_expr) })
    }

    /// Generates code for a function call.
    fn generate_call(
        &mut self,
        function: &str,
        arguments: &[RirExpression],
    ) -> Result<TokenStream> {
        // Special handling for print function
        if function == "print" {
            if arguments.is_empty() {
                return Err(Error::Codegen(
                    "print() requires at least one argument".to_string(),
                ));
            }

            let args = arguments
                .iter()
                .map(|arg| self.generate_expression(arg))
                .collect::<Result<Vec<_>>>()?;

            let format_parts: Vec<String> = arguments
                .iter()
                .map(|arg| {
                    if arg.type_id() == TypeId::TEXT {
                        "{}".to_string()
                    } else {
                        "{:?}".to_string()
                    }
                })
                .collect();

            let format_str = format_parts.join("");
            return Ok(quote! { println!(#format_str, #(#args),*) });
        }

        let func_name = format_ident!("{}", function);
        let args = arguments
            .iter()
            .map(|arg| self.generate_expression(arg))
            .collect::<Result<Vec<_>>>()?;

        Ok(quote! { #func_name(#(#args),*) })
    }

    /// Generates code for an array literal.
    fn generate_array_literal(&mut self, elements: &[RirExpression]) -> Result<TokenStream> {
        let elems = elements
            .iter()
            .map(|elem| self.generate_expression(elem))
            .collect::<Result<Vec<_>>>()?;

        Ok(quote! { [#(#elems),*] })
    }

    /// Generates code for array indexing.
    fn generate_index(
        &mut self,
        array: &RirExpression,
        index: &RirExpression,
    ) -> Result<TokenStream> {
        let array_expr = self.generate_expression(array)?;
        let index_expr = self.generate_expression(index)?;
        Ok(quote! { #array_expr[#index_expr] })
    }

    /// Generates code for Elvis operator (null-coalescing).
    ///
    /// # Example
    /// `value ?: fallback` compiles to:
    /// - `value.unwrap_or_else(|| fallback)` if fallback is an expression
    /// - `value.unwrap_or(fallback)` if fallback is a simple value
    fn generate_elvis(
        &mut self,
        value: &RirExpression,
        fallback: &RirExpression,
    ) -> Result<TokenStream> {
        let value_expr = self.generate_expression(value)?;
        let fallback_expr = self.generate_expression(fallback)?;

        // Check if fallback is a simple literal or variable
        // If so, use unwrap_or, otherwise use unwrap_or_else
        let is_simple = matches!(
            fallback,
            RirExpression::IntLiteral { .. }
                | RirExpression::FloatLiteral { .. }
                | RirExpression::StringLiteral { .. }
                | RirExpression::BoolLiteral { .. }
                | RirExpression::Variable { .. }
                | RirExpression::NullLiteral { .. }
        );

        if is_simple {
            Ok(quote! { #value_expr.unwrap_or(#fallback_expr) })
        } else {
            Ok(quote! { #value_expr.unwrap_or_else(|| #fallback_expr) })
        }
    }

    /// Generates code for Safe Call operator.
    ///
    /// # Example
    /// `object?.method()` compiles to:
    /// - `object.and_then(|obj| /* rewrite method() to use obj */)`
    ///
    /// # Note
    /// Currently, we use a simplified approach. The call expression
    /// is evaluated independently, but it should reference the object.
    /// A more sophisticated approach would rewrite the call to use
    /// the unwrapped object value.
    fn generate_safe_call(
        &mut self,
        object: &RirExpression,
        call: &RirExpression,
    ) -> Result<TokenStream> {
        let object_expr = self.generate_expression(object)?;
        let call_expr = self.generate_expression(call)?;

        // Generate: object.map(|_| call)
        // In a real implementation, we'd need to rewrite the call to use the unwrapped object
        // For now, we assume the call is self-contained
        Ok(quote! { #object_expr.and_then(|_obj| Some(#call_expr)) })
    }
}
