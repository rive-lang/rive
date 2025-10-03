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
use rive_core::type_system::TypeRegistry;
use rive_lexer::Token;

/// Parses a slice of tokens into a Rive program AST and returns the type registry.
///
/// # Arguments
/// * `tokens` - The token stream to parse
///
/// # Returns
/// A tuple of (parsed program AST, type registry)
///
/// # Errors
/// Returns an error if the token stream contains syntax errors
///
/// # Examples
/// ```
/// use rive_lexer::tokenize;
/// use rive_parser::parse_with_types;
///
/// let source = "fun main() { print(\"Hello\") }";
/// let tokens = tokenize(source).unwrap();
/// let (ast, type_registry) = parse_with_types(&tokens).unwrap();
/// ```
pub fn parse_with_types(tokens: &[(Token, rive_core::Span)]) -> Result<(Program, TypeRegistry)> {
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    let type_registry = parser.into_type_registry();
    Ok((program, type_registry))
}

/// Parses a slice of tokens into a Rive program AST.
///
/// This is a convenience function that discards the type registry.
/// Use `parse_with_types` if you need access to the type registry.
///
/// # Arguments
/// * `tokens` - The token stream to parse
///
/// # Returns
/// A parsed program AST
///
/// # Errors
/// Returns an error if the token stream contains syntax errors
pub fn parse(tokens: &[(Token, rive_core::Span)]) -> Result<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
