//! Tests for the Rive lexer.

use rive_lexer::{TokenKind, tokenize};

#[test]
fn test_keywords() {
    let source = "let mut fun if else while for return break continue";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 10);
    assert!(matches!(tokens[0].0.kind, TokenKind::Let));
    assert!(matches!(tokens[1].0.kind, TokenKind::Mut));
    assert!(matches!(tokens[2].0.kind, TokenKind::Fun));
    assert!(matches!(tokens[3].0.kind, TokenKind::If));
    assert!(matches!(tokens[4].0.kind, TokenKind::Else));
    assert!(matches!(tokens[5].0.kind, TokenKind::While));
    assert!(matches!(tokens[6].0.kind, TokenKind::For));
    assert!(matches!(tokens[7].0.kind, TokenKind::Return));
    assert!(matches!(tokens[8].0.kind, TokenKind::Break));
    assert!(matches!(tokens[9].0.kind, TokenKind::Continue));
}

#[test]
fn test_literals() {
    let source = r#"42 3.14 "hello" true false null"#;
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 6);
    assert!(matches!(tokens[0].0.kind, TokenKind::Integer));
    assert_eq!(tokens[0].0.text, "42");

    assert!(matches!(tokens[1].0.kind, TokenKind::Float));
    assert_eq!(tokens[1].0.text, "3.14");

    assert!(matches!(tokens[2].0.kind, TokenKind::String));
    assert_eq!(tokens[2].0.text, r#""hello""#);

    assert!(matches!(tokens[3].0.kind, TokenKind::True));
    assert!(matches!(tokens[4].0.kind, TokenKind::False));
    assert!(matches!(tokens[5].0.kind, TokenKind::Null));
}

#[test]
fn test_operators() {
    let source = "+ - * / % = == != < <= > >= && || !";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 15);
    assert!(matches!(tokens[0].0.kind, TokenKind::Plus));
    assert!(matches!(tokens[1].0.kind, TokenKind::Minus));
    assert!(matches!(tokens[2].0.kind, TokenKind::Star));
    assert!(matches!(tokens[3].0.kind, TokenKind::Slash));
    assert!(matches!(tokens[4].0.kind, TokenKind::Percent));
    assert!(matches!(tokens[5].0.kind, TokenKind::Equal));
    assert!(matches!(tokens[6].0.kind, TokenKind::EqualEqual));
    assert!(matches!(tokens[7].0.kind, TokenKind::BangEqual));
    assert!(matches!(tokens[8].0.kind, TokenKind::Less));
    assert!(matches!(tokens[9].0.kind, TokenKind::LessEqual));
    assert!(matches!(tokens[10].0.kind, TokenKind::Greater));
    assert!(matches!(tokens[11].0.kind, TokenKind::GreaterEqual));
    assert!(matches!(tokens[12].0.kind, TokenKind::AmpersandAmpersand));
    assert!(matches!(tokens[13].0.kind, TokenKind::PipePipe));
    assert!(matches!(tokens[14].0.kind, TokenKind::Bang));
}

#[test]
fn test_simple_function() {
    let source = r#"
        fun greet(name: Text): Text {
            print("Hello, " + name)
        }
    "#;

    let tokens = tokenize(source).unwrap();

    // fun greet ( name : Text ) : Text { print ( "Hello, " + name ) }
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::Fun)));
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.0.kind, TokenKind::Identifier))
    );
    // Type names are now parsed as identifiers
    assert!(tokens.iter().any(|t| t.0.text == "Text"));
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::Print)));
}

#[test]
fn test_variable_declaration() {
    let source = "let x = 42";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 4);
    assert!(matches!(tokens[0].0.kind, TokenKind::Let));
    assert!(matches!(tokens[1].0.kind, TokenKind::Identifier));
    assert_eq!(tokens[1].0.text, "x");
    assert!(matches!(tokens[2].0.kind, TokenKind::Equal));
    assert!(matches!(tokens[3].0.kind, TokenKind::Integer));
    assert_eq!(tokens[3].0.text, "42");
}

#[test]
fn test_mutable_variable() {
    let source = "let mut count = 0";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 5);
    assert!(matches!(tokens[0].0.kind, TokenKind::Let));
    assert!(matches!(tokens[1].0.kind, TokenKind::Mut));
    assert!(matches!(tokens[2].0.kind, TokenKind::Identifier));
    assert_eq!(tokens[2].0.text, "count");
}

#[test]
fn test_optional_type() {
    let source = "let age?: Optional<Int> = null";
    let tokens = tokenize(source).unwrap();

    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.0.kind, TokenKind::Question))
    );
    // Type names are now parsed as identifiers
    assert!(tokens.iter().any(|t| t.0.text == "Optional"));
    assert!(tokens.iter().any(|t| t.0.text == "Int"));
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::Null)));
}

#[test]
fn test_comments_ignored() {
    let source = r#"
        let x = 42 // This is a comment
        // Another comment
        let y = 10
    "#;

    let tokens = tokenize(source).unwrap();

    // Should only contain tokens, not comments
    assert!(!tokens.iter().any(|t| t.0.text.contains("//")));

    // Verify actual tokens
    assert!(tokens.iter().any(|t| t.0.text == "x"));
    assert!(tokens.iter().any(|t| t.0.text == "y"));
}

#[test]
fn test_array_syntax() {
    let source = "[Int; 10]";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 5);
    assert!(matches!(tokens[0].0.kind, TokenKind::LeftBracket));
    // Type names are now parsed as identifiers
    assert!(matches!(tokens[1].0.kind, TokenKind::Identifier));
    assert_eq!(tokens[1].0.text, "Int");
    assert!(matches!(tokens[2].0.kind, TokenKind::Semicolon));
    assert!(matches!(tokens[3].0.kind, TokenKind::Integer));
    assert!(matches!(tokens[4].0.kind, TokenKind::RightBracket));
}

#[test]
fn test_negative_numbers() {
    let source = "-42 -3.14";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].0.kind, TokenKind::Integer));
    assert_eq!(tokens[0].0.text, "-42");
    assert!(matches!(tokens[1].0.kind, TokenKind::Float));
    assert_eq!(tokens[1].0.text, "-3.14");
}

#[test]
fn test_control_flow_keywords() {
    let source = "loop match in";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0].0.kind, TokenKind::Loop));
    assert_eq!(tokens[0].0.text, "loop");
    assert!(matches!(tokens[1].0.kind, TokenKind::Match));
    assert_eq!(tokens[1].0.text, "match");
    assert!(matches!(tokens[2].0.kind, TokenKind::In));
    assert_eq!(tokens[2].0.text, "in");
}

#[test]
fn test_range_operators() {
    let source = "1..10 1..=10";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 6);
    assert!(matches!(tokens[0].0.kind, TokenKind::Integer));
    assert!(matches!(tokens[1].0.kind, TokenKind::DotDot));
    assert_eq!(tokens[1].0.text, "..");
    assert!(matches!(tokens[2].0.kind, TokenKind::Integer));
    assert!(matches!(tokens[3].0.kind, TokenKind::Integer));
    assert!(matches!(tokens[4].0.kind, TokenKind::DotDotEq));
    assert_eq!(tokens[4].0.text, "..=");
    assert!(matches!(tokens[5].0.kind, TokenKind::Integer));
}

#[test]
fn test_match_arrow() {
    let source = "x -> y";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0].0.kind, TokenKind::Identifier));
    assert!(matches!(tokens[1].0.kind, TokenKind::Arrow));
    assert_eq!(tokens[1].0.text, "->");
    assert!(matches!(tokens[2].0.kind, TokenKind::Identifier));
}

#[test]
fn test_underscore_wildcard() {
    let source = "_ _x x_y";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 3);
    // Standalone underscore should be Underscore token
    assert!(matches!(tokens[0].0.kind, TokenKind::Underscore));
    assert_eq!(tokens[0].0.text, "_");
    // _x and x_y should be identifiers
    assert!(matches!(tokens[1].0.kind, TokenKind::Identifier));
    assert_eq!(tokens[1].0.text, "_x");
    assert!(matches!(tokens[2].0.kind, TokenKind::Identifier));
    assert_eq!(tokens[2].0.text, "x_y");
}

#[test]
fn test_loop_expression() {
    let source = r#"
        loop {
            if x > 10 {
                break
            }
        }
    "#;

    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::Loop)));
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::If)));
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::Break)));
}

#[test]
fn test_for_loop() {
    let source = "for i in 1..10 { }";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::For)));
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::In)));
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::DotDot)));
}

#[test]
fn test_match_expression() {
    let source = r#"
        match x {
            1 -> "one"
            2 -> "two"
            _ -> "other"
        }
    "#;

    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::Match)));
    assert!(tokens.iter().any(|t| matches!(t.0.kind, TokenKind::Arrow)));
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.0.kind, TokenKind::Underscore))
    );
}
