//! Tests for code generation from RIR.

use rive_codegen::CodeGenerator;
use rive_core::type_system::TypeRegistry;
use rive_ir::{AstLowering, Optimizer};
use rive_lexer::tokenize;
use rive_parser::parse;

fn compile_to_rust(source: &str) -> String {
    // Full pipeline: Source → Tokens → AST → RIR → Optimized RIR → Rust
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();

    // Create type registry and lowering
    let type_registry = TypeRegistry::new();
    let mut lowering = AstLowering::new(type_registry);
    let mut rir_module = lowering.lower_program(&ast).unwrap();

    // Apply optimizations (Optimizer::new() already has default passes)
    let optimizer = Optimizer::new();
    optimizer.optimize(&mut rir_module);

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

    // After constant folding, 10 + 20 becomes 30
    assert!(rust_code.contains("let result"));
    assert!(rust_code.contains("30")); // Constant folded value
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
    let source = r#"fun main() { let x = add(1, 2) }"#;
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

    // After constant folding, 10 < 20 becomes true
    assert!(rust_code.contains("let result"));
    assert!(rust_code.contains("true"));
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
fn test_generate_complex_expression() {
    let source = r#"fun main() { let result = (10 + 20) * 3 print(result) }"#;
    let rust_code = compile_to_rust(source);

    // After constant folding, (10 + 20) * 3 becomes 90
    assert!(rust_code.contains("let result"));
    assert!(rust_code.contains("90"));
}
