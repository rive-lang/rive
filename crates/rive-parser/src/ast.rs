//! Abstract Syntax Tree (AST) definitions for Rive.

use rive_core::Span;
use rive_core::types::Type;

/// A complete Rive program (compilation unit).
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items in a Rive program.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
}

/// Function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Block,
    pub span: Span,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub span: Span,
}

/// A block of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// Statements in Rive.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable declaration: `let [mut] name [: type] = expr`
    Let {
        name: String,
        mutable: bool,
        var_type: Option<Type>,
        initializer: Expression,
        span: Span,
    },

    /// Assignment statement: `name = expr`
    Assignment {
        name: String,
        value: Expression,
        span: Span,
    },

    /// Expression statement
    Expression { expression: Expression, span: Span },

    /// Return statement: `return [expr]`
    Return {
        value: Option<Expression>,
        span: Span,
    },
}

/// Expressions in Rive.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal
    Integer { value: i64, span: Span },

    /// Float literal
    Float { value: f64, span: Span },

    /// String literal
    String { value: String, span: Span },

    /// Boolean literal
    Boolean { value: bool, span: Span },

    /// Null literal
    Null { span: Span },

    /// Variable reference
    Variable { name: String, span: Span },

    /// Binary operation: `left op right`
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },

    /// Unary operation: `op expr`
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },

    /// Function call: `name(args...)`
    Call {
        callee: String,
        arguments: Vec<Expression>,
        span: Span,
    },

    /// Array literal: `[expr, ...]`
    Array {
        elements: Vec<Expression>,
        span: Span,
    },
}

impl Expression {
    /// Returns the span of this expression.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Integer { span, .. }
            | Self::Float { span, .. }
            | Self::String { span, .. }
            | Self::Boolean { span, .. }
            | Self::Null { span }
            | Self::Variable { span, .. }
            | Self::Binary { span, .. }
            | Self::Unary { span, .. }
            | Self::Call { span, .. }
            | Self::Array { span, .. } => *span,
        }
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}
