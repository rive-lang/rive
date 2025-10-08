//! Method implementations for RirStatement and RirPattern.

use rive_core::span::Span;

use super::types::{RirPattern, RirStatement};

impl RirPattern {
    /// Returns the span of this pattern
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::IntLiteral { span, .. }
            | Self::FloatLiteral { span, .. }
            | Self::StringLiteral { span, .. }
            | Self::BoolLiteral { span, .. }
            | Self::Wildcard { span }
            | Self::RangePattern { span, .. }
            | Self::EnumVariant { span, .. } => *span,
        }
    }
}

impl RirStatement {
    /// Returns the span of this statement
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Let { span, .. }
            | Self::Assign { span, .. }
            | Self::AssignIndex { span, .. }
            | Self::Return { span, .. }
            | Self::If { span, .. }
            | Self::While { span, .. }
            | Self::For { span, .. }
            | Self::Loop { span, .. }
            | Self::Break { span, .. }
            | Self::Continue { span, .. }
            | Self::Match { span, .. }
            | Self::Expression { span, .. }
            | Self::Block { span, .. } => *span,
        }
    }

    /// Returns true if this is a return statement
    #[must_use]
    pub const fn is_return(&self) -> bool {
        matches!(self, Self::Return { .. })
    }

    /// Returns true if this is a control flow statement
    #[must_use]
    pub const fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Self::If { .. }
                | Self::While { .. }
                | Self::For { .. }
                | Self::Loop { .. }
                | Self::Break { .. }
                | Self::Continue { .. }
                | Self::Match { .. }
        )
    }

    /// Returns true if this is a loop statement
    #[must_use]
    pub const fn is_loop(&self) -> bool {
        matches!(
            self,
            Self::While { .. } | Self::For { .. } | Self::Loop { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RirExpression;
    use rive_core::{
        span::Location,
        type_system::{MemoryStrategy, TypeId},
    };

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_let_statement() {
        let span = dummy_span();
        let stmt = RirStatement::Let {
            name: "x".to_string(),
            type_id: TypeId::INT,
            is_mutable: false,
            value: Box::new(RirExpression::IntLiteral { value: 42, span }),
            memory_strategy: MemoryStrategy::Copy,
            span,
        };
        assert_eq!(stmt.span(), span);
        assert!(!stmt.is_return());
    }

    #[test]
    fn test_return_statement() {
        let span = dummy_span();
        let stmt = RirStatement::Return {
            value: Some(Box::new(RirExpression::IntLiteral { value: 42, span })),
            span,
        };
        assert!(stmt.is_return());
    }

    #[test]
    fn test_control_flow_detection() {
        use crate::RirBlock;
        let span = dummy_span();
        let if_stmt = RirStatement::If {
            condition: Box::new(RirExpression::BoolLiteral { value: true, span }),
            then_block: RirBlock::new(span),
            else_block: None,
            span,
        };
        assert!(if_stmt.is_control_flow());

        let let_stmt = RirStatement::Let {
            name: "x".to_string(),
            type_id: TypeId::INT,
            is_mutable: false,
            value: Box::new(RirExpression::IntLiteral { value: 42, span }),
            memory_strategy: MemoryStrategy::Copy,
            span,
        };
        assert!(!let_stmt.is_control_flow());
    }
}
