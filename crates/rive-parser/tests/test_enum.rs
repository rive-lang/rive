//! Tests for enum parsing.

use rive_lexer::tokenize;
use rive_parser::Parser;

#[test]
fn test_simple_enum_declaration() {
    let source = r#"
        enum Color {
            Red,
            Green,
            Blue
        }
    "#;

    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        rive_parser::ast::Item::EnumDecl(enum_decl) => {
            assert_eq!(enum_decl.name, "Color");
            assert_eq!(enum_decl.variants.len(), 3);
            assert_eq!(enum_decl.variants[0].name, "Red");
            assert_eq!(enum_decl.variants[1].name, "Green");
            assert_eq!(enum_decl.variants[2].name, "Blue");
            assert!(enum_decl.variants[0].fields.is_none());
        }
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_enum_with_fields() {
    let source = r#"
        enum HttpStatus {
            Ok(code: Int),
            NotFound(code: Int, message: Text),
            ServerError(code: Int, message: Text)
        }
    "#;

    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        rive_parser::ast::Item::EnumDecl(enum_decl) => {
            assert_eq!(enum_decl.name, "HttpStatus");
            assert_eq!(enum_decl.variants.len(), 3);

            // Check Ok variant
            assert_eq!(enum_decl.variants[0].name, "Ok");
            let ok_fields = enum_decl.variants[0].fields.as_ref().unwrap();
            assert_eq!(ok_fields.len(), 1);
            assert_eq!(ok_fields[0].name, "code");

            // Check NotFound variant
            assert_eq!(enum_decl.variants[1].name, "NotFound");
            let not_found_fields = enum_decl.variants[1].fields.as_ref().unwrap();
            assert_eq!(not_found_fields.len(), 2);
            assert_eq!(not_found_fields[0].name, "code");
            assert_eq!(not_found_fields[1].name, "message");
        }
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_enum_variant_construction() {
    let source = r#"
        fun test() {
            let red = Color.Red
            let custom = Color.RGB(255, 128, 0)
        }
    "#;

    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        rive_parser::ast::Item::Function(func) => {
            assert_eq!(func.name, "test");
            match &func.body {
                rive_parser::ast::FunctionBody::Block(block) => {
                    assert_eq!(block.statements.len(), 2);

                    // Check first statement: let red = Color.Red
                    match &block.statements[0] {
                        rive_parser::ast::Statement::Let { initializer, .. } => match initializer {
                            rive_parser::ast::Expression::EnumVariant {
                                enum_name,
                                variant_name,
                                arguments,
                                ..
                            } => {
                                assert_eq!(enum_name, "Color");
                                assert_eq!(variant_name, "Red");
                                assert_eq!(arguments.len(), 0);
                            }
                            _ => panic!("Expected EnumVariant expression"),
                        },
                        _ => panic!("Expected Let statement"),
                    }

                    // Check second statement: let custom = Color.RGB(255, 128, 0)
                    match &block.statements[1] {
                        rive_parser::ast::Statement::Let { initializer, .. } => match initializer {
                            rive_parser::ast::Expression::EnumVariant {
                                enum_name,
                                variant_name,
                                arguments,
                                ..
                            } => {
                                assert_eq!(enum_name, "Color");
                                assert_eq!(variant_name, "RGB");
                                assert_eq!(arguments.len(), 3);
                            }
                            _ => panic!("Expected EnumVariant expression"),
                        },
                        _ => panic!("Expected Let statement"),
                    }
                }
                _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_enum_variant_with_named_arguments() {
    let source = r#"
        fun test() {
            let event = NetworkEvent.Connected(url = "https://github.com")
        }
    "#;

    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        rive_parser::ast::Item::Function(func) => {
            match &func.body {
                rive_parser::ast::FunctionBody::Block(block) => {
                    match &block.statements[0] {
                        rive_parser::ast::Statement::Let { initializer, .. } => {
                            match initializer {
                                rive_parser::ast::Expression::EnumVariant {
                                    enum_name,
                                    variant_name,
                                    arguments,
                                    ..
                                } => {
                                    assert_eq!(enum_name, "NetworkEvent");
                                    assert_eq!(variant_name, "Connected");
                                    assert_eq!(arguments.len(), 1);

                                    // Check that it's a named argument
                                    match &arguments[0] {
                                        rive_parser::ast::Argument::Named { name, .. } => {
                                            assert_eq!(name, "url");
                                        }
                                        _ => panic!("Expected named argument"),
                                    }
                                }
                                _ => panic!("Expected EnumVariant expression"),
                            }
                        }
                        _ => panic!("Expected Let statement"),
                    }
                }
                _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_enum_pattern_matching() {
    let source = r#"
        enum HttpStatus {
            Ok(code: Int),
            NotFound(code: Int, message: Text)
        }
        
        fun test() {
            let status = HttpStatus.Ok(code = 200)
            when status {
                HttpStatus.Ok(code) -> print(code)
                HttpStatus.NotFound(code, message) -> print(message)
                _ -> print("unknown")
            }
        }
    "#;

    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 2); // enum + function
    match &program.items[1] {
        rive_parser::ast::Item::Function(func) => {
            match &func.body {
                rive_parser::ast::FunctionBody::Block(block) => {
                    match &block.statements[1] {
                        // second statement is the when expression
                        rive_parser::ast::Statement::Expression { expression, .. } => {
                            match expression {
                                rive_parser::ast::Expression::Match(match_expr) => {
                                    assert_eq!(match_expr.arms.len(), 3);

                                    // Check first arm: HttpStatus.Ok(code)
                                    match &match_expr.arms[0].pattern {
                                        rive_parser::control_flow::Pattern::EnumVariant {
                                            enum_name,
                                            variant_name,
                                            bindings,
                                            ..
                                        } => {
                                            assert_eq!(enum_name, "HttpStatus");
                                            assert_eq!(variant_name, "Ok");
                                            let bindings = bindings.as_ref().unwrap();
                                            assert_eq!(bindings.len(), 1);
                                            assert_eq!(bindings[0].0, "code");
                                            assert!(bindings[0].1.is_none()); // No renaming
                                        }
                                        _ => panic!("Expected EnumVariant pattern"),
                                    }

                                    // Check third arm: wildcard
                                    match &match_expr.arms[2].pattern {
                                        rive_parser::control_flow::Pattern::Wildcard { .. } => {}
                                        _ => panic!("Expected Wildcard pattern"),
                                    }
                                }
                                _ => panic!("Expected Match expression"),
                            }
                        }
                        _ => panic!("Expected Expression statement"),
                    }
                }
                _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_pattern_with_guard() {
    let source = r#"
        enum Status {
            Ok(code: Int)
        }
        
        fun test() {
            let status = Status.Ok(code = 200)
            when status {
                Status.Ok(code) if code == 200 -> print("success")
                _ -> print("other")
            }
        }
    "#;

    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 2); // enum + function
    match &program.items[1] {
        rive_parser::ast::Item::Function(func) => {
            match &func.body {
                rive_parser::ast::FunctionBody::Block(block) => {
                    match &block.statements[1] {
                        // second statement is the when expression
                        rive_parser::ast::Statement::Expression { expression, .. } => {
                            match expression {
                                rive_parser::ast::Expression::Match(match_expr) => {
                                    // Check first arm has a guard
                                    match &match_expr.arms[0].pattern {
                                        rive_parser::control_flow::Pattern::Guarded {
                                            guard,
                                            ..
                                        } => {
                                            // Guard should be a binary expression
                                            assert!(matches!(
                                                guard.as_ref(),
                                                rive_parser::ast::Expression::Binary { .. }
                                            ));
                                        }
                                        _ => panic!("Expected Guarded pattern"),
                                    }
                                }
                                _ => panic!("Expected Match expression"),
                            }
                        }
                        _ => panic!("Expected Expression statement"),
                    }
                }
                _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_multi_value_pattern() {
    let source = r#"
        fun test(x: Int) {
            when x {
                404, 410 -> print("not found")
                _ -> print("other")
            }
        }
    "#;

    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    match &program.items[0] {
        rive_parser::ast::Item::Function(func) => {
            match &func.body {
                rive_parser::ast::FunctionBody::Block(block) => {
                    match &block.statements[0] {
                        rive_parser::ast::Statement::Expression { expression, .. } => {
                            match expression {
                                rive_parser::ast::Expression::Match(match_expr) => {
                                    // Check first arm has multiple patterns
                                    match &match_expr.arms[0].pattern {
                                        rive_parser::control_flow::Pattern::Multiple {
                                            patterns,
                                            ..
                                        } => {
                                            assert_eq!(patterns.len(), 2);
                                            match &patterns[0] {
                                                rive_parser::control_flow::Pattern::Integer {
                                                    value,
                                                    ..
                                                } => {
                                                    assert_eq!(*value, 404);
                                                }
                                                _ => panic!("Expected Integer pattern"),
                                            }
                                            match &patterns[1] {
                                                rive_parser::control_flow::Pattern::Integer {
                                                    value,
                                                    ..
                                                } => {
                                                    assert_eq!(*value, 410);
                                                }
                                                _ => panic!("Expected Integer pattern"),
                                            }
                                        }
                                        _ => panic!("Expected Multiple pattern"),
                                    }
                                }
                                _ => panic!("Expected Match expression"),
                            }
                        }
                        _ => panic!("Expected Expression statement"),
                    }
                }
                _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected Function"),
    }
}
