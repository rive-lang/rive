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
            // Variables or labeled loops
            TokenKind::Identifier | TokenKind::Print => {
                // Check if this is a labeled loop: `label: for/while/loop`
                if self.check_ahead(1, &TokenKind::Colon) {
                    let label = token.0.text.clone();
                    self.advance(); // consume identifier
                    self.advance(); // consume colon

                    // Parse the loop with the label
                    match self.peek().0.kind {
                        TokenKind::For => {
                            Ok(Expression::For(Box::new(self.parse_for(Some(label))?)))
                        }
                        TokenKind::While => {
                            Ok(Expression::While(Box::new(self.parse_while(Some(label))?)))
                        }
                        TokenKind::Loop => {
                            Ok(Expression::Loop(Box::new(self.parse_loop(Some(label))?)))
                        }
                        _ => {
                            let span = self.current_span();
                            Err(Error::Parser(
                                "Expected 'for', 'while', or 'loop' after label".to_string(),
                                span,
                            ))
                        }
                    }
                } else {
                    self.advance();
                    Ok(Expression::Variable {
                        name: token.0.text.clone(),
                        span,
                    })
                }
            }
            // Parenthesized expression or tuple literal
            TokenKind::LeftParen => self.parse_paren_or_tuple(),
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
            TokenKind::While => Ok(Expression::While(Box::new(self.parse_while(None)?))),
            TokenKind::For => Ok(Expression::For(Box::new(self.parse_for(None)?))),
            TokenKind::Loop => Ok(Expression::Loop(Box::new(self.parse_loop(None)?))),
            TokenKind::When => Ok(Expression::Match(Box::new(self.parse_match()?))),
            // Dict literal or block expression
            TokenKind::LeftBrace => self.parse_brace_expression(),
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
        use crate::ast::StringPart;

        let token = self.peek();
        let span = self.current_span();
        // Remove surrounding quotes
        let raw_value = token.0.text[1..token.0.text.len() - 1].to_string();
        self.advance();

        // Check if string contains interpolation markers ($)
        if !raw_value.contains('$') {
            // Simple string without interpolation
            return Ok(Expression::String {
                value: raw_value,
                span,
            });
        }

        // Parse string interpolation
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = raw_value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                // Save current literal part if not empty
                if !current.is_empty() {
                    parts.push(StringPart::Literal(current.clone()));
                    current.clear();
                }

                // Check if it's ${expression} or $identifier
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'

                    // Collect expression until '}'
                    let mut expr_str = String::new();
                    let mut depth = 1;
                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            depth += 1;
                            expr_str.push(ch);
                        } else if ch == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            expr_str.push(ch);
                        } else {
                            expr_str.push(ch);
                        }
                    }

                    // Parse the expression
                    let expr_tokens = rive_lexer::tokenize(&expr_str).map_err(|_| {
                        Error::Parser(
                            format!("Failed to parse interpolated expression: {}", expr_str),
                            span,
                        )
                    })?;
                    let mut expr_parser = crate::Parser::new(&expr_tokens);
                    let expr = expr_parser.parse_expression()?;
                    parts.push(StringPart::Interpolation(Box::new(expr)));
                } else {
                    // Simple $identifier
                    let mut ident = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            ident.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    if ident.is_empty() {
                        return Err(Error::Parser(
                            "Expected identifier or '{' after '$'".to_string(),
                            span,
                        ));
                    }

                    // Create a variable reference expression
                    let var_expr = Expression::Variable { name: ident, span };
                    parts.push(StringPart::Interpolation(Box::new(var_expr)));
                }
            } else if ch == '\\' {
                // Handle escape sequences
                if let Some(&next_ch) = chars.peek() {
                    chars.next();
                    match next_ch {
                        'n' => current.push('\n'),
                        't' => current.push('\t'),
                        'r' => current.push('\r'),
                        '\\' => current.push('\\'),
                        '"' => current.push('"'),
                        '$' => current.push('$'),
                        _ => {
                            current.push('\\');
                            current.push(next_ch);
                        }
                    }
                } else {
                    current.push('\\');
                }
            } else {
                current.push(ch);
            }
        }

        // Add final literal part if not empty
        if !current.is_empty() {
            parts.push(StringPart::Literal(current));
        }

        // If no interpolation parts were found, return simple string
        if parts.is_empty() {
            return Ok(Expression::String {
                value: String::new(),
                span,
            });
        }

        // If only one literal part, return simple string
        if parts.len() == 1 {
            if let StringPart::Literal(s) = &parts[0] {
                return Ok(Expression::String {
                    value: s.clone(),
                    span,
                });
            }
        }

        Ok(Expression::StringInterpolation { parts, span })
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

    /// Parses parenthesized expression or tuple literal.
    ///
    /// Disambiguates:
    /// - `(expr)` - grouped expression
    /// - `(expr,)` - single-element tuple
    /// - `(a, b, c)` - multi-element tuple
    /// - `()` - empty tuple (unit)
    fn parse_paren_or_tuple(&mut self) -> Result<Expression> {
        let start_span = self.current_span();
        self.advance(); // consume `(`

        // Empty tuple `()`
        if self.check(&TokenKind::RightParen) {
            let end_span = self.expect(&TokenKind::RightParen)?;
            return Ok(Expression::Tuple {
                elements: Vec::new(),
                span: start_span.merge(end_span),
            });
        }

        // Parse first expression
        let first = self.parse_expression()?;

        // Check for comma
        if self.match_token(&TokenKind::Comma) {
            // It's a tuple
            let mut elements = vec![first];

            // Parse remaining elements
            if !self.check(&TokenKind::RightParen) {
                loop {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                    // Allow trailing comma
                    if self.check(&TokenKind::RightParen) {
                        break;
                    }
                }
            }

            let end_span = self.expect(&TokenKind::RightParen)?;
            Ok(Expression::Tuple {
                elements,
                span: start_span.merge(end_span),
            })
        } else {
            // Just a grouped expression
            self.expect(&TokenKind::RightParen)?;
            Ok(first)
        }
    }

    /// Parses dict literal or block expression based on lookahead.
    ///
    /// Context-sensitive parsing:
    /// - `{}` - empty dict in expression context
    /// - `{"key": value, ...}` - dict literal
    /// - `{ statements... }` - block expression (fallback)
    fn parse_brace_expression(&mut self) -> Result<Expression> {
        let start_span = self.current_span();
        self.advance(); // consume `{`

        // Empty braces `{}` - treat as empty dict in expression context
        if self.check(&TokenKind::RightBrace) {
            let end_span = self.expect(&TokenKind::RightBrace)?;
            return Ok(Expression::Dict {
                entries: Vec::new(),
                span: start_span.merge(end_span),
            });
        }

        // Lookahead to distinguish dict from block
        // Dict: first token is String followed by Colon
        if self.check(&TokenKind::String) && self.check_ahead(1, &TokenKind::Colon) {
            // Parse as dict literal
            let mut entries = Vec::new();

            loop {
                // Parse key (must be string literal)
                let key_token = self.peek();
                let key_span = self.current_span();
                if key_token.0.kind != TokenKind::String {
                    return Err(Error::Parser(
                        "Dictionary keys must be string literals".to_string(),
                        key_span,
                    ));
                }
                let key = key_token.0.text[1..key_token.0.text.len() - 1].to_string();
                self.advance();

                // Expect colon
                self.expect(&TokenKind::Colon)?;

                // Parse value
                let value = self.parse_expression()?;
                entries.push((key, value));

                // Check for comma
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }

                // Allow trailing comma
                if self.check(&TokenKind::RightBrace) {
                    break;
                }
            }

            let end_span = self.expect(&TokenKind::RightBrace)?;
            Ok(Expression::Dict {
                entries,
                span: start_span.merge(end_span),
            })
        } else {
            // Parse as block expression
            // Rewind by creating a new parser state would be complex,
            // so we'll parse statements until we hit `}`
            let mut statements = Vec::new();

            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }

            let end_span = self.expect(&TokenKind::RightBrace)?;
            Ok(Expression::Block(Box::new(crate::ast::Block {
                statements,
                span: start_span.merge(end_span),
            })))
        }
    }
}
