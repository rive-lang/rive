//! Common parsing helper functions.

use super::parser::Parser;
use rive_core::{Error, Result};
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses a literal value into an i64.
    pub(crate) fn parse_i64_literal(&mut self) -> Result<i64> {
        let token = self.peek();
        let value = token
            .0
            .text
            .parse()
            .map_err(|_| Error::Parser("Invalid integer literal".to_string(), token.1))?;
        self.advance();
        Ok(value)
    }

    /// Parses a literal value into an f64.
    pub(crate) fn parse_f64_literal(&mut self) -> Result<f64> {
        let token = self.peek();
        let value = token
            .0
            .text
            .parse()
            .map_err(|_| Error::Parser("Invalid float literal".to_string(), token.1))?;
        self.advance();
        Ok(value)
    }

    /// Parses a string literal (removes surrounding quotes).
    pub(crate) fn parse_string_content(&mut self) -> Result<String> {
        let token = self.peek();
        let value = token.0.text[1..token.0.text.len() - 1].to_string();
        self.advance();
        Ok(value)
    }

    /// Parses a range operator (.. or ..=) and returns whether it's inclusive.
    pub(crate) fn parse_range_operator(&mut self) -> Result<bool> {
        if self.check(&TokenKind::DotDotEq) {
            self.advance();
            Ok(true)
        } else if self.check(&TokenKind::DotDot) {
            self.advance();
            Ok(false)
        } else {
            Err(Error::Parser(
                "Expected '..' or '..='".to_string(),
                self.current_span(),
            ))
        }
    }

    /// Parses a condition expression (with optional parentheses).
    pub(crate) fn parse_condition(&mut self) -> Result<crate::ast::Expression> {
        let has_paren = self.match_token(&TokenKind::LeftParen);
        let condition = self.parse_expression()?;
        if has_paren {
            self.expect(&TokenKind::RightParen)?;
        }
        Ok(condition)
    }
}
