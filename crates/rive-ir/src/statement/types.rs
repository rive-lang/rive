//! RIR statement and pattern type definitions.

use rive_core::{
    span::Span,
    type_system::{MemoryStrategy, TypeId},
};

use crate::{RirBlock, RirExpression};

/// A pattern in RIR (used in match expressions)
#[derive(Debug, Clone)]
pub enum RirPattern {
    /// Integer literal pattern
    IntLiteral { value: i64, span: Span },
    /// Float literal pattern
    FloatLiteral { value: f64, span: Span },
    /// String literal pattern
    StringLiteral { value: String, span: Span },
    /// Boolean literal pattern
    BoolLiteral { value: bool, span: Span },
    /// Wildcard pattern (_)
    Wildcard { span: Span },

    /// Range pattern (in start..end or in start..=end)
    RangePattern {
        start: Box<RirExpression>,
        end: Box<RirExpression>,
        inclusive: bool,
        span: Span,
    },
}

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
        /// Optional label for multi-level break/continue
        label: Option<String>,
        /// Source location
        span: Span,
    },

    /// For loop (range iteration)
    For {
        /// Iterator variable name
        variable: String,
        /// Start of range
        start: Box<RirExpression>,
        /// End of range
        end: Box<RirExpression>,
        /// Whether range is inclusive (..=) or exclusive (..)
        inclusive: bool,
        /// Loop body
        body: RirBlock,
        /// Optional label for multi-level break/continue
        label: Option<String>,
        /// Source location
        span: Span,
    },

    /// Infinite loop
    Loop {
        /// Loop body
        body: RirBlock,
        /// Optional label for multi-level break/continue
        label: Option<String>,
        /// Source location
        span: Span,
    },

    /// Break statement
    Break {
        /// Target label for multi-level break
        label: Option<String>,
        /// Optional value to return from loop
        value: Option<Box<RirExpression>>,
        /// Source location
        span: Span,
    },

    /// Continue statement
    Continue {
        /// Target label for multi-level continue
        label: Option<String>,
        /// Source location
        span: Span,
    },

    /// Match statement (converted to if-else chain in Phase 1)
    Match {
        /// Value to match against
        scrutinee: Box<RirExpression>,
        /// Match arms (pattern, body)
        arms: Vec<(RirPattern, RirBlock)>,
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
