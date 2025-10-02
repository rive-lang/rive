//! Lexical analysis for the Rive language.
//!
//! This crate provides tokenization of Rive source code into a stream of tokens.

mod token;

pub use token::{Token, TokenKind};

use logos::Logos;
use rive_core::{Error, Result, Span};

/// Tokenizes Rive source code into a vector of tokens.
///
/// # Arguments
/// * `source` - The source code to tokenize
///
/// # Returns
/// A vector of tokens with their spans
///
/// # Errors
/// Returns an error if the source contains invalid tokens
///
/// # Examples
/// ```
/// use rive_lexer::tokenize;
///
/// let source = "let x = 42;";
/// let tokens = tokenize(source).unwrap();
/// ```
pub fn tokenize(source: &str) -> Result<Vec<(Token, Span)>> {
    let mut tokens = Vec::new();
    let mut lexer = TokenKind::lexer(source);

    while let Some(result) = lexer.next() {
        let kind = result.map_err(|_| {
            Error::Lexer(format!("Invalid token at position {}", lexer.span().start))
        })?;

        let span = lexer.span();
        let text = lexer.slice().to_string();

        tokens.push((Token { kind, text }, Span::from_range(span.start, span.end)));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let source = "let x = 42";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].0.kind, TokenKind::Let));
        assert!(matches!(tokens[1].0.kind, TokenKind::Identifier));
        assert!(matches!(tokens[2].0.kind, TokenKind::Equal));
        assert!(matches!(tokens[3].0.kind, TokenKind::Integer));
    }
}
