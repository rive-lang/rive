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
    TypeDecl(TypeDecl),
    EnumDecl(EnumDecl),
    InterfaceDecl(InterfaceDecl),
    ImplBlock(ImplBlock),
}

/// Function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: TypeId,
    pub body: FunctionBody,
    pub span: Span,
}

/// Function body: either a block or a single expression.
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    /// Block body: `{ statements... }`
    Block(Block),
    /// Expression body: `= expr`
    Expression(Expression),
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: TypeId,
    pub span: Span,
}

/// Argument in a function/method/constructor call
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// Positional argument
    Positional(Expression),
    /// Named argument: `name = value`
    Named {
        name: String,
        value: Expression,
        span: Span,
    },
}

/// Part of a string interpolation
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    /// Literal string part
    Literal(String),
    /// Expression to be interpolated: $var or ${expr}
    Interpolation(Box<Expression>),
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

    /// Constant declaration: `const name[?] [: type[?]] = expr`
    Const {
        name: String,
        var_type: Option<TypeId>,
        /// Whether to infer as nullable when no explicit type
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

    /// String with interpolation: "Hello $name, you have ${count} messages"
    /// Parts alternate between string literals and expressions
    StringInterpolation { parts: Vec<StringPart>, span: Span },

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
        arguments: Vec<Argument>,
        span: Span,
    },

    /// Constructor call: `TypeName(args...)`
    ConstructorCall {
        type_name: String,
        arguments: Vec<Argument>,
        span: Span,
    },

    /// Enum variant construction: `EnumName.Variant` or `EnumName.Variant(args...)`
    EnumVariant {
        enum_name: String,
        variant_name: String,
        arguments: Vec<Argument>,
        span: Span,
    },

    /// Array literal: `[expr, ...]`
    Array {
        elements: Vec<Expression>,
        span: Span,
    },

    /// Tuple literal: `(a, b, c)` or `(a,)` for single element
    Tuple {
        elements: Vec<Expression>,
        span: Span,
    },

    /// List constructor: `List(1, 2, 3)`
    List {
        elements: Vec<Expression>,
        span: Span,
    },

    /// Dictionary literal: `{"key": value, ...}`
    Dict {
        entries: Vec<(String, Expression)>,
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

    /// Method call: `object.method(args...)`
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Argument>,
        span: Span,
    },

    /// Field access: `object.field` (for tuple indexing like `t.0`)
    FieldAccess {
        object: Box<Expression>,
        field: String,
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
            Self::StringInterpolation { span, .. } => *span,
            Self::Boolean { span, .. } => *span,
            Self::Null { span } => *span,
            Self::Variable { span, .. } => *span,
            Self::Binary { span, .. } => *span,
            Self::Unary { span, .. } => *span,
            Self::Call { span, .. } => *span,
            Self::Array { span, .. } => *span,
            Self::Tuple { span, .. } => *span,
            Self::List { span, .. } => *span,
            Self::Dict { span, .. } => *span,
            Self::If(expr) => expr.span,
            Self::While(expr) => expr.span,
            Self::For(expr) => expr.span,
            Self::Loop(expr) => expr.span,
            Self::Match(expr) => expr.span,
            Self::Range(expr) => expr.span,
            Self::Block(block) => block.span,
            Self::Elvis { span, .. } => *span,
            Self::SafeCall { span, .. } => *span,
            Self::MethodCall { span, .. } => *span,
            Self::FieldAccess { span, .. } => *span,
            Self::ConstructorCall { span, .. } => *span,
            Self::EnumVariant { span, .. } => *span,
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

/// Type declaration: `type Name(ctor_params) { fields, methods, impls }`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: String,
    /// Constructor parameters that become fields
    pub ctor_params: Vec<Field>,
    /// Additional fields defined in the body
    pub fields: Vec<Field>,
    /// Methods defined in the type body
    pub methods: Vec<MethodDecl>,
    /// Inline interface implementations
    pub inline_impls: Vec<InlineImpl>,
    pub span: Span,
}

/// Field definition in a type
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: TypeId,
    pub mutable: bool,
    pub span: Span,
}

/// Enum declaration: `enum Name { Variant1, Variant2(field: Type), ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    /// Enum variants
    pub variants: Vec<EnumVariantDecl>,
    /// Methods defined in the enum body
    pub methods: Vec<MethodDecl>,
    /// Inline interface implementations
    pub inline_impls: Vec<InlineImpl>,
    pub span: Span,
}

/// Enum variant declaration
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantDecl {
    pub name: String,
    /// Optional named fields for this variant
    pub fields: Option<Vec<Field>>,
    pub span: Span,
}

/// Method declaration (instance or static)
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub is_static: bool,
    pub params: Vec<Parameter>,
    pub return_type: TypeId,
    pub body: FunctionBody,
    pub span: Span,
}

/// Interface declaration: `interface Name { method_signatures }`
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    pub name: String,
    pub methods: Vec<MethodSig>,
    pub span: Span,
}

/// Method signature (no body)
#[derive(Debug, Clone, PartialEq)]
pub struct MethodSig {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: TypeId,
    pub span: Span,
}

/// Implementation block: `impl [Interface for] Type { methods }`
#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    /// Type being implemented for
    pub target_type: String,
    /// Optional interface being implemented
    pub interface: Option<String>,
    /// Methods in this impl block
    pub methods: Vec<MethodDecl>,
    pub span: Span,
}

/// Inline implementation within type declaration
#[derive(Debug, Clone, PartialEq)]
pub struct InlineImpl {
    pub interface: String,
    pub methods: Vec<MethodDecl>,
    pub span: Span,
}
