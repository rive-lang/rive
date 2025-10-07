//! Tests for the Rive parser.

use rive_lexer::tokenize;
use rive_parser::ast::FunctionBody;
use rive_parser::{BinaryOperator, Expression, Item, Statement, parse};

/// Helper function to get statements from a function body
fn get_statements(body: &FunctionBody) -> &[Statement] {
    match body {
        FunctionBody::Block(block) => &block.statements,
        FunctionBody::Expression(_) => &[], // Expression bodies have no statements
    }
}

#[test]
fn test_parse_simple_function() {
    let source = r#"fun main() {}"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    assert_eq!(program.items.len(), 1);

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(func.name, "main");
    assert_eq!(func.params.len(), 0);
    assert_eq!(get_statements(&func.body).len(), 0);
}

#[test]
fn test_parse_function_with_parameters() {
    let source = r#"fun add(x: Int, y: Int): Int { return x + y }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    assert_eq!(program.items.len(), 1);

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 1);

    if let Statement::Let { name, mutable, .. } = &get_statements(&func.body)[0] {
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    if let Statement::Let { name, mutable, .. } = &get_statements(&func.body)[0] {
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    if let Statement::Let { initializer, .. } = &get_statements(&func.body)[0] {
        if let Expression::Binary { operator, .. } = initializer {
            assert_eq!(operator, &BinaryOperator::Add);
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    if let Statement::Expression { expression, .. } = &get_statements(&func.body)[0] {
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    if let Statement::Return { value, .. } = &get_statements(&func.body)[0] {
        assert!(value.is_some());
        if let Some(Expression::Integer { value, .. }) = value {
            assert_eq!(value, &42);
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    if let Statement::Let { initializer, .. } = &get_statements(&func.body)[0] {
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 6);
}

#[test]
fn test_parse_logical_operators() {
    let source = r#"fun main() { 
        let result = true && false || true
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 1);
}

#[test]
fn test_parse_unary_operators() {
    let source = r#"fun main() { 
        let a = -42
        let b = !true
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 2);
}

#[test]
fn test_parse_complex_expression() {
    let source = r#"fun main() { 
        let result = (10 + 20) * 3 - 5 / 2
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 1);
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

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(func.name, "main");
    assert_eq!(get_statements(&func.body).len(), 1);
}

#[test]
fn test_parse_nullable_type() {
    let source = r#"fun test(x: Int?): Text? { return null }"#;
    let tokens = tokenize(source).unwrap();
    let (program, type_registry) = parse(&tokens).unwrap();

    assert_eq!(program.items.len(), 1);

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(func.name, "test");
    assert_eq!(func.params.len(), 1);

    // Parameter should have nullable type
    let param_type = func.params[0].param_type;
    let param_meta = type_registry.get(param_type).unwrap();
    assert!(matches!(
        param_meta.kind,
        rive_core::type_system::TypeKind::Optional { .. }
    ));

    // Return type should be nullable
    let return_type = func.return_type;
    let return_meta = type_registry.get(return_type).unwrap();
    assert!(matches!(
        return_meta.kind,
        rive_core::type_system::TypeKind::Optional { .. }
    ));
}

#[test]
fn test_parse_multiple_nullable_types() {
    let source = r#"
        fun test(a: Int?, b: Float?, c: Text?, d: Bool?) {
            let x: Int? = null
            let y: Text? = null
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let (program, type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(func.params.len(), 4);

    // All parameters should be nullable
    for param in &func.params {
        let param_meta = type_registry.get(param.param_type).unwrap();
        assert!(matches!(
            param_meta.kind,
            rive_core::type_system::TypeKind::Optional { .. }
        ));
    }

    // Variables should be nullable
    assert_eq!(get_statements(&func.body).len(), 2);
}

#[test]
fn test_parse_array_of_nullable_types() {
    let source = r#"fun test(arr: [Int?; 5]) {}"#;
    let tokens = tokenize(source).unwrap();
    let (program, type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    let param_type = func.params[0].param_type;
    let param_meta = type_registry.get(param_type).unwrap();

    // Should be an array type
    if let rive_core::type_system::TypeKind::Array { element, size } = param_meta.kind {
        assert_eq!(size, 5);

        // Element should be nullable Int
        let element_meta = type_registry.get(element).unwrap();
        assert!(matches!(
            element_meta.kind,
            rive_core::type_system::TypeKind::Optional { .. }
        ));
    } else {
        panic!("Expected array type");
    }
}

#[test]
fn test_reject_optional_syntax() {
    // Old Optional<T> syntax should now fail
    let source = r#"fun test(x: Optional<Int>) {}"#;
    let tokens = tokenize(source).unwrap();
    let result = parse(&tokens);

    // Should fail because "Optional" is unknown type
    assert!(result.is_err());
}

#[test]
fn test_parse_elvis_operator() {
    let source = r#"fun test() {
        let x: Int? = null
        let y: Int = x ?: 42
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 2);

    // Check the Elvis operator expression
    if let Statement::Let { initializer, .. } = &get_statements(&func.body)[1] {
        assert!(matches!(initializer, Expression::Elvis { .. }));
    } else {
        panic!("Expected let statement with Elvis operator");
    }
}

#[test]
fn test_parse_chained_elvis() {
    let source = r#"fun test() {
        let a: Int? = null
        let b: Int? = null
        let c: Int = a ?: b ?: 0
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    // Should parse successfully with nested Elvis
    assert_eq!(get_statements(&func.body).len(), 3);
}

#[test]
fn test_parse_safe_call() {
    let source = r#"fun test() {
        let x: Text? = null
        let len: Int? = x?.length()
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 2);

    // Check the SafeCall expression
    if let Statement::Let { initializer, .. } = &get_statements(&func.body)[1] {
        assert!(matches!(initializer, Expression::SafeCall { .. }));
    } else {
        panic!("Expected let statement with SafeCall operator");
    }
}

#[test]
fn test_parse_chained_safe_call() {
    let source = r#"fun test() {
        let result: Int? = user?.profile?.age()
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    // Should parse successfully with chained safe calls
    assert_eq!(get_statements(&func.body).len(), 1);
}

#[test]
fn test_parse_elvis_with_safe_call() {
    let source = r#"fun test() {
        let name: Text = user?.name ?: "Anonymous"
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 1);

    // Should have Elvis with SafeCall inside
    if let Statement::Let { initializer, .. } = &get_statements(&func.body)[0] {
        if let Expression::Elvis { value, .. } = initializer {
            assert!(matches!(&**value, Expression::SafeCall { .. }));
        } else {
            panic!("Expected Elvis operator");
        }
    } else {
        panic!("Expected let statement");
    }
}

#[test]
fn test_parse_block_expression() {
    let source = r#"fun test() {
        let x: Int = {
            let temp = 10
            temp * 2
        }
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 1);

    // Check block expression
    if let Statement::Let { initializer, .. } = &get_statements(&func.body)[0] {
        assert!(matches!(initializer, Expression::Block(_)));
    } else {
        panic!("Expected let statement with block expression");
    }
}

#[test]
fn test_parse_elvis_with_block() {
    let source = r#"fun test() {
        let x: Int? = null
        let y: Int = x ?: {
            print("Using fallback")
            42
        }
    }"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::Function(func) = &program.items[0] else { panic!("Expected function item") };
    assert_eq!(get_statements(&func.body).len(), 2);

    // Elvis with block fallback
    if let Statement::Let { initializer, .. } = &get_statements(&func.body)[1] {
        if let Expression::Elvis { fallback, .. } = initializer {
            assert!(matches!(&**fallback, Expression::Block(_)));
        } else {
            panic!("Expected Elvis operator");
        }
    } else {
        panic!("Expected let statement");
    }
}
