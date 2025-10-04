//! Tests for the Rive parser.

use rive_lexer::tokenize;
use rive_parser::{BinaryOperator, Expression, Item, Statement, parse};

#[test]
fn test_parse_simple_function() {
    let source = r#"fun main() {}"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    assert_eq!(program.items.len(), 1);

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.name, "main");
    assert_eq!(func.params.len(), 0);
    assert_eq!(func.body.statements.len(), 0);
}

#[test]
fn test_parse_function_with_parameters() {
    let source = r#"fun add(x: Int, y: Int): Int { return x + y }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    assert_eq!(program.items.len(), 1);

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.params[0].name, "x");
    assert_eq!(func.params[1].name, "y");
}

#[test]
fn test_parse_let_statement() {
    let source = r#"fun main() { let x = 42 }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.body.statements.len(), 1);

    if let Statement::Let { name, mutable, .. } = &func.body.statements[0] {
        assert_eq!(name, "x");
        assert!(!mutable);
    } else {
        panic!("Expected let statement");
    }
}

#[test]
fn test_parse_mutable_variable() {
    let source = r#"fun main() { let mut count = 0 }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    if let Statement::Let { name, mutable, .. } = &func.body.statements[0] {
        assert_eq!(name, "count");
        assert!(mutable);
    } else {
        panic!("Expected let statement");
    }
}

#[test]
fn test_parse_binary_expression() {
    let source = r#"fun main() { let result = 10 + 20 }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    if let Statement::Let { initializer, .. } = &func.body.statements[0] {
        if let Expression::Binary { operator, .. } = initializer {
            assert_eq!(*operator, BinaryOperator::Add);
        } else {
            panic!("Expected binary expression");
        }
    }
}

#[test]
fn test_parse_function_call() {
    let source = r#"fun main() { print("Hello") }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    if let Statement::Expression { expression, .. } = &func.body.statements[0] {
        if let Expression::Call {
            callee, arguments, ..
        } = expression
        {
            assert_eq!(callee, "print");
            assert_eq!(arguments.len(), 1);
        } else {
            panic!("Expected function call");
        }
    }
}

#[test]
fn test_parse_return_statement() {
    let source = r#"fun test(): Int { return 42 }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    if let Statement::Return { value, .. } = &func.body.statements[0] {
        assert!(value.is_some());
        if let Some(Expression::Integer { value, .. }) = value {
            assert_eq!(*value, 42);
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_parse_array_literal() {
    let source = r#"fun main() { let arr = [1, 2, 3] }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    if let Statement::Let { initializer, .. } = &func.body.statements[0] {
        if let Expression::Array { elements, .. } = initializer {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected array expression");
        }
    }
}

#[test]
fn test_parse_comparison_operators() {
    let source = r#"fun main() { 
        let a = 10 < 20
        let b = 10 <= 20
        let c = 10 > 5
        let d = 10 >= 5
        let e = 10 == 10
        let f = 10 != 5
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.body.statements.len(), 6);
}

#[test]
fn test_parse_logical_operators() {
    let source = r#"fun main() { 
        let result = true && false || true
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.body.statements.len(), 1);
}

#[test]
fn test_parse_unary_operators() {
    let source = r#"fun main() { 
        let a = -42
        let b = !true
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.body.statements.len(), 2);
}

#[test]
fn test_parse_complex_expression() {
    let source = r#"fun main() { 
        let result = (10 + 20) * 3 - 5 / 2
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.body.statements.len(), 1);
}

#[test]
fn test_parse_hello_world() {
    let source = r#"
        fun main() {
            print("Hello, Rive!")
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    assert_eq!(program.items.len(), 1);

    let Item::Function(func) = &program.items[0];
    assert_eq!(func.name, "main");
    assert_eq!(func.body.statements.len(), 1);
}
