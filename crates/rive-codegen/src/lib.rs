//! Code generation for the Rive language.
//!
//! This crate generates Rust code from Rive ASTs.

mod codegen;

pub use codegen::CodeGenerator;

use rive_core::Result;
use rive_parser::Program;

/// Generates Rust code from a Rive program AST.
///
/// # Arguments
/// * `program` - The parsed Rive program
///
/// # Returns
/// Generated Rust source code as a string
///
/// # Errors
/// Returns an error if code generation fails
///
/// # Examples
/// ```
/// use rive_lexer::tokenize;
/// use rive_parser::parse;
/// use rive_codegen::generate;
///
/// let source = "fun main() { print(\"Hello\") }";
/// let tokens = tokenize(source).unwrap();
/// let ast = parse(&tokens).unwrap();
/// let rust_code = generate(&ast).unwrap();
/// ```
pub fn generate(program: &Program) -> Result<String> {
    let mut generator = CodeGenerator::new();
    generator.generate(program)
}
