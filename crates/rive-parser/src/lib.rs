//! Parser for the Rive language.
//!
//! This crate provides parsing of token streams into Abstract Syntax Trees (ASTs).

pub mod ast;
mod parser;

pub use ast::{
    BinaryOperator, Block, Expression, Function, Item, Parameter, Program, Statement, UnaryOperator,
};
pub use parser::Parser;

use rive_core::Result;
use rive_lexer::Token;

/// Parses a slice of tokens into a Rive program AST.
///
/// # Arguments
/// * `tokens` - The token stream to parse
///
/// # Returns
/// A parsed program AST
///
/// # Errors
/// Returns an error if the token stream contains syntax errors
///
/// # Examples
/// ```
/// use rive_lexer::tokenize;
/// use rive_parser::parse;
///
/// let source = "fun main() { print(\"Hello\") }";
/// let tokens = tokenize(source).unwrap();
/// let ast = parse(&tokens).unwrap();
/// ```
pub fn parse(tokens: &[(Token, rive_core::Span)]) -> Result<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
