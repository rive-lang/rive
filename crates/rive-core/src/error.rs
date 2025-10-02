//! Error types and result aliases for the Rive compiler.

use crate::Span;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Main error type for the Rive compiler.
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Lexer error: {0}")]
    Lexer(String),

    #[error("Parser error: {0}")]
    #[diagnostic(code(rive::parser))]
    Parser(String, #[label("here")] Span),

    #[error("Semantic error: {0}")]
    Semantic(String),

    #[error("{0}")]
    #[diagnostic(code(rive::semantic))]
    SemanticWithSpan(String, #[label("here")] Span),

    #[error("Code generation error: {0}")]
    Codegen(String),
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::from(span.start.offset..span.end.offset)
    }
}

/// Result type alias using the Rive Error type.
pub type Result<T> = std::result::Result<T, Error>;
