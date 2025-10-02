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
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.0.kind, TokenKind::TypeText))
    );
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
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.0.kind, TokenKind::TypeOptional))
    );
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.0.kind, TokenKind::TypeInt))
    );
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
    assert!(matches!(tokens[1].0.kind, TokenKind::TypeInt));
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
