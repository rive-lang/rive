//! Common test utilities for semantic analysis tests.

use rive_core::{type_system::TypeRegistry, Result};
use rive_lexer::tokenize;
use rive_parser::{ast::Program, parse};
use rive_semantic::analyze_with_registry;

/// Helper function to compile and analyze Rive source code.
///
/// # Returns
/// - `Ok(())` if compilation and analysis succeed
/// - `Err(rive_core::Error)` if any stage fails
pub fn compile_and_analyze(source: &str) -> Result<()> {
    let tokens = tokenize(source)?;
    let (ast, type_registry) = parse(&tokens)?;
    analyze_with_registry(&ast, type_registry)?;
    Ok(())
}

/// Helper function to compile source and return the AST and type registry.
///
/// # Returns
/// - `Ok((Program, TypeRegistry))` if compilation succeeds
/// - `Err(rive_core::Error)` if any stage fails
pub fn compile(source: &str) -> Result<(Program, TypeRegistry)> {
    let tokens = tokenize(source)?;
    let (ast, type_registry) = parse(&tokens)?;
    Ok((ast, type_registry))
}

/// Helper function to check if source code fails semantic analysis.
pub fn should_fail(source: &str) -> bool {
    compile_and_analyze(source).is_err()
}

/// Helper function to check if source code passes semantic analysis.
pub fn should_pass(source: &str) -> bool {
    compile_and_analyze(source).is_ok()
}

