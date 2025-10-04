//! Type annotation parsing.

use super::parser::Parser;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses a type annotation and returns a TypeId.
    pub(crate) fn parse_type(&mut self) -> Result<TypeId> {
        let token = self.peek();

        match &token.0.kind {
            TokenKind::Identifier => self.parse_named_type(),
            TokenKind::LeftBracket => self.parse_array_type(),
            _ => {
                let span = self.current_span();
                Err(Error::Parser(
                    format!("Expected type, found '{}'", token.0.text),
                    span,
                ))
            }
        }
    }

    /// Parses a named type (Int, Float, Text, Bool, Optional<T>).
    fn parse_named_type(&mut self) -> Result<TypeId> {
        let type_name = self.peek().0.text.clone();
        let span = self.current_span();
        self.advance();

        match type_name.as_str() {
            "Int" => Ok(TypeId::INT),
            "Float" => Ok(TypeId::FLOAT),
            "Text" => Ok(TypeId::TEXT),
            "Bool" => Ok(TypeId::BOOL),
            "Optional" => {
                self.expect(&TokenKind::Less)?;
                let inner_type = self.parse_type()?;
                self.expect(&TokenKind::Greater)?;
                Ok(self.type_registry_mut().create_optional(inner_type))
            }
            _ => Err(Error::Parser(format!("Unknown type '{type_name}'"), span)),
        }
    }

    /// Parses an array type [T; N].
    fn parse_array_type(&mut self) -> Result<TypeId> {
        self.advance(); // consume '['
        let element_type = self.parse_type()?;
        self.expect(&TokenKind::Semicolon)?;

        self.expect(&TokenKind::Integer)?;
        let size = self.previous_token().0.text.parse::<usize>().map_err(|_| {
            let span = self.previous_span();
            Error::Parser("Invalid array size".to_string(), span)
        })?;

        self.expect(&TokenKind::RightBracket)?;
        Ok(self.type_registry_mut().create_array(element_type, size))
    }
}
