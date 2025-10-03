//! Control flow AST nodes for Rive.
//!
//! This module contains AST node definitions for all control flow constructs:
//! - If expressions/statements
//! - Loop constructs (while, for, loop)
//! - Match expressions
//! - Break and continue statements
//! - Patterns for match expressions

use rive_core::Span;

use crate::ast::{Block, Expression};

/// If expression/statement with optional else-if chain and else block.
///
/// Can be used as both expression and statement:
/// - As expression: All branches must exist and have same type
/// - As statement: Returns Unit type
#[derive(Debug, Clone, PartialEq)]
pub struct If {
    /// The main condition
    pub condition: Box<Expression>,
    /// The main then block
    pub then_block: Block,
    /// Optional else-if branches
    pub else_if_branches: Vec<ElseIf>,
    /// Optional final else block
    pub else_block: Option<Block>,
    /// Source span
    pub span: Span,
}

/// Else-if branch in an if expression/statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ElseIf {
    /// The condition for this branch
    pub condition: Box<Expression>,
    /// The block to execute if condition is true
    pub block: Block,
    /// Source span
    pub span: Span,
}

/// While loop - conditional loop with test before each iteration.
///
/// Can return a value via `break value`.
#[derive(Debug, Clone, PartialEq)]
pub struct While {
    /// Loop condition
    pub condition: Box<Expression>,
    /// Loop body
    pub body: Block,
    /// Source span
    pub span: Span,
}

/// For loop - iteration over ranges (and eventually collections).
///
/// Currently supports ranges only (e.g., `1..10`, `1..=10`).
/// Can return a value via `break value`.
#[derive(Debug, Clone, PartialEq)]
pub struct For {
    /// Iterator variable name
    pub variable: String,
    /// Range/iterable expression
    pub iterable: Box<Expression>,
    /// Loop body
    pub body: Block,
    /// Source span
    pub span: Span,
}

/// Loop - infinite loop construct.
///
/// Executes until a `break` statement is encountered.
/// Can return a value via `break value`.
#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    /// Loop body
    pub body: Block,
    /// Source span
    pub span: Span,
}

/// Break statement - exits from loop(s).
///
/// - `break` - exits innermost loop (returns Unit)
/// - `break 2` - exits 2 nested loops (returns Unit)
/// - `break with value` - exits innermost loop and returns value
/// - `break 2 with value` - exits 2 nested loops and returns value
#[derive(Debug, Clone, PartialEq)]
pub struct Break {
    /// Number of loop levels to break (None = 1 = innermost)
    pub depth: Option<u32>,
    /// Optional value to return from the loop
    pub value: Option<Box<Expression>>,
    /// Source span
    pub span: Span,
}

/// Continue statement - skips to next iteration.
///
/// - `continue` - continues innermost loop
/// - `continue 2` - continues outer loop (skip 2 levels)
#[derive(Debug, Clone, PartialEq)]
pub struct Continue {
    /// Number of loop levels to skip (None = 1 = innermost)
    pub depth: Option<u32>,
    /// Source span
    pub span: Span,
}

/// Match expression - pattern matching.
///
/// Always returns a value (type of all arms must match).
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    /// Expression being matched
    pub scrutinee: Box<Expression>,
    /// Match arms (pattern -> expression)
    pub arms: Vec<MatchArm>,
    /// Source span
    pub span: Span,
}

/// A single arm in a match expression.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    /// Pattern to match against
    pub pattern: Pattern,
    /// Expression to evaluate if pattern matches
    pub body: Box<Expression>,
    /// Source span
    pub span: Span,
}

/// Patterns for match expressions.
///
/// Phase 1: Simple literal patterns and wildcard
/// Future: Destructuring, guards, ranges, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Integer literal pattern: `42`
    Integer { value: i64, span: Span },

    /// Float literal pattern: `3.14`
    Float { value: f64, span: Span },

    /// String literal pattern: `"hello"`
    String { value: String, span: Span },

    /// Boolean literal pattern: `true` or `false`
    Boolean { value: bool, span: Span },

    /// Null pattern: `null`
    Null { span: Span },

    /// Wildcard pattern: `_` (matches anything)
    Wildcard { span: Span },

    /// Range pattern: `in start..end` or `in start..=end`
    Range { 
        start: Box<Expression>, 
        end: Box<Expression>, 
        inclusive: bool, 
        span: Span 
    },
    // Future patterns (not implemented yet):
    // Variable { name: String, span: Span },
    // Tuple { elements: Vec<Pattern>, span: Span },
    // Range { start: i64, end: i64, inclusive: bool, span: Span },
}

impl Pattern {
    /// Returns the span of this pattern.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Integer { span, .. }
            | Self::Float { span, .. }
            | Self::String { span, .. }
            | Self::Boolean { span, .. }
            | Self::Null { span }
            | Self::Wildcard { span }
            | Self::Range { span, .. } => *span,
        }
    }
}

/// Range expression for use in for loops.
///
/// Represents both exclusive (`..`) and inclusive (`..=`) ranges.
#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    /// Start of range (inclusive)
    pub start: Box<Expression>,
    /// End of range
    pub end: Box<Expression>,
    /// Whether the range is inclusive of end value
    pub inclusive: bool,
    /// Source span
    pub span: Span,
}
