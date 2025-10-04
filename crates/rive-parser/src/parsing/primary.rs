//! Primary expression parsing (literals, variables, arrays, control flow).

use super::parser::Parser;
use crate::ast::Expression;
use rive_core::{Error, Result};
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses a primary expression.
    pub(crate) fn parse_primary(&mut self) -> Result<Expression> {
        let span = self.current_span();
        let token = self.peek().clone();

        match &token.0.kind {
            // Literals
            TokenKind::Integer => self.parse_integer_literal(),
            TokenKind::Float => self.parse_float_literal(),
            TokenKind::String => self.parse_string_literal(),
            TokenKind::True => {
                self.advance();
                Ok(Expression::Boolean { value: true, span })
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Boolean { value: false, span })
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Null { span })
            }
            // Variables
            TokenKind::Identifier | TokenKind::Print => {
                self.advance();
                Ok(Expression::Variable {
                    name: token.0.text.clone(),
                    span,
                })
            }
            // Parenthesized expression
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(expr)
            }
            // Array literal
            TokenKind::LeftBracket => {
                self.advance();
                let elements = self.parse_array_elements()?;
                let end_span = self.expect(&TokenKind::RightBracket)?;
                Ok(Expression::Array {
                    elements,
                    span: span.merge(end_span),
                })
            }
            // Control flow expressions
            TokenKind::If => Ok(Expression::If(Box::new(self.parse_if()?))),
            TokenKind::While => Ok(Expression::While(Box::new(self.parse_while()?))),
            TokenKind::For => Ok(Expression::For(Box::new(self.parse_for()?))),
            TokenKind::Loop => Ok(Expression::Loop(Box::new(self.parse_loop()?))),
            TokenKind::Match => Ok(Expression::Match(Box::new(self.parse_match()?))),
            _ => {
                let span = self.current_span();
                Err(Error::Parser(
                    format!("Unexpected token '{}'", token.0.text),
                    span,
                ))
            }
        }
    }

    /// Parses an integer literal.
    fn parse_integer_literal(&mut self) -> Result<Expression> {
        let token = self.peek();
        let span = self.current_span();
        let value = token
            .0
            .text
            .parse::<i64>()
            .map_err(|_| Error::Parser(format!("Invalid integer: {}", token.0.text), span))?;
        self.advance();
        Ok(Expression::Integer { value, span })
    }

    /// Parses a float literal.
    fn parse_float_literal(&mut self) -> Result<Expression> {
        let token = self.peek();
        let span = self.current_span();
        let value = token
            .0
            .text
            .parse::<f64>()
            .map_err(|_| Error::Parser(format!("Invalid float: {}", token.0.text), span))?;
        self.advance();
        Ok(Expression::Float { value, span })
    }

    /// Parses a string literal.
    fn parse_string_literal(&mut self) -> Result<Expression> {
        let token = self.peek();
        let span = self.current_span();
        // Remove surrounding quotes
        let value = token.0.text[1..token.0.text.len() - 1].to_string();
        self.advance();
        Ok(Expression::String { value, span })
    }

    /// Parses array elements.
    fn parse_array_elements(&mut self) -> Result<Vec<Expression>> {
        let mut elements = Vec::new();

        if !self.check(&TokenKind::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(elements)
    }
}
