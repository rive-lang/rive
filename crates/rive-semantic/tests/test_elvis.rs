//! Elvis operator (?:) semantic analysis tests.

mod common;
use common::{compile, should_fail};
use rive_semantic::analyze_with_registry;

/// Helper to test Elvis operator scenarios.
fn test_elvis(source: &str) -> bool {
    compile(source)
        .and_then(|(ast, type_registry)| analyze_with_registry(&ast, type_registry))
        .is_ok()
}

#[test]
fn test_elvis_nullable_with_non_nullable_fallback() {
    let source = r#"
        fun main() {
            let x: Int? = null
            let y: Int = x ?: 42
        }
    "#;
    assert!(
        test_elvis(source),
        "Elvis operator should work with nullable value and non-nullable fallback"
    );
}

#[test]
fn test_elvis_nullable_with_nullable_fallback() {
    let source = r#"
        fun main() {
            let x: Int? = null
            let y: Int? = 10
            let z: Int? = x ?: y
        }
    "#;
    assert!(
        test_elvis(source),
        "Elvis operator should work with two nullable types"
    );
}

#[test]
fn test_elvis_null_literal_with_fallback() {
    let source = r#"
        fun main() {
            let x: Int = null ?: 42
        }
    "#;
    assert!(
        test_elvis(source),
        "Elvis operator should work with null literal"
    );
}

#[test]
fn test_elvis_chained() {
    let source = r#"
        fun main() {
            let x: Int? = null
            let y: Int? = null
            let z: Int = x ?: y ?: 42
        }
    "#;
    assert!(test_elvis(source), "Chained Elvis operators should work");
}

#[test]
fn test_elvis_non_nullable_value() {
    let source = r#"
        fun main() {
            let x: Int = 10
            let y: Int = x ?: 42
        }
    "#;
    // This should be OK (though could warn in the future that Elvis is redundant)
    assert!(
        test_elvis(source),
        "Elvis operator should work even with non-nullable (though redundant)"
    );
}

#[test]
fn test_elvis_type_mismatch() {
    let source = r#"
        fun main() {
            let x: Int? = null
            let y: Int = x ?: "hello"
        }
    "#;
    assert!(
        should_fail(source),
        "Elvis operator should reject incompatible types"
    );
}

