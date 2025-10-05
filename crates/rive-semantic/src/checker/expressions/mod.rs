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

            Expression::Array { elements, span } => self.check_array(elements, *span),

            // Control flow expressions
            Expression::If(_) => {
                // TODO: Implement type checking for if expressions
                Ok(TypeId::UNIT)
            }
            Expression::While(_) => {
                // TODO: Implement type checking for while expressions
                Ok(TypeId::UNIT)
            }
            Expression::For(_) => {
                // TODO: Implement type checking for for expressions
                Ok(TypeId::UNIT)
            }
            Expression::Loop(_) => {
                // TODO: Implement type checking for loop expressions
                Ok(TypeId::UNIT)
            }
            Expression::Match(_) => {
                // TODO: Implement type checking for match expressions
                Ok(TypeId::UNIT)
            }
            Expression::Range(_) => {
                // TODO: Implement type checking for range expressions
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

            Expression::SafeCall {
                object,
                call,
                span,
            } => self.check_safe_call(object, call, *span),
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

