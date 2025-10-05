//! Method implementations for RirExpression.

use rive_core::{span::Span, type_system::TypeId};

use super::types::RirExpression;

impl RirExpression {
    /// Returns the span of this expression
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::IntLiteral { span, .. }
            | Self::FloatLiteral { span, .. }
            | Self::StringLiteral { span, .. }
            | Self::BoolLiteral { span, .. }
            | Self::Unit { span }
            | Self::Variable { span, .. }
            | Self::Binary { span, .. }
            | Self::Unary { span, .. }
            | Self::Call { span, .. }
            | Self::ArrayLiteral { span, .. }
            | Self::Index { span, .. }
            | Self::If { span, .. }
            | Self::Match { span, .. }
            | Self::Block { span, .. }
            | Self::While { span, .. }
            | Self::For { span, .. }
            | Self::Loop { span, .. }
            | Self::NullLiteral { span, .. }
            | Self::Elvis { span, .. }
            | Self::SafeCall { span, .. }
            | Self::WrapOptional { span, .. } => *span,
        }
    }

    /// Returns the type of this expression
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        match self {
            Self::IntLiteral { .. } => TypeId::INT,
            Self::FloatLiteral { .. } => TypeId::FLOAT,
            Self::StringLiteral { .. } => TypeId::TEXT,
            Self::BoolLiteral { .. } => TypeId::BOOL,
            Self::Unit { .. } => TypeId::UNIT,
            Self::Variable { type_id, .. }
            | Self::Binary {
                result_type: type_id,
                ..
            }
            | Self::Unary {
                result_type: type_id,
                ..
            }
            | Self::Call {
                return_type: type_id,
                ..
            }
            | Self::Index {
                element_type: type_id,
                ..
            }
            | Self::If {
                result_type: type_id,
                ..
            }
            | Self::Match {
                result_type: type_id,
                ..
            }
            | Self::Block {
                result_type: type_id,
                ..
            }
            | Self::While {
                result_type: type_id,
                ..
            }
            | Self::For {
                result_type: type_id,
                ..
            }
            | Self::Loop {
                result_type: type_id,
                ..
            }
            | Self::NullLiteral { type_id, .. }
            | Self::Elvis {
                result_type: type_id,
                ..
            }
            | Self::SafeCall {
                result_type: type_id,
                ..
            }
            | Self::WrapOptional {
                result_type: type_id,
                ..
            } => *type_id,
            Self::ArrayLiteral { element_type, .. } => *element_type,
        }
    }

    /// Returns true if this is a literal expression
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::IntLiteral { .. }
                | Self::FloatLiteral { .. }
                | Self::StringLiteral { .. }
                | Self::BoolLiteral { .. }
                | Self::Unit { .. }
                | Self::NullLiteral { .. }
        )
    }

    /// Returns true if this is a constant expression (literal or constant operation)
    #[must_use]
    pub fn is_constant(&self) -> bool {
        match self {
            Self::IntLiteral { .. }
            | Self::FloatLiteral { .. }
            | Self::StringLiteral { .. }
            | Self::BoolLiteral { .. }
            | Self::Unit { .. }
            | Self::NullLiteral { .. } => true,
            Self::Binary { left, right, .. } => left.is_constant() && right.is_constant(),
            Self::Unary { operand, .. } => operand.is_constant(),
            Self::ArrayLiteral { elements, .. } => elements.iter().all(Self::is_constant),
            Self::Elvis {
                value, fallback, ..
            } => value.is_constant() && fallback.is_constant(),
            Self::WrapOptional { value, .. } => value.is_constant(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BinaryOp;
    use rive_core::span::Location;

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_literal_expressions() {
        let span = dummy_span();
        let int_expr = RirExpression::IntLiteral { value: 42, span };
        assert_eq!(int_expr.type_id(), TypeId::INT);
        assert!(int_expr.is_literal());
        assert!(int_expr.is_constant());

        let bool_expr = RirExpression::BoolLiteral { value: true, span };
        assert_eq!(bool_expr.type_id(), TypeId::BOOL);
        assert!(bool_expr.is_literal());
    }

    #[test]
    fn test_variable_expression() {
        let span = dummy_span();
        let var_expr = RirExpression::Variable {
            name: "x".to_string(),
            type_id: TypeId::INT,
            span,
        };
        assert_eq!(var_expr.type_id(), TypeId::INT);
        assert!(!var_expr.is_literal());
        assert!(!var_expr.is_constant());
    }

    #[test]
    fn test_binary_operation() {
        let span = dummy_span();
        let left = RirExpression::IntLiteral { value: 1, span };
        let right = RirExpression::IntLiteral { value: 2, span };
        let binary = RirExpression::Binary {
            op: BinaryOp::Add,
            left: Box::new(left),
            right: Box::new(right),
            result_type: TypeId::INT,
            span,
        };
        assert_eq!(binary.type_id(), TypeId::INT);
        assert!(binary.is_constant());
    }

    #[test]
    fn test_constant_propagation() {
        let span = dummy_span();

        // Constant expression: 1 + 2
        let const_expr = RirExpression::Binary {
            op: BinaryOp::Add,
            left: Box::new(RirExpression::IntLiteral { value: 1, span }),
            right: Box::new(RirExpression::IntLiteral { value: 2, span }),
            result_type: TypeId::INT,
            span,
        };
        assert!(const_expr.is_constant());

        // Non-constant: x + 2
        let non_const_expr = RirExpression::Binary {
            op: BinaryOp::Add,
            left: Box::new(RirExpression::Variable {
                name: "x".to_string(),
                type_id: TypeId::INT,
                span,
            }),
            right: Box::new(RirExpression::IntLiteral { value: 2, span }),
            result_type: TypeId::INT,
            span,
        };
        assert!(!non_const_expr.is_constant());
    }
}
