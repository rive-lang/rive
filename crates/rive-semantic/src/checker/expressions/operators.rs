//! Binary and unary operator type checking.

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result, Span};
use rive_parser::ast::Expression;

impl TypeChecker {
    /// Checks a binary operation.
    pub(super) fn check_binary_op(
        &mut self,
        left: &Expression,
        operator: &rive_parser::BinaryOperator,
        right: &Expression,
        span: Span,
    ) -> Result<TypeId> {
        use rive_parser::BinaryOperator;

        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        // Type compatibility check
        if !self.types_compatible(left_type, right_type) {
            return Err(self.type_mismatch_error(
                "Binary operation type mismatch",
                left_type,
                right_type,
                span,
            ));
        }

        // Determine result type based on operator
        let result_type = match operator {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => left_type,

            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::And
            | BinaryOperator::Or => TypeId::BOOL,
        };

        Ok(result_type)
    }

    /// Checks a unary operation.
    pub(super) fn check_unary_op(
        &mut self,
        operator: &rive_parser::UnaryOperator,
        operand: &Expression,
        span: Span,
    ) -> Result<TypeId> {
        use rive_parser::UnaryOperator;

        let operand_type = self.check_expression(operand)?;

        match operator {
            UnaryOperator::Negate => {
                if !self.types_compatible(operand_type, TypeId::INT)
                    && !self.types_compatible(operand_type, TypeId::FLOAT)
                {
                    let registry = self.symbols.type_registry();
                    let type_str = registry.get_type_name(operand_type);
                    return Err(Error::SemanticWithSpan(
                        format!("Cannot negate type '{type_str}'"),
                        span,
                    ));
                }
                Ok(operand_type)
            }
            UnaryOperator::Not => {
                if !self.types_compatible(operand_type, TypeId::BOOL) {
                    let registry = self.symbols.type_registry();
                    let type_str = registry.get_type_name(operand_type);
                    return Err(Error::SemanticWithSpan(
                        format!("Cannot apply logical NOT to type '{type_str}'"),
                        span,
                    ));
                }
                Ok(TypeId::BOOL)
            }
        }
    }
}

