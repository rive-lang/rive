//! Null safety rules semantic analysis tests.

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

/// Helper to test null safety scenarios.
fn test_null_safety(source: &str) -> bool {
    compile_and_analyze(source).is_ok()
}

/// Helper to check if source should fail.
fn should_fail(source: &str) -> bool {
    compile_and_analyze(source).is_err()
}

#[test]
fn test_cannot_assign_nullable_to_non_nullable() {
    let source = r#"
        fun main() {
            let x: Int? = 42
            let y: Int = x
        }
    "#;
    assert!(
        should_fail(source),
        "Cannot assign T? to T without unwrapping"
    );
}

#[test]
fn test_can_assign_non_nullable_to_nullable() {
    let source = r#"
        fun main() {
            let x: Int = 42
            let y: Int? = x
        }
    "#;
    assert!(
        test_null_safety(source),
        "Can assign T to T? (implicit conversion)"
    );
}

#[test]
fn test_cannot_pass_nullable_to_non_nullable_param() {
    let source = r#"
        fun process(x: Int) {
            return
        }
        
        fun main() {
            let value: Int? = 42
            process(value)
        }
    "#;
    assert!(
        should_fail(source),
        "Cannot pass T? to function expecting T"
    );
}

#[test]
fn test_can_pass_non_nullable_to_nullable_param() {
    let source = r#"
        fun process(x: Int?) {
            return
        }
        
        fun main() {
            let value: Int = 42
            process(value)
        }
    "#;
    assert!(
        test_null_safety(source),
        "Can pass T to function expecting T?"
    );
}

#[test]
fn test_cannot_return_nullable_as_non_nullable() {
    let source = r#"
        fun get_value(): Int {
            let x: Int? = 42
            return x
        }
        
        fun main() {
        }
    "#;
    assert!(
        should_fail(source),
        "Cannot return T? from function expecting T"
    );
}

#[test]
fn test_can_return_non_nullable_as_nullable() {
    let source = r#"
        fun get_value(): Int? {
            let x: Int = 42
            return x
        }
        
        fun main() {
        }
    "#;
    assert!(
        test_null_safety(source),
        "Can return T from function expecting T?"
    );
}

#[test]
fn test_null_can_assign_to_nullable() {
    let source = r#"
        fun main() {
            let x: Int? = null
            let y: Text? = null
        }
    "#;
    assert!(test_null_safety(source), "null can be assigned to any T?");
}

#[test]
fn test_null_cannot_assign_to_non_nullable() {
    let source = r#"
        fun main() {
            let x: Int = null
        }
    "#;
    assert!(
        should_fail(source),
        "null cannot be assigned to T (non-nullable)"
    );
}
