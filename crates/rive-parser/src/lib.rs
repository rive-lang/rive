//! Parser for the Rive language.
//!
//! This crate provides parsing of token streams into Abstract Syntax Trees (ASTs).

pub mod ast;
pub mod control_flow;

mod parsing;

pub use ast::{
    BinaryOperator, Block, Expression, Function, Item, Parameter, Program, Statement, UnaryOperator,
};
pub use control_flow::{
    Break, Continue, ElseIf, For, If, Loop, Match, MatchArm, Pattern, Range, While,
};
pub use parsing::Parser;

use rive_core::type_system::TypeRegistry;
use rive_core::{Result, Span};
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
/// use rive_parser::parse;
/// use rive_lexer::tokenize;
///
/// let source = "fun main() { print(\"Hello\") }";
/// let tokens = tokenize(source).unwrap();
/// let (program, type_registry) = parse(&tokens).unwrap();
/// ```
pub fn parse(tokens: &[(Token, Span)]) -> Result<(Program, TypeRegistry)> {
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    let type_registry = parser.into_type_registry();
    Ok((program, type_registry))
}
