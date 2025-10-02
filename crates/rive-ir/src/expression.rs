//! RIR expression types.

use rive_core::{span::Span, type_system::TypeId};

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

/// An expression in RIR
#[derive(Debug, Clone)]
pub enum RirExpression {
    /// Integer literal
    IntLiteral { value: i64, span: Span },

    /// Float literal
    FloatLiteral { value: f64, span: Span },

    /// String literal
    StringLiteral { value: String, span: Span },

    /// Boolean literal
    BoolLiteral { value: bool, span: Span },

    /// Unit literal ()
    Unit { span: Span },

    /// Variable reference
    Variable {
        name: String,
        type_id: TypeId,
        span: Span,
    },

    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<RirExpression>,
        right: Box<RirExpression>,
        result_type: TypeId,
        span: Span,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        operand: Box<RirExpression>,
        result_type: TypeId,
        span: Span,
    },

    /// Function call
    Call {
        function: String,
        arguments: Vec<RirExpression>,
        return_type: TypeId,
        span: Span,
    },

    /// Array literal
    ArrayLiteral {
        elements: Vec<RirExpression>,
        element_type: TypeId,
        span: Span,
    },

    /// Array indexing
    Index {
        array: Box<RirExpression>,
        index: Box<RirExpression>,
        element_type: TypeId,
        span: Span,
    },
}

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
            | Self::Index { span, .. } => *span,
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
            | Self::Unit { .. } => true,
            Self::Binary { left, right, .. } => left.is_constant() && right.is_constant(),
            Self::Unary { operand, .. } => operand.is_constant(),
            Self::ArrayLiteral { elements, .. } => elements.iter().all(Self::is_constant),
            _ => false,
        }
    }
}

impl BinaryOp {
    /// Returns true if this is an arithmetic operator
    #[must_use]
    pub const fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Modulo
        )
    }

    /// Returns true if this is a comparison operator
    #[must_use]
    pub const fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Equal
                | Self::NotEqual
                | Self::LessThan
                | Self::LessEqual
                | Self::GreaterThan
                | Self::GreaterEqual
        )
    }

    /// Returns true if this is a logical operator
    #[must_use]
    pub const fn is_logical(self) -> bool {
        matches!(self, Self::And | Self::Or)
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
    fn test_binary_op_categories() {
        assert!(BinaryOp::Add.is_arithmetic());
        assert!(!BinaryOp::Add.is_comparison());
        assert!(!BinaryOp::Add.is_logical());

        assert!(!BinaryOp::Equal.is_arithmetic());
        assert!(BinaryOp::Equal.is_comparison());
        assert!(!BinaryOp::Equal.is_logical());

        assert!(!BinaryOp::And.is_arithmetic());
        assert!(!BinaryOp::And.is_comparison());
        assert!(BinaryOp::And.is_logical());
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
