//! Tests for code generation from RIR.

use rive_codegen::CodeGenerator;
use rive_ir::AstLowering;
use rive_lexer::tokenize;
use rive_parser::parse;

fn compile_to_rust(source: &str) -> String {
    // Full pipeline: Source → Tokens → AST → RIR → Optimized RIR → Rust
    let tokens = tokenize(source).unwrap();
    let (ast, type_registry) = parse(&tokens).unwrap();

    // Use the type registry from parser
    let mut lowering = AstLowering::new(type_registry);
    let rir_module = lowering.lower_program(&ast).unwrap();

    // Generate Rust code
    let mut codegen = CodeGenerator::new();
    codegen.generate(&rir_module).unwrap()
}

#[test]
fn test_generate_simple_function() {
    let source = r#"fun main() {}"#;
    let rust_code = compile_to_rust(source);

    assert!(rust_code.contains("fn main"));
    assert!(rust_code.contains("()"));
}

#[test]
fn test_generate_function_with_params() {
    let source = r#"fun add(x: Int, y: Int): Int { return x + y }"#;
    let rust_code = compile_to_rust(source);

    assert!(rust_code.contains("fn add"));
    assert!(rust_code.contains("i64"));
    assert!(rust_code.contains("return"));
}

#[test]
fn test_generate_let_statement() {
    let source = r#"fun main() { let x = 42 print(x) }"#;
    let rust_code = compile_to_rust(source);

    assert!(rust_code.contains("let x"));
    assert!(rust_code.contains("42"));
}

#[test]
fn test_generate_mutable_variable() {
    let source = r#"fun main() { let mut count = 0 print(count) }"#;
    let rust_code = compile_to_rust(source);

    assert!(rust_code.contains("let mut count"));
}

#[test]
fn test_generate_binary_expression() {
    let source = r#"fun main() { let result = 10 + 20 print(result) }"#;
    let rust_code = compile_to_rust(source);

    // Without optimizer, 10 + 20 remains as 10 + 20
    assert!(rust_code.contains("let result"));
    assert!(rust_code.contains("10 + 20")); // Original expression
}

#[test]
fn test_generate_print_call() {
    let source = r#"fun main() { print("Hello") }"#;
    let rust_code = compile_to_rust(source);

    assert!(rust_code.contains("println!"));
    assert!(rust_code.contains("\"Hello\""));
}

#[test]
fn test_generate_function_call() {
    let source = r#"
        fun add(x: Int, y: Int): Int { return x + y }
        fun main() { let x = add(1, 2) }
    "#;
    let rust_code = compile_to_rust(source);

    assert!(rust_code.contains("add"));
    assert!(rust_code.contains("1"));
    assert!(rust_code.contains("2"));
}

#[test]
fn test_generate_array_literal() {
    let source = r#"fun main() { let arr = [1, 2, 3] print(arr) }"#;
    let rust_code = compile_to_rust(source);

    // Fixed-size arrays for Copy types
    assert!(rust_code.contains("[1, 2, 3]"));
}

#[test]
fn test_generate_comparison() {
    let source = r#"fun main() { let result = 10 < 20 print(result) }"#;
    let rust_code = compile_to_rust(source);

    // Without optimizer, 10 < 20 remains as 10 < 20
    assert!(rust_code.contains("let result"));
    assert!(rust_code.contains("10 < 20")); // Original expression
}

#[test]
fn test_generate_logical_ops() {
    let source = r#"fun main() { let result = true && false print(result) }"#;
    let rust_code = compile_to_rust(source);

    // After constant folding, true && false becomes false
    assert!(rust_code.contains("let result"));
    assert!(rust_code.contains("false"));
}

#[test]
fn test_generate_unary_ops() {
    let source = r#"fun main() { let x = -42 print(x) }"#;
    let rust_code = compile_to_rust(source);

    // After constant folding, this stays as -42
    assert!(rust_code.contains("let x"));
    assert!(rust_code.contains("-42"));
}

#[test]
fn test_generate_hello_world() {
    let source = r#"
        fun main() {
            print("Hello, Rive!")
        }
    "#;
    let rust_code = compile_to_rust(source);

    assert!(rust_code.contains("fn main"));
    assert!(rust_code.contains("println!"));
    assert!(rust_code.contains("\"Hello, Rive!\""));
}

#[test]
fn test_inline_optimization() {
    let source = r#"
        fun main() {
            let result = add(3, 4)
            print("Result:", result)
        }
        
        fun add(x: Int, y: Int): Int {
            return x + y
        }
        
        fun complex_function(x: Int, y: Int): Int {
            let step1 = x + y
            let step2 = x - y
            let step3 = step1 * step2
            let step4 = step3 / 2
            let step5 = step4 + 1
            let step6 = step5 * 2
            return step6
        }
    "#;

    let rust_code = compile_to_rust(source);

    // Simple function should be inlined
    assert!(rust_code.contains("#[inline]"));
    assert!(rust_code.contains("fn add("));

    // Complex function should not be inlined
    assert!(rust_code.contains("fn complex_function("));
    assert!(!rust_code.contains("#[inline]\nfn complex_function("));
}

// ==================== Null Safety Tests ====================

#[test]
fn test_generate_null_literal() {
    let source = r#"fun main() { let x: Int? = null }"#;
    let rust_code = compile_to_rust(source);
    assert!(rust_code.contains("None"), "null should compile to None");
}

#[test]
fn test_generate_elvis_operator() {
    let source = r#"
        fun main() {
            let x: Int? = null
            let y: Int = x ?: 42
        }
    "#;
    let rust_code = compile_to_rust(source);
    assert!(
        rust_code.contains("unwrap_or") || rust_code.contains("unwrap_or_else"),
        "Elvis operator should compile to unwrap_or or unwrap_or_else"
    );
}

#[test]
fn test_generate_safe_call_operator() {
    let source = r#"
        fun get_value(): Int? {
            return null
        }
        fun main() {
            let x: Int? = get_value()?.to_string()
        }
    "#;
    let rust_code = compile_to_rust(source);
    // Safe call should use Option's map/and_then methods
    assert!(
        rust_code.contains("and_then") || rust_code.contains("map"),
        "Safe call should compile to and_then or map"
    );
}

#[test]
fn test_generate_nullable_type_conversion() {
    let source = r#"
        fun main() {
            let x: Int = 42
            let y: Int? = x
        }
    "#;
    let rust_code = compile_to_rust(source);
    assert!(rust_code.contains("Some"), "T -> T? should wrap in Some");
}
