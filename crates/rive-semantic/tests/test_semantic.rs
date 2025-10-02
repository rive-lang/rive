//! Integration tests for semantic analysis.

use rive_lexer::tokenize;
use rive_parser::parse;
use rive_semantic::analyze;

#[test]
fn test_simple_variable_declaration() {
    let source = r#"
        fun main() {
            let x: Int = 42
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_type_inference() {
    let source = r#"
        fun main() {
            let x = 42
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_type_mismatch() {
    let source = r#"
        fun main() {
            let x: Int = "hello"
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_err());
}

#[test]
fn test_undefined_variable() {
    let source = r#"
        fun main() {
            let x = y
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_err());
}

#[test]
fn test_function_call() {
    let source = r#"
        fun add(x: Int, y: Int): Int {
            return x + y
        }
        
        fun main() {
            let result = add(1, 2)
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_function_wrong_argument_count() {
    let source = r#"
        fun add(x: Int, y: Int): Int {
            return x + y
        }
        
        fun main() {
            let result = add(1)
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_err());
}

#[test]
fn test_function_wrong_argument_type() {
    let source = r#"
        fun add(x: Int, y: Int): Int {
            return x + y
        }
        
        fun main() {
            let result = add(1, "hello")
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_err());
}

#[test]
fn test_return_type_mismatch() {
    let source = r#"
        fun add(x: Int, y: Int): Int {
            return "hello"
        }
        
        fun main() {
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_err());
}

#[test]
fn test_print_function() {
    let source = r#"
        fun main() {
            print(42)
            print("hello")
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_array_literal() {
    let source = r#"
        fun main() {
            let arr = [1, 2, 3]
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_array_type_mismatch() {
    let source = r#"
        fun main() {
            let arr = [1, "hello", 3]
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_err());
}

#[test]
fn test_binary_operations() {
    let source = r#"
        fun main() {
            let x = 1 + 2
            let y = 3 - 4
            let z = 5 * 6
            let w = 7 / 8
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_comparison_operations() {
    let source = r#"
        fun main() {
            let a = 1 < 2
            let b = 3 > 4
            let c = 5 == 6
            let d = 7 != 8
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_no_main_function() {
    let source = r#"
        fun add(x: Int, y: Int): Int {
            return x + y
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let ast = parse(&tokens).unwrap();
    let result = analyze(&ast);
    assert!(result.is_err());
}
