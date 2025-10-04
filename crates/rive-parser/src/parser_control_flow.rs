//! Control flow parsing logic for Rive.
//!
//! This module implements parsing for all control flow constructs:
//! - If expressions/statements
//! - Loop constructs (while, for, loop)
//! - Match expressions
//! - Break and continue statements
//! - Range expressions
//! - Patterns for match expressions

use rive_core::{Error, Result};
use rive_lexer::TokenKind;

use crate::{
    ast::Expression,
    control_flow::{
        Break, Continue, ElseIf, For, If, Loop, Match, MatchArm, Pattern, Range, While,
    },
    parser::Parser,
};

impl<'a> Parser<'a> {
    /// Parses an if expression/statement.
    ///
    /// Syntax:
    /// ```text
    /// if condition { ... }
    /// if condition { ... } else { ... }
    /// if condition { ... } else if condition2 { ... } else { ... }
    /// ```
    pub(crate) fn parse_if(&mut self) -> Result<If> {
        let start = self.expect(&TokenKind::If)?;

        // Parse condition (parentheses optional)
        let has_paren = self.check(&TokenKind::LeftParen);
        if has_paren {
            self.advance();
        }

        let condition = Box::new(self.parse_expression()?);

        if has_paren {
            self.expect(&TokenKind::RightParen)?;
        }

        // Parse then block
        let then_block = self.parse_block()?;

        // Parse else-if chain
        let mut else_if_branches = Vec::new();
        while self.check(&TokenKind::Else) && self.check_ahead(1, &TokenKind::If) {
            self.advance(); // consume 'else'
            self.advance(); // consume 'if'

            let has_paren = self.check(&TokenKind::LeftParen);
            if has_paren {
                self.advance();
            }

            let condition = Box::new(self.parse_expression()?);

            if has_paren {
                self.expect(&TokenKind::RightParen)?;
            }

            let block = self.parse_block()?;
            let block_span = block.span;

            else_if_branches.push(ElseIf {
                condition,
                block,
                span: start.merge(block_span),
            });
        }

        // Parse optional else block
        let else_block = if self.check(&TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = else_block.as_ref().map_or(then_block.span, |b| b.span);

        Ok(If {
            condition,
            then_block,
            else_if_branches,
            else_block,
            span: start.merge(end),
        })
    }

    /// Parses a while loop.
    ///
    /// Syntax:
    /// ```text
    /// while condition { ... }
    /// ```
    pub(crate) fn parse_while(&mut self) -> Result<While> {
        let start = self.expect(&TokenKind::While)?;

        // Parse condition (parentheses optional)
        let has_paren = self.check(&TokenKind::LeftParen);
        if has_paren {
            self.advance();
        }

        let condition = Box::new(self.parse_expression()?);

        if has_paren {
            self.expect(&TokenKind::RightParen)?;
        }

        // Parse body
        let body = self.parse_block()?;
        let end = body.span;

        Ok(While {
            condition,
            body,
            span: start.merge(end),
        })
    }

    /// Parses a for loop.
    ///
    /// Syntax:
    /// ```text
    /// for variable in iterable { ... }
    /// ```
    pub(crate) fn parse_for(&mut self) -> Result<For> {
        let start = self.expect(&TokenKind::For)?;

        // Parse iterator variable
        let var_token = self.peek();
        let variable = var_token.0.text.clone();
        self.expect(&TokenKind::Identifier)?;

        // Expect 'in' keyword
        self.expect(&TokenKind::In)?;

        // Parse iterable expression (typically a range)
        let iterable = Box::new(self.parse_expression()?);

        // Parse body
        let body = self.parse_block()?;
        let end = body.span;

        Ok(For {
            variable,
            iterable,
            body,
            span: start.merge(end),
        })
    }

    /// Parses an infinite loop.
    ///
    /// Syntax:
    /// ```text
    /// loop { ... }
    /// ```
    pub(crate) fn parse_loop(&mut self) -> Result<Loop> {
        let start = self.expect(&TokenKind::Loop)?;

        // Parse body
        let body = self.parse_block()?;
        let end = body.span;

        Ok(Loop {
            body,
            span: start.merge(end),
        })
    }

    /// Parses a match expression.
    ///
    /// Syntax:
    /// ```text
    /// match expr {
    ///     pattern -> expression,
    ///     pattern -> expression,
    /// }
    /// ```
    pub(crate) fn parse_match(&mut self) -> Result<Match> {
        let start = self.expect(&TokenKind::Match)?;

        // Parse scrutinee expression
        let scrutinee = Box::new(self.parse_expression()?);

        // Expect opening brace
        self.expect(&TokenKind::LeftBrace)?;

        // Parse match arms
        let mut arms = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let arm_start = self.peek().1;

            // Parse pattern
            let pattern = self.parse_pattern()?;

            // Expect arrow (->)
            self.expect(&TokenKind::Arrow)?;

            // Parse body expression (can be a block or expression)
            let body = if self.check(&TokenKind::LeftBrace) {
                // Parse as block expression
                let block = self.parse_block()?;
                Expression::Block(Box::new(block))
            } else {
                // Parse as regular expression
                self.parse_expression()?
            };
            let arm_end = body.span();

            arms.push(MatchArm {
                pattern,
                body: Box::new(body),
                span: arm_start.merge(arm_end),
            });

            // Optional comma (allow trailing comma and newline separation)
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
            // Allow arms to be separated by newlines without commas
            // Continue if we can parse another pattern, otherwise expect '}'
        }

        let end = self.expect(&TokenKind::RightBrace)?;

        if arms.is_empty() {
            return Err(Error::Parser(
                "Match expression must have at least one arm".to_string(),
                start.merge(end),
            ));
        }

        Ok(Match {
            scrutinee,
            arms,
            span: start.merge(end),
        })
    }

    /// Parses a pattern for match expressions.
    ///
    /// Phase 1: Only literal patterns and wildcard
    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern> {
        let token = self.peek();
        let span = token.1;

        match &token.0.kind {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard { span })
            }
            TokenKind::Integer => {
                let value = token
                    .0
                    .text
                    .parse()
                    .map_err(|_| Error::Parser("Invalid integer literal".to_string(), span))?;
                self.advance();
                Ok(Pattern::Integer { value, span })
            }
            TokenKind::Float => {
                let value = token
                    .0
                    .text
                    .parse()
                    .map_err(|_| Error::Parser("Invalid float literal".to_string(), span))?;
                self.advance();
                Ok(Pattern::Float { value, span })
            }
            TokenKind::String => {
                // Remove surrounding quotes from string literal
                let value = token.0.text[1..token.0.text.len() - 1].to_string();
                self.advance();
                Ok(Pattern::String { value, span })
            }
            TokenKind::True | TokenKind::False => {
                let value = matches!(token.0.kind, TokenKind::True);
                self.advance();
                Ok(Pattern::Boolean { value, span })
            }
            TokenKind::In => {
                // Parse range pattern: in start..end or in start..=end
                self.advance(); // consume 'in'

                // Parse start expression (only primary expressions to avoid conflicts)
                let start = Box::new(self.parse_primary()?);

                // Parse range operator
                let inclusive = if self.check(&TokenKind::DotDotEq) {
                    self.advance();
                    true
                } else if self.check(&TokenKind::DotDot) {
                    self.advance();
                    false
                } else {
                    return Err(Error::Parser(
                        "Expected '..' or '..=' after range start".to_string(),
                        self.peek().1,
                    ));
                };

                // Parse end expression (only primary expressions to avoid conflicts)
                let end = Box::new(self.parse_primary()?);

                let end_span = end.span();
                Ok(Pattern::Range {
                    start,
                    end,
                    inclusive,
                    span: span.merge(end_span),
                })
            }
            _ => Err(Error::Parser(
                "Expected pattern (literal, '_', or 'in range')".to_string(),
                span,
            )),
        }
    }

    /// Parses a range expression.
    ///
    /// Syntax:
    /// ```text
    /// start..end       // Exclusive
    /// start..=end      // Inclusive
    /// ```
    pub(crate) fn parse_range(&mut self, start: Expression) -> Result<Range> {
        let start_span = start.span();

        // Determine if inclusive or exclusive
        let inclusive = if self.check(&TokenKind::DotDotEq) {
            self.advance();
            true
        } else if self.check(&TokenKind::DotDot) {
            self.advance();
            false
        } else {
            return Err(Error::Parser(
                "Expected '..' or '..='".to_string(),
                self.peek().1,
            ));
        };

        // Parse end expression
        let end = Box::new(self.parse_primary()?);
        let end_span = end.span();

        Ok(Range {
            start: Box::new(start),
            end,
            inclusive,
            span: start_span.merge(end_span),
        })
    }

    /// Parses a break statement.
    ///
    /// Syntax:
    /// ```text
    /// break
    /// break 2
    /// break with value
    /// break 2 with value
    /// ```
    pub(crate) fn parse_break(&mut self) -> Result<Break> {
        let start = self.expect(&TokenKind::Break)?;

        // Try to parse depth (optional integer literal)
        let depth = if self.check(&TokenKind::Integer) {
            let token = self.peek();
            let depth_val: u32 = token
                .0
                .text
                .parse()
                .map_err(|_| Error::Parser("Invalid break depth".to_string(), token.1))?;
            self.advance();
            Some(depth_val)
        } else {
            None
        };

        // Try to parse 'with value'
        let value = if self.check(&TokenKind::With) {
            self.advance(); // consume 'with'
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(Break {
            depth,
            value,
            span: start,
        })
    }

    /// Parses a continue statement.
    ///
    /// Syntax:
    /// ```text
    /// continue
    /// continue 2
    /// ```
    pub(crate) fn parse_continue(&mut self) -> Result<Continue> {
        let start = self.expect(&TokenKind::Continue)?;

        // Try to parse depth (optional integer literal)
        let depth = if self.check(&TokenKind::Integer) {
            let token = self.peek();
            let depth_val: u32 = token
                .0
                .text
                .parse()
                .map_err(|_| Error::Parser("Invalid continue depth".to_string(), token.1))?;
            self.advance();
            Some(depth_val)
        } else {
            None
        };

        Ok(Continue { depth, span: start })
    }
}
