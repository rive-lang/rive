//! Expression type checking.
//!
//! This module is split into focused submodules:
//! - `operators`: Binary and unary operator checking
//! - `calls_arrays`: Function calls and array literals
//! - `nullable`: Elvis and safe call operators

mod calls_arrays;
mod nullable;
mod operators;

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::ast::Expression;

impl TypeChecker {
    /// Checks an expression and returns its type.
    ///
    /// This is the main entry point for expression type checking.
    /// It delegates to specialized methods for different expression types.
    pub(crate) fn check_expression(&mut self, expr: &Expression) -> Result<TypeId> {
        match expr {
            // Literals
            Expression::Integer { .. } => Ok(TypeId::INT),
            Expression::Float { .. } => Ok(TypeId::FLOAT),
            Expression::String { .. } => Ok(TypeId::TEXT),
            Expression::Boolean { .. } => Ok(TypeId::BOOL),
            Expression::Null { .. } => Ok(TypeId::NULL),

            // Variables
            Expression::Variable { name, span } => {
                let symbol = self.symbols.lookup(name).ok_or_else(|| {
                    Error::SemanticWithSpan(format!("Undefined variable '{name}'"), *span)
                })?;
                Ok(symbol.symbol_type)
            }

            // Operators
            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => self.check_binary_op(left, operator, right, *span),

            Expression::Unary {
                operator,
                operand,
                span,
            } => self.check_unary_op(operator, operand, *span),

            // Function calls and arrays
            Expression::Call {
                callee,
                arguments,
                span,
            } => self.check_call(callee, arguments, *span),

            Expression::ConstructorCall {
                type_name,
                arguments,
                span,
            } => self.check_constructor_call(type_name, arguments, *span),

            Expression::Array { elements, span } => self.check_array(elements, *span),

            // Control flow expressions
            Expression::If(if_expr) => self.check_if(if_expr, true),
            Expression::While(while_loop) => self.check_while_expr(while_loop),
            Expression::For(for_loop) => self.check_for_expr(for_loop),
            Expression::Loop(loop_expr) => self.check_loop_expr(loop_expr),
            Expression::Match(match_expr) => self.check_match(match_expr, true),
            Expression::Range(_) => {
                // Range expressions are used in for loops - return Unit for now
                Ok(TypeId::UNIT)
            }

            // Block expressions
            Expression::Block(block) => self.check_block_expression(block),

            // Null safety operators
            Expression::Elvis {
                value,
                fallback,
                span,
            } => self.check_elvis(value, fallback, *span),

            Expression::SafeCall { object, call, span } => {
                self.check_safe_call(object, call, *span)
            }

            // New collection literals
            Expression::Tuple { elements, span } => self.check_tuple(elements, *span),
            Expression::List { elements, span } => self.check_list(elements, *span),
            Expression::Dict { entries, span } => self.check_dict(entries, *span),

            // Method calls and field access
            Expression::MethodCall {
                object,
                method,
                arguments,
                span,
            } => self.check_method_call(object, method, arguments, *span),

            Expression::FieldAccess {
                object,
                field,
                span,
            } => self.check_field_access(object, field, *span),
        }
    }

    /// Checks a block expression and returns its type.
    fn check_block_expression(&mut self, block: &rive_parser::Block) -> Result<TypeId> {
        // Check all statements in the block
        for statement in &block.statements {
            self.check_statement(statement)?;
        }

        // Check if there's a final expression
        if let Some(rive_parser::Statement::Expression { expression, .. }) = block.statements.last()
        {
            return self.check_expression(expression);
        }

        // No final expression, block has Unit type
        Ok(TypeId::UNIT)
    }
}
