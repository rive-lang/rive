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
use rive_core::type_system::TypeRegistry;
use rive_parser::ast::Program;

/// Performs semantic analysis on a Rive program.
///
/// # Arguments
/// * `program` - The parsed AST to analyze
/// * `type_registry` - The type registry from the parser (optional)
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
/// use rive_semantic::analyze_with_registry;
/// use rive_parser::parse;
/// use rive_lexer::tokenize;
///
/// let source = "fun main() { let x: Int = 42 }";
/// let tokens = tokenize(source).unwrap();
/// let (ast, type_registry) = parse(&tokens).unwrap();
/// let result = analyze_with_registry(&ast, type_registry);
/// assert!(result.is_ok());
/// ```
pub fn analyze_with_registry(
    program: &Program,
    type_registry: TypeRegistry,
) -> Result<TypeRegistry> {
    let symbols = SymbolTable::with_registry(type_registry);
    let mut checker = TypeChecker::with_symbols(symbols);
    checker.check_program(program)?;
    // Extract and return the type registry
    Ok(checker.into_type_registry())
}

/// Performs semantic analysis on a Rive program (for backward compatibility).
///
/// **Note**: This creates a new TypeRegistry and may not work correctly with
/// programs that have type annotations. Use `analyze_with_registry` instead.
///
/// # Deprecated
/// Use `analyze_with_registry` instead to pass the parser's type registry.
pub fn analyze(program: &Program) -> Result<()> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}
