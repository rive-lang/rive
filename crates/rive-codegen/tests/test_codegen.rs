//! Tests for code generation.

use rive_codegen::generate;
use rive_lexer::tokenize;
use rive_parser::parse;

#[test]
fn test_generate_simple_function() {
    let source = r#"fun main() {}"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("fn main"));
    assert!(rust_code.contains("()"));
}

#[test]
fn test_generate_function_with_params() {
    let source = r#"fun add(x: Int, y: Int): Int { return x + y }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("fn add"));
    // After formatting, there might be spaces around colons and arrows
    assert!(rust_code.contains("i64"));
    assert!(rust_code.contains("return"));
}

#[test]
fn test_generate_let_statement() {
    let source = r#"fun main() { let x = 42 }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("let x"));
    assert!(rust_code.contains("42"));
}

#[test]
fn test_generate_mutable_variable() {
    let source = r#"fun main() { let mut count = 0 }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("let mut count"));
}

#[test]
fn test_generate_binary_expression() {
    let source = r#"fun main() { let result = 10 + 20 }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("10"));
    assert!(rust_code.contains("20"));
    assert!(rust_code.contains("+"));
}

#[test]
fn test_generate_print_call() {
    let source = r#"fun main() { print("Hello") }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("println!"));
    assert!(rust_code.contains("\"Hello\""));
}

#[test]
fn test_generate_function_call() {
    let source = r#"fun main() { let x = add(1, 2) }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("add"));
    assert!(rust_code.contains("1"));
    assert!(rust_code.contains("2"));
}

#[test]
fn test_generate_array_literal() {
    let source = r#"fun main() { let arr = [1, 2, 3] }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    // Fixed-size arrays for Copy types
    assert!(rust_code.contains("[1, 2, 3]"));
}

#[test]
fn test_generate_comparison() {
    let source = r#"fun main() { let result = 10 < 20 }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("10"));
    assert!(rust_code.contains("20"));
    assert!(rust_code.contains("<"));
}

#[test]
fn test_generate_logical_ops() {
    let source = r#"fun main() { let result = true && false }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("true"));
    assert!(rust_code.contains("false"));
    assert!(rust_code.contains("&&"));
}

#[test]
fn test_generate_unary_ops() {
    let source = r#"fun main() { let x = -42 }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("-"));
    assert!(rust_code.contains("42"));
}

#[test]
fn test_generate_hello_world() {
    let source = r#"
        fun main() {
            print("Hello, Rive!")
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("fn main"));
    assert!(rust_code.contains("println!"));
    assert!(rust_code.contains("\"Hello, Rive!\""));
}

#[test]
fn test_generate_complex_expression() {
    let source = r#"fun main() { let result = (10 + 20) * 3 }"#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let rust_code = generate(&ast).unwrap();

    assert!(rust_code.contains("10"));
    assert!(rust_code.contains("20"));
    assert!(rust_code.contains("3"));
    assert!(rust_code.contains("+"));
    assert!(rust_code.contains("*"));
}
