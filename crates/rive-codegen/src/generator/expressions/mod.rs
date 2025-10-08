//! Expression code generation.
//!
//! This module provides code generation for all Rive expression types.
//! The functionality is split into focused submodules:
//!
//! - `literals`: Primitive and array literals
//! - `operators`: Binary and unary operations
//! - `collections`: Tuple, List, and Map literals
//! - `methods`: Method call generation and dispatch
//! - `calls`: Function calls (including print formatting)
//! - `nullable`: Null-related operations (Elvis, SafeCall, etc.)
//!
//! Note: Control flow expressions (if, while, for, loop, match) are handled
//! in the parent `control_flow` module.

mod calls;
mod collections;
mod literals;
mod methods;
mod nullable;
mod operators;

use super::core::CodeGenerator;
use proc_macro2::TokenStream;
use rive_core::Result;
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for a RIR expression.
    ///
    /// This is the main dispatch function that routes each expression type
    /// to its specialized generation function.
    pub(crate) fn generate_expression(&mut self, expr: &RirExpression) -> Result<TokenStream> {
        match expr {
            // Literals
            RirExpression::Unit { .. } => self.generate_unit(),
            RirExpression::IntLiteral { value, .. } => self.generate_int_literal(*value),
            RirExpression::FloatLiteral { value, .. } => self.generate_float_literal(*value),
            RirExpression::StringLiteral { value, .. } => self.generate_string_literal(value),
            RirExpression::BoolLiteral { value, .. } => self.generate_bool_literal(*value),
            RirExpression::Variable { name, .. } => self.generate_variable(name),

            // Operators
            RirExpression::Binary {
                op,
                left,
                right,
                result_type,
                ..
            } => self.generate_binary(op, left, right, *result_type),
            RirExpression::Unary { op, operand, .. } => self.generate_unary(op, operand),

            // Function calls
            RirExpression::Call {
                function,
                arguments,
                ..
            } => self.generate_call(function, arguments),

            // Arrays
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
                use crate::generator::control_flow::ForLoopParams;
                let params = ForLoopParams {
                    variable,
                    start,
                    end,
                    inclusive: *inclusive,
                    body,
                    label,
                };
                self.generate_for_expr(params, *result_type)
            }
            RirExpression::Loop { body, label, .. } => self.generate_loop_expr(body, label),

            // Nullable operations
            RirExpression::NullLiteral { .. } => self.generate_null_literal(),
            RirExpression::Elvis {
                value, fallback, ..
            } => self.generate_elvis(value, fallback),
            RirExpression::SafeCall { object, call, .. } => self.generate_safe_call(object, call),
            RirExpression::WrapOptional { value, .. } => self.generate_wrap_optional(value),

            // Collection literals
            RirExpression::TupleLiteral { elements, .. } => self.generate_tuple_literal(elements),
            RirExpression::ListLiteral { elements, .. } => self.generate_list_literal(elements),
            RirExpression::DictLiteral { entries, .. } => self.generate_dict_literal(entries),

            // Method calls and field access
            RirExpression::MethodCall {
                object,
                method,
                arguments,
                return_type,
                ..
            } => self.generate_method_call(object, method, arguments, *return_type),
            RirExpression::FieldAccess { object, field, .. } => {
                self.generate_field_access(object, field)
            }
            RirExpression::ConstructorCall {
                type_id, arguments, ..
            } => self.generate_constructor_call(*type_id, arguments),
            RirExpression::EnumVariant {
                enum_type_id,
                variant_name,
                arguments,
                ..
            } => self.generate_enum_variant(*enum_type_id, variant_name, arguments),
        }
    }
}
