//! Control flow parsing (if, while, for, loop, match, break, continue, range).

use super::parser::Parser;
use crate::{
    ast::Expression,
    control_flow::{
        Break, Continue, ElseIf, For, If, Loop, Match, MatchArm, Pattern, Range, While,
    },
};
use rive_core::{Error, Result};
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses an if expression/statement.
    pub(crate) fn parse_if(&mut self) -> Result<If> {
        let start = self.expect(&TokenKind::If)?;

        let condition = Box::new(self.parse_condition()?);
        let then_block = self.parse_block()?;

        // Parse else-if chain
        let mut else_if_branches = Vec::new();
        while self.check(&TokenKind::Else) && self.check_ahead(1, &TokenKind::If) {
            self.advance(); // consume 'else'
            self.advance(); // consume 'if'

            let condition = Box::new(self.parse_condition()?);
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
    pub(crate) fn parse_while(&mut self) -> Result<While> {
        let start = self.expect(&TokenKind::While)?;
        let condition = Box::new(self.parse_condition()?);
        let body = self.parse_block()?;
        let end = body.span;

        Ok(While {
            condition,
            body,
            span: start.merge(end),
        })
    }

    /// Parses a for loop.
    pub(crate) fn parse_for(&mut self) -> Result<For> {
        let start = self.expect(&TokenKind::For)?;

        let variable = self.expect_identifier()?;
        self.expect(&TokenKind::In)?;
        let iterable = Box::new(self.parse_expression()?);

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
    pub(crate) fn parse_loop(&mut self) -> Result<Loop> {
        let start = self.expect(&TokenKind::Loop)?;
        let body = self.parse_block()?;
        let end = body.span;

        Ok(Loop {
            body,
            span: start.merge(end),
        })
    }

    /// Parses a match expression.
    pub(crate) fn parse_match(&mut self) -> Result<Match> {
        let start = self.expect(&TokenKind::Match)?;
        let scrutinee = Box::new(self.parse_expression()?);
        self.expect(&TokenKind::LeftBrace)?;

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let arm = self.parse_match_arm()?;
            arms.push(arm);

            // Optional comma
            self.match_token(&TokenKind::Comma);
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

    /// Parses a single match arm.
    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let arm_start = self.peek().1;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::Arrow)?;

        let body = if self.check(&TokenKind::LeftBrace) {
            Expression::Block(Box::new(self.parse_block()?))
        } else {
            self.parse_expression()?
        };
        let arm_end = body.span();

        Ok(MatchArm {
            pattern,
            body: Box::new(body),
            span: arm_start.merge(arm_end),
        })
    }

    /// Parses a pattern for match expressions.
    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern> {
        let token = self.peek();
        let span = token.1;

        match &token.0.kind {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard { span })
            }
            TokenKind::Integer => {
                let value = self.parse_i64_literal()?;
                Ok(Pattern::Integer { value, span })
            }
            TokenKind::Float => {
                let value = self.parse_f64_literal()?;
                Ok(Pattern::Float { value, span })
            }
            TokenKind::String => {
                let value = self.parse_string_content()?;
                Ok(Pattern::String { value, span })
            }
            TokenKind::True | TokenKind::False => {
                let value = matches!(token.0.kind, TokenKind::True);
                self.advance();
                Ok(Pattern::Boolean { value, span })
            }
            TokenKind::In => self.parse_range_pattern(span),
            _ => Err(Error::Parser(
                "Expected pattern (literal, '_', or 'in range')".to_string(),
                span,
            )),
        }
    }

    /// Parses a range pattern: `in start..end` or `in start..=end`.
    fn parse_range_pattern(&mut self, span: rive_core::Span) -> Result<Pattern> {
        self.advance(); // consume 'in'

        let start = Box::new(self.parse_primary()?);
        let inclusive = self.parse_range_operator()?;
        let end = Box::new(self.parse_primary()?);
        let end_span = end.span();

        Ok(Pattern::Range {
            start,
            end,
            inclusive,
            span: span.merge(end_span),
        })
    }

    /// Parses a range expression: `start..end` or `start..=end`.
    pub(crate) fn parse_range(&mut self, start: Expression) -> Result<Range> {
        let start_span = start.span();
        let inclusive = self.parse_range_operator()?;
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
    pub(crate) fn parse_break(&mut self) -> Result<Break> {
        let start = self.expect(&TokenKind::Break)?;
        let depth = self.parse_depth()?;

        let value = if self.check(&TokenKind::With) {
            self.advance();
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
    pub(crate) fn parse_continue(&mut self) -> Result<Continue> {
        let start = self.expect(&TokenKind::Continue)?;
        let depth = self.parse_depth()?;

        Ok(Continue { depth, span: start })
    }
}
