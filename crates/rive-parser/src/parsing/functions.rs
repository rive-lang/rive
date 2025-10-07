//! Function and parameter parsing.

use super::parser::Parser;
use crate::ast::{Function, FunctionBody, Parameter};
use rive_core::Result;
use rive_core::type_system::TypeId;
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses a function declaration.
    /// Supports both block syntax: `fun name() { ... }`
    /// and expression syntax: `fun name() = expr`
    pub(crate) fn parse_function(&mut self) -> Result<Function> {
        let start_span = self.expect(&TokenKind::Fun)?;

        let name = self.expect_identifier()?;

        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_parameter_list()?;
        self.expect(&TokenKind::RightParen)?;

        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            TypeId::UNIT
        };

        // Check if it's an expression body (= expr) or block body ({ ... })
        let (body, end_span) = if self.match_token(&TokenKind::Equal) {
            // Expression body: fun add(a: Int, b: Int): Int = a + b
            let expr = self.parse_expression()?;
            let expr_span = expr.span();
            (FunctionBody::Expression(expr), expr_span)
        } else {
            // Block body: fun add(a: Int, b: Int): Int { return a + b }
            let block = self.parse_block()?;
            let block_span = block.span;
            (FunctionBody::Block(block), block_span)
        };

        Ok(Function {
            name,
            params,
            return_type,
            body,
            span: start_span.merge(end_span),
        })
    }

    /// Parses a parameter list.
    pub(super) fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let name_span = self.current_span();
                let name = self.expect_identifier()?;

                self.expect(&TokenKind::Colon)?;
                let param_type = self.parse_type()?;

                params.push(Parameter {
                    name,
                    param_type,
                    span: name_span,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }
}
