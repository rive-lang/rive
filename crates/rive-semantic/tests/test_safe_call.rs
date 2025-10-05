//! Safe call operator (?.) semantic analysis tests.

mod common;
use common::{compile, should_fail};
use rive_semantic::analyze_with_registry;

/// Helper to test safe call scenarios.
fn test_safe_call(source: &str) -> bool {
    compile(source)
        .and_then(|(ast, type_registry)| analyze_with_registry(&ast, type_registry))
        .is_ok()
}

#[test]
fn test_safe_call_with_function() {
    let source = r#"
        fun get_value(): Int {
            return 42
        }
        
        fun main() {
            let x: Int? = 10
            let y: Int? = x?.get_value()
        }
    "#;
    assert!(
        test_safe_call(source),
        "Safe call should work with nullable object and function call"
    );
}

#[test]
fn test_safe_call_with_non_nullable() {
    let source = r#"
        fun get_value(): Int {
            return 42
        }
        
        fun main() {
            let x: Int = 10
            let y: Int? = x?.get_value()
        }
    "#;
    assert!(
        test_safe_call(source),
        "Safe call should work even with non-nullable object"
    );
}

#[test]
fn test_safe_call_chained() {
    let source = r#"
        fun get_value(): Int? {
            return 42
        }
        
        fun get_another(): Int {
            return 10
        }
        
        fun main() {
            let x: Int? = 10
            let y: Int? = x?.get_value()?.get_another()
        }
    "#;
    assert!(test_safe_call(source), "Chained safe calls should work");
}

#[test]
fn test_safe_call_result_always_nullable() {
    let source = r#"
        fun get_value(): Int {
            return 42
        }
        
        fun main() {
            let x: Int? = 10
            let y: Int? = x?.get_value()
            let z: Int = x?.get_value()
        }
    "#;
    // This should fail because safe call always returns nullable
    assert!(
        should_fail(source),
        "Safe call result must be nullable, cannot assign to non-nullable"
    );
}

