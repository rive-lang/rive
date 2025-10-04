//! Binary and unary operators.

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
}
