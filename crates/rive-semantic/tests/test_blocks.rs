//! Block expression semantic analysis tests.

use rive_core::Result;
use rive_lexer::tokenize;
use rive_parser::parse;
use rive_semantic::analyze_with_registry;

/// Helper to compile and analyze Rive source code.
fn compile_and_analyze(source: &str) -> Result<()> {
    let tokens = tokenize(source)?;
    let (ast, type_registry) = parse(&tokens)?;
    analyze_with_registry(&ast, type_registry)?;
    Ok(())
}

/// Helper to test block expression scenarios.
fn test_blocks(source: &str) -> bool {
    compile_and_analyze(source).is_ok()
}

/// Helper to check if source should fail.
fn should_fail(source: &str) -> bool {
    compile_and_analyze(source).is_err()
}

#[test]
fn test_block_expression_with_final_expr() {
    let source = r#"
        fun main() {
            let x: Int = {
                let a: Int = 10
                let b: Int = 20
                a + b
            }
        }
    "#;
    assert!(
        test_blocks(source),
        "Block expression should return the type of its final expression"
    );
}

#[test]
fn test_block_expression_without_final_expr() {
    let source = r#"
        fun process(): Int {
            let x = {
                let a: Int = 10
                return a
            }
            return 0
        }
        
        fun main() {
        }
    "#;
    assert!(
        test_blocks(source),
        "Block expression with return statement should be valid"
    );
}

#[test]
fn test_block_expression_type_mismatch() {
    let source = r#"
        fun main() {
            let x: Int = {
                let a: Text = "hello"
                a
            }
        }
    "#;
    assert!(
        should_fail(source),
        "Block expression type must match variable type"
    );
}

#[test]
fn test_nested_block_expressions() {
    let source = r#"
        fun main() {
            let x: Int = {
                let a: Int = {
                    10 + 20
                }
                a * 2
            }
        }
    "#;
    assert!(test_blocks(source), "Nested block expressions should work");
}

#[test]
fn test_block_expression_with_elvis() {
    let source = r#"
        fun main() {
            let x: Int = {
                let a: Int? = null
                a ?: 42
            }
        }
    "#;
    assert!(
        test_blocks(source),
        "Block expression with Elvis operator should work"
    );
}

