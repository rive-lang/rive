//! Semantic analysis for Rive programs.
//!
//! This crate provides type checking, symbol resolution, and semantic validation
//! for Rive source code. It operates on the AST produced by the parser and
//! ensures type safety and proper variable usage.

mod checker;
mod symbol_table;

pub use checker::TypeChecker;
pub use symbol_table::{Symbol, SymbolTable};

use rive_core::Result;
use rive_parser::ast::Program;

/// Performs semantic analysis on a Rive program.
///
/// # Arguments
/// * `program` - The parsed AST to analyze
///
/// # Returns
/// * `Result<()>` - Ok if analysis succeeds, Err with semantic errors otherwise
///
/// # Errors
/// Returns semantic errors for:
/// - Type mismatches
/// - Undefined variables or functions
/// - Mutability violations
/// - Invalid operations
///
/// # Examples
/// ```
/// use rive_semantic::analyze;
/// use rive_parser::parse;
/// use rive_lexer::tokenize;
///
/// let source = "fun main() { let x: Int = 42 }";
/// let tokens = tokenize(source).unwrap();
/// let (ast, _type_registry) = parse(&tokens).unwrap();
/// let result = analyze(&ast);
/// assert!(result.is_ok());
/// ```
pub fn analyze(program: &Program) -> Result<()> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}
