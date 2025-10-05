//! Expression parsing with operator precedence.

use super::parser::Parser;
use crate::ast::{BinaryOperator, Expression, UnaryOperator};
use rive_core::{Error, Result};
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses an expression.
    pub(crate) fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_elvis()
    }

    /// Parses Elvis operator (null-coalescing): `value ?: fallback`
    ///
    /// This has the lowest precedence of all operators.
    fn parse_elvis(&mut self) -> Result<Expression> {
        let mut expr = self.parse_or()?;

        while self.peek().0.kind == TokenKind::Question {
            // Check if next token after `?` is `:`
            if self.check_ahead(1, &TokenKind::Colon) {
                // Consume `?` and `:`
                self.advance();
                self.advance();

                // Parse the fallback expression
                let fallback = self.parse_or()?;
                let span = expr.span().merge(fallback.span());

                expr = Expression::Elvis {
                    value: Box::new(expr),
                    fallback: Box::new(fallback),
                    span,
                };
            } else {
                // Not an Elvis operator (might be nullable type `?`), stop parsing
                break;
            }
        }

        Ok(expr)
    }

    /// Parses logical OR expression.
    fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;

        while self.match_token(&TokenKind::PipePipe) {
            let right = self.parse_and()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses logical AND expression.
    fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::AmpersandAmpersand) {
            let right = self.parse_equality()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses equality expression (==, !=).
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;

        while let Some(op_kind) = self.match_tokens(&[TokenKind::EqualEqual, TokenKind::BangEqual])
        {
            let operator = match op_kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses comparison expression (<, <=, >, >=).
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;

        while let Some(op_kind) = self.match_tokens(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let operator = match op_kind {
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::LessEqual => BinaryOperator::LessEqual,
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses addition/subtraction expression.
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_range_expr()?;

        while let Some(op_kind) = self.match_tokens(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = match op_kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_range_expr()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses range expression (.. or ..=).
    fn parse_range_expr(&mut self) -> Result<Expression> {
        let expr = self.parse_factor()?;

        // Check for range operators
        if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotEq) {
            let range = self.parse_range(expr)?;
            Ok(Expression::Range(Box::new(range)))
        } else {
            Ok(expr)
        }
    }

    /// Parses multiplication/division/modulo expression.
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        while let Some(op_kind) =
            self.match_tokens(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent])
        {
            let operator = match op_kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses unary expression (-, !).
    fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(op_kind) = self.match_tokens(&[TokenKind::Minus, TokenKind::Bang]) {
            let start_span = self.previous_span();
            let operator = match op_kind {
                TokenKind::Minus => UnaryOperator::Negate,
                TokenKind::Bang => UnaryOperator::Not,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            let span = start_span.merge(operand.span());
            return Ok(Expression::Unary {
                operator,
                operand: Box::new(operand),
                span,
            });
        }

        self.parse_call()
    }

    /// Parses function call, safe call, or primary expression.
    ///
    /// Handles both regular calls `func()` and safe calls `obj?.func()`.
    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LeftParen) {
                // Regular function call
                self.advance();
                let arguments = self.parse_argument_list()?;
                let end_span = self.expect(&TokenKind::RightParen)?;

                if let Expression::Variable { name, .. } = &expr {
                    let span = expr.span().merge(end_span);
                    expr = Expression::Call {
                        callee: name.clone(),
                        arguments,
                        span,
                    };
                } else {
                    let span = expr.span();
                    return Err(Error::Parser(
                        "Only identifiers can be called".to_string(),
                        span,
                    ));
                }
            } else if self.peek().0.kind == TokenKind::Question
                && self.check_ahead(1, &TokenKind::Dot)
            {
                // Safe call operator `?.`
                self.advance(); // consume `?`
                self.advance(); // consume `.`

                // Parse the call expression (method name + args)
                let call_expr = self.parse_call()?;
                let span = expr.span().merge(call_expr.span());

                expr = Expression::SafeCall {
                    object: Box::new(expr),
                    call: Box::new(call_expr),
                    span,
                };
            } else {
                // No more calls/safe calls
                break;
            }
        }

        Ok(expr)
    }

    /// Parses an argument list for function calls.
    fn parse_argument_list(&mut self) -> Result<Vec<Expression>> {
        let mut arguments = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(arguments)
    }
}
