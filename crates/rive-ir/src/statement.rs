//! RIR statement types.

use rive_core::{
    span::Span,
    type_system::{MemoryStrategy, TypeId},
};

use crate::{RirBlock, RirExpression};

/// A statement in RIR
#[derive(Debug, Clone)]
pub enum RirStatement {
    /// Variable declaration with initialization
    Let {
        /// Variable name
        name: String,
        /// Variable type
        type_id: TypeId,
        /// Whether the variable is mutable
        is_mutable: bool,
        /// Initial value
        value: Box<RirExpression>,
        /// Memory strategy for this variable
        memory_strategy: MemoryStrategy,
        /// Source location
        span: Span,
    },

    /// Assignment to an existing variable
    Assign {
        /// Variable name
        name: String,
        /// New value
        value: Box<RirExpression>,
        /// Source location
        span: Span,
    },

    /// Array element assignment
    AssignIndex {
        /// Array variable name
        array: String,
        /// Index expression
        index: Box<RirExpression>,
        /// New value
        value: Box<RirExpression>,
        /// Source location
        span: Span,
    },

    /// Return statement
    Return {
        /// Optional return value
        value: Option<Box<RirExpression>>,
        /// Source location
        span: Span,
    },

    /// If statement (conditional)
    If {
        /// Condition expression
        condition: Box<RirExpression>,
        /// Then block
        then_block: RirBlock,
        /// Optional else block
        else_block: Option<RirBlock>,
        /// Source location
        span: Span,
    },

    /// While loop
    While {
        /// Loop condition
        condition: Box<RirExpression>,
        /// Loop body
        body: RirBlock,
        /// Source location
        span: Span,
    },

    /// Standalone expression statement
    Expression {
        /// Expression to evaluate
        expr: Box<RirExpression>,
        /// Source location
        span: Span,
    },

    /// Block statement
    Block {
        /// Inner block
        block: RirBlock,
        /// Source location
        span: Span,
    },
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
            | Self::Expression { span, .. }
            | Self::Block { span, .. } => *span,
        }
    }

    /// Returns true if this is a return statement
    #[must_use]
    pub const fn is_return(&self) -> bool {
        matches!(self, Self::Return { .. })
    }

    /// Returns true if this is a control flow statement (if, while)
    #[must_use]
    pub const fn is_control_flow(&self) -> bool {
        matches!(self, Self::If { .. } | Self::While { .. })
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
