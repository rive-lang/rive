//! Statement parsing.

use super::parser::Parser;
use crate::ast::Statement;
use rive_core::Result;
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses a statement.
    pub(crate) fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek().0.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Break => {
                let break_stmt = self.parse_break()?;
                Ok(Statement::Break(break_stmt))
            }
            TokenKind::Continue => {
                let continue_stmt = self.parse_continue()?;
                Ok(Statement::Continue(continue_stmt))
            }
            _ => self.parse_expression_or_assignment(),
        }
    }

    /// Parses a let statement.
    /// Supports both `let x? = expr` (infer nullable) and `let x: Type? = expr` (explicit nullable)
    fn parse_let_statement(&mut self) -> Result<Statement> {
        let start_span = self.expect(&TokenKind::Let)?;

        let mutable = self.match_token(&TokenKind::Mut);
        let name = self.expect_identifier()?;

        // Check for `?` after variable name (e.g., `let result? = ...`)
        let infer_nullable = self.match_token(&TokenKind::Question);

        let var_type = if self.check(&TokenKind::Colon) {
            self.advance();

            // If we have both `name?` and `: Type`, that's an error
            if infer_nullable {
                return Err(rive_core::Error::Parser(
                    format!(
                        "Cannot use '{}?' with explicit type annotation. Use ': Type?' for explicit nullable type",
                        name
                    ),
                    start_span,
                ));
            }

            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Equal)?;
        let initializer = self.parse_expression()?;
        let end_span = initializer.span();

        Ok(Statement::Let {
            name,
            mutable,
            var_type,
            infer_nullable,
            initializer,
            span: start_span.merge(end_span),
        })
    }

    /// Parses a return statement.
    fn parse_return_statement(&mut self) -> Result<Statement> {
        let start_span = self.expect(&TokenKind::Return)?;

        let value = if self.check(&TokenKind::RightBrace) || self.is_at_end() {
            None
        } else {
            Some(self.parse_expression()?)
        };

        let end_span = value.as_ref().map_or(start_span, |v| v.span());

        Ok(Statement::Return {
            value,
            span: start_span.merge(end_span),
        })
    }

    /// Parses an expression statement or assignment.
    fn parse_expression_or_assignment(&mut self) -> Result<Statement> {
        // Check if this is an assignment (identifier followed by =)
        if self.check(&TokenKind::Identifier) && self.check_ahead(1, &TokenKind::Equal) {
            let start_span = self.current_span();
            let name = self.peek().0.text.clone();
            self.advance(); // consume identifier
            self.advance(); // consume =

            let value = self.parse_expression()?;
            let span = start_span.merge(value.span());

            return Ok(Statement::Assignment { name, value, span });
        }

        // Otherwise, it's an expression statement
        let expression = self.parse_expression()?;
        let span = expression.span();

        Ok(Statement::Expression { expression, span })
    }
}
