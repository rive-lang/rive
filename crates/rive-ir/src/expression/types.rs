//! RIR expression type definitions.

use rive_core::{span::Span, type_system::TypeId};

use crate::{RirBlock, RirPattern};

use super::operators::{BinaryOp, UnaryOp};

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

    /// If expression (must have else branch to be an expression)
    If {
        condition: Box<RirExpression>,
        then_block: RirBlock,
        else_block: RirBlock,
        result_type: TypeId,
        span: Span,
    },

    /// Match expression
    Match {
        scrutinee: Box<RirExpression>,
        arms: Vec<(RirPattern, Box<RirExpression>)>,
        result_type: TypeId,
        span: Span,
    },

    /// Block expression with result
    Block {
        block: RirBlock,
        result: Option<Box<RirExpression>>,
        result_type: TypeId,
        span: Span,
    },

    /// While loop expression (can break with value)
    While {
        condition: Box<RirExpression>,
        body: RirBlock,
        label: Option<String>,
        result_type: TypeId,
        span: Span,
    },

    /// For loop expression (can break with value)
    For {
        variable: String,
        start: Box<RirExpression>,
        end: Box<RirExpression>,
        inclusive: bool,
        body: RirBlock,
        label: Option<String>,
        result_type: TypeId,
        span: Span,
    },

    /// Infinite loop expression (can break with value)
    Loop {
        body: RirBlock,
        label: Option<String>,
        result_type: TypeId,
        span: Span,
    },

    /// Null literal (None in Rust)
    NullLiteral { type_id: TypeId, span: Span },

    /// Elvis operator (null-coalescing): `value ?: fallback`
    ///
    /// Compiles to: `value.unwrap_or_else(|| fallback)`
    Elvis {
        value: Box<RirExpression>,
        fallback: Box<RirExpression>,
        result_type: TypeId,
        span: Span,
    },

    /// Safe call operator: `object?.method()`
    ///
    /// Compiles to: `object.and_then(|obj| method(obj))`
    SafeCall {
        object: Box<RirExpression>,
        call: Box<RirExpression>,
        result_type: TypeId,
        span: Span,
    },

    /// Conversion from T to T? (wrapping in Some)
    ///
    /// This is inserted by the lowering pass when a T is used where T? is expected.
    WrapOptional {
        value: Box<RirExpression>,
        result_type: TypeId,
        span: Span,
    },
}

impl RirExpression {
    /// Returns true if this expression is a loop (for/while/loop).
    pub fn is_loop(&self) -> bool {
        matches!(
            self,
            Self::For { .. } | Self::While { .. } | Self::Loop { .. }
        )
    }
}
