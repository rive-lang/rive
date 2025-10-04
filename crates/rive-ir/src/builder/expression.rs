//! Helper functions for creating common expressions.

use rive_core::{span::Span, type_system::TypeId};

use crate::{BinaryOp, RirExpression, UnaryOp};

/// Helper functions for creating common expressions
pub struct ExprBuilder;

impl ExprBuilder {
    /// Creates an integer literal
    #[must_use]
    pub fn int(value: i64, span: Span) -> RirExpression {
        RirExpression::IntLiteral { value, span }
    }

    /// Creates a float literal
    #[must_use]
    pub fn float(value: f64, span: Span) -> RirExpression {
        RirExpression::FloatLiteral { value, span }
    }

    /// Creates a string literal
    #[must_use]
    pub fn string(value: String, span: Span) -> RirExpression {
        RirExpression::StringLiteral { value, span }
    }

    /// Creates a boolean literal
    #[must_use]
    pub fn bool(value: bool, span: Span) -> RirExpression {
        RirExpression::BoolLiteral { value, span }
    }

    /// Creates a unit expression
    #[must_use]
    pub fn unit(span: Span) -> RirExpression {
        RirExpression::Unit { span }
    }

    /// Creates a variable reference
    #[must_use]
    pub fn var(name: String, type_id: TypeId, span: Span) -> RirExpression {
        RirExpression::Variable {
            name,
            type_id,
            span,
        }
    }

    /// Creates a binary operation
    #[must_use]
    pub fn binary(
        op: BinaryOp,
        left: RirExpression,
        right: RirExpression,
        result_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            result_type,
            span,
        }
    }

    /// Creates a unary operation
    #[must_use]
    pub fn unary(
        op: UnaryOp,
        operand: RirExpression,
        result_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Unary {
            op,
            operand: Box::new(operand),
            result_type,
            span,
        }
    }

    /// Creates a function call
    #[must_use]
    pub fn call(
        function: String,
        arguments: Vec<RirExpression>,
        return_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Call {
            function,
            arguments,
            return_type,
            span,
        }
    }

    /// Creates an array literal
    #[must_use]
    pub fn array(elements: Vec<RirExpression>, element_type: TypeId, span: Span) -> RirExpression {
        RirExpression::ArrayLiteral {
            elements,
            element_type,
            span,
        }
    }

    /// Creates an array index expression
    #[must_use]
    pub fn index(
        array: RirExpression,
        index: RirExpression,
        element_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Index {
            array: Box::new(array),
            index: Box::new(index),
            element_type,
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_core::span::Location;

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_expr_builder() {
        let span = dummy_span();

        let int_expr = ExprBuilder::int(42, span);
        assert_eq!(int_expr.type_id(), TypeId::INT);

        let bool_expr = ExprBuilder::bool(true, span);
        assert_eq!(bool_expr.type_id(), TypeId::BOOL);

        let var_expr = ExprBuilder::var("x".to_string(), TypeId::INT, span);
        assert_eq!(var_expr.type_id(), TypeId::INT);
    }

    #[test]
    fn test_complex_expression_building() {
        let span = dummy_span();

        // Build: (1 + 2) * 3
        let left = ExprBuilder::binary(
            BinaryOp::Add,
            ExprBuilder::int(1, span),
            ExprBuilder::int(2, span),
            TypeId::INT,
            span,
        );

        let expr = ExprBuilder::binary(
            BinaryOp::Multiply,
            left,
            ExprBuilder::int(3, span),
            TypeId::INT,
            span,
        );

        assert_eq!(expr.type_id(), TypeId::INT);
    }
}
