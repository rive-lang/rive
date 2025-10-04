//! Core parser structure and helper methods.

use crate::ast::{Block, Item, Program};
use rive_core::type_system::TypeRegistry;
use rive_core::{Error, Result, Span};
use rive_lexer::{Token, TokenKind};

/// Parser for Rive source code.
pub struct Parser<'a> {
    tokens: &'a [(Token, Span)],
    current: usize,
    type_registry: TypeRegistry,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given token stream.
    pub fn new(tokens: &'a [(Token, Span)]) -> Self {
        Self {
            tokens,
            current: 0,
            type_registry: TypeRegistry::new(),
        }
    }

    /// Returns a reference to the type registry.
    pub fn type_registry(&self) -> &TypeRegistry {
        &self.type_registry
    }

    /// Returns a mutable reference to the type registry.
    pub(crate) fn type_registry_mut(&mut self) -> &mut TypeRegistry {
        &mut self.type_registry
    }

    /// Consumes the parser and returns the type registry.
    pub fn into_type_registry(self) -> TypeRegistry {
        self.type_registry
    }

    /// Returns the previous token.
    pub(crate) fn previous_token(&self) -> &(Token, Span) {
        if self.current > 0 {
            &self.tokens[self.current - 1]
        } else {
            self.peek()
        }
    }

    /// Parses a complete program.
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        Ok(Program { items })
    }

    /// Parses a top-level item (currently only functions).
    fn parse_item(&mut self) -> Result<Item> {
        if self.check(&TokenKind::Fun) {
            Ok(Item::Function(self.parse_function()?))
        } else {
            let span = self.current_span();
            Err(Error::Parser(
                format!(
                    "Expected function declaration, found '{}'",
                    self.peek().0.text
                ),
                span,
            ))
        }
    }

    /// Parses a block of statements.
    pub(crate) fn parse_block(&mut self) -> Result<Block> {
        let start_span = self.expect(&TokenKind::LeftBrace)?;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        let end_span = self.expect(&TokenKind::RightBrace)?;

        Ok(Block {
            statements,
            span: start_span.merge(end_span),
        })
    }

    // ==================== Helper Methods ====================

    pub(crate) fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    pub(crate) fn peek(&self) -> &(Token, Span) {
        if self.is_at_end() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.current]
        }
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        !self.is_at_end() && &self.peek().0.kind == kind
    }

    pub(crate) fn check_ahead(&self, offset: usize, kind: &TokenKind) -> bool {
        self.tokens
            .get(self.current + offset)
            .is_some_and(|t| &t.0.kind == kind)
    }

    pub(crate) fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    pub(crate) fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn match_tokens(&mut self, kinds: &[TokenKind]) -> Option<TokenKind> {
        for kind in kinds {
            if self.check(kind) {
                let matched = kind.clone();
                self.advance();
                return Some(matched);
            }
        }
        None
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind) -> Result<Span> {
        if self.check(kind) {
            let span = self.current_span();
            self.advance();
            Ok(span)
        } else {
            let span = self.current_span();
            Err(Error::Parser(
                format!("Expected '{}', found '{}'", kind, self.peek().0.text),
                span,
            ))
        }
    }

    pub(crate) fn expect_identifier(&mut self) -> Result<String> {
        if self.check(&TokenKind::Identifier) {
            let name = self.peek().0.text.clone();
            self.advance();
            Ok(name)
        } else {
            let span = self.current_span();
            Err(Error::Parser(
                format!("Expected identifier, found '{}'", self.peek().0.text),
                span,
            ))
        }
    }

    pub(crate) fn current_span(&self) -> Span {
        if self.is_at_end() {
            self.tokens[self.tokens.len() - 1].1
        } else {
            self.tokens[self.current].1
        }
    }

    pub(crate) fn previous_span(&self) -> Span {
        if self.current > 0 {
            self.tokens[self.current - 1].1
        } else {
            self.current_span()
        }
    }
}
