//! Abstract Syntax Tree (AST) definitions for Rive.

use rive_core::Span;
use rive_core::type_system::TypeId;

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
    pub return_type: TypeId,
    pub body: Block,
    pub span: Span,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: TypeId,
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
    /// Variable declaration: `let [mut] name[?] [: type[?]] = expr`
    Let {
        name: String,
        mutable: bool,
        var_type: Option<TypeId>,
        /// Whether to infer as nullable when no explicit type (e.g., `let result? = ...`)
        infer_nullable: bool,
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

    /// Break statement: `break [depth] [value]`
    Break(crate::control_flow::Break),

    /// Continue statement: `continue [depth]`
    Continue(crate::control_flow::Continue),
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

    /// If expression: `if cond { ... } else { ... }`
    If(Box<crate::control_flow::If>),

    /// While loop: `while cond { ... }`
    While(Box<crate::control_flow::While>),

    /// For loop: `for var in iterable { ... }`
    For(Box<crate::control_flow::For>),

    /// Infinite loop: `loop { ... }`
    Loop(Box<crate::control_flow::Loop>),

    /// Match expression: `match expr { pattern -> expr, ... }`
    Match(Box<crate::control_flow::Match>),

    /// Range expression: `start..end` or `start..=end`
    Range(Box<crate::control_flow::Range>),

    /// Block expression: `{ statements... }`
    Block(Box<Block>),

    /// Elvis operator (null-coalescing): `value ?: fallback`
    ///
    /// Returns `value` if non-null, otherwise evaluates and returns `fallback`.
    Elvis {
        value: Box<Expression>,
        fallback: Box<Expression>,
        span: Span,
    },

    /// Safe call operator: `object?.method()` or `object?.field`
    ///
    /// Evaluates to null if `object` is null, otherwise calls the method/accesses field.
    SafeCall {
        object: Box<Expression>,
        call: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    /// Returns the span of this expression.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Integer { span, .. } => *span,
            Self::Float { span, .. } => *span,
            Self::String { span, .. } => *span,
            Self::Boolean { span, .. } => *span,
            Self::Null { span } => *span,
            Self::Variable { span, .. } => *span,
            Self::Binary { span, .. } => *span,
            Self::Unary { span, .. } => *span,
            Self::Call { span, .. } => *span,
            Self::Array { span, .. } => *span,
            Self::If(expr) => expr.span,
            Self::While(expr) => expr.span,
            Self::For(expr) => expr.span,
            Self::Loop(expr) => expr.span,
            Self::Match(expr) => expr.span,
            Self::Range(expr) => expr.span,
            Self::Block(block) => block.span,
            Self::Elvis { span, .. } => *span,
            Self::SafeCall { span, .. } => *span,
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
