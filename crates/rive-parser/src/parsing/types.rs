//! Type annotation and type declaration parsing.

use super::parser::Parser;
use crate::ast::{
    Field, ImplBlock, InlineImpl, InterfaceDecl, MethodDecl, MethodSig, TypeDecl,
};
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parses a type annotation and returns a TypeId.
    ///
    /// Supports nullable types with `?` suffix: `Int?`, `Text?`, etc.
    pub(crate) fn parse_type(&mut self) -> Result<TypeId> {
        let token = self.peek();

        let base_type = match &token.0.kind {
            TokenKind::Identifier => self.parse_named_type(),
            TokenKind::LeftBracket => self.parse_array_type(),
            _ => {
                let span = self.current_span();
                return Err(Error::Parser(
                    format!("Expected type, found '{}'", token.0.text),
                    span,
                ));
            }
        }?;

        // Check for nullable type suffix `?`
        if self.peek().0.kind == TokenKind::Question {
            self.advance(); // consume `?`
            Ok(self.type_registry_mut().create_optional(base_type))
        } else {
            Ok(base_type)
        }
    }

    /// Parses a named type (Int, Float, Text, Bool, or user-defined).
    ///
    /// Note: Nullable types are handled by `parse_type()` with the `?` suffix.
    fn parse_named_type(&mut self) -> Result<TypeId> {
        let type_name = self.peek().0.text.clone();
        let span = self.current_span();
        self.advance();

        match type_name.as_str() {
            "Int" => Ok(TypeId::INT),
            "Float" => Ok(TypeId::FLOAT),
            "Text" => Ok(TypeId::TEXT),
            "Bool" => Ok(TypeId::BOOL),
            _ => {
                // Try to look up user-defined type
                if let Some(type_id) = self.type_registry().get_by_name(&type_name) {
                    Ok(type_id)
                } else {
                    Err(Error::Parser(format!("Unknown type '{type_name}'"), span))
                }
            }
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

    /// Parses a type declaration: `type Name(x: Type, ...) { ... }`
    pub(super) fn parse_type_decl(&mut self) -> Result<TypeDecl> {
        let start_span = self.expect(&TokenKind::Type)?;

        // Parse type name
        self.expect(&TokenKind::Identifier)?;
        let name = self.previous_token().0.text.clone();

        // Pre-register the type with a placeholder so methods can reference it
        let type_id = self.type_registry_mut().generate_id();
        let placeholder_kind = rive_core::type_system::TypeKind::Struct {
            name: name.clone(),
            fields: Vec::new(),
        };
        let placeholder_metadata = rive_core::type_system::TypeMetadata::user_defined(
            type_id,
            placeholder_kind.clone(),
            rive_core::type_system::MemoryStrategy::Copy, // Will update later
            false,
        );
        self.type_registry_mut().register(placeholder_metadata);

        // Parse constructor parameters
        self.expect(&TokenKind::LeftParen)?;
        let ctor_params = self.parse_field_list()?;
        self.expect(&TokenKind::RightParen)?;

        // Optional body with fields, methods, and inline impls
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut inline_impls = Vec::new();

        let end_span = if self.check(&TokenKind::LeftBrace) {
            self.advance(); // consume '{'

            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                if self.check(&TokenKind::Let) || self.check(&TokenKind::Const) {
                    // Field declaration
                    fields.push(self.parse_field_decl()?);
                } else if self.check(&TokenKind::Fun) || self.check(&TokenKind::Static) {
                    // Method declaration
                    methods.push(self.parse_method_decl()?);
                } else if self.check(&TokenKind::Impl) {
                    // Inline impl
                    inline_impls.push(self.parse_inline_impl()?);
                } else {
                    let span = self.current_span();
                    return Err(Error::Parser(
                        format!("Expected field, method, or impl, found '{}'", self.peek().0.text),
                        span,
                    ));
                }
            }

            self.expect(&TokenKind::RightBrace)?
        } else {
            self.previous_span()
        };

        // Update the type registration with complete field information
        let mut all_fields: Vec<(String, TypeId)> = ctor_params
            .iter()
            .map(|f| (f.name.clone(), f.field_type))
            .collect();
        all_fields.extend(fields.iter().map(|f| (f.name.clone(), f.field_type)));

        let type_kind = rive_core::type_system::TypeKind::Struct {
            name: name.clone(),
            fields: all_fields,
        };

        // Determine memory strategy based on field sizes
        let memory_strategy = self.determine_memory_strategy(&ctor_params, &fields);
        let updated_metadata = rive_core::type_system::TypeMetadata::user_defined(
            type_id,
            type_kind,
            memory_strategy,
            false, // explicit_unique - not supported yet
        );
        // Replace the placeholder with the complete metadata
        self.type_registry_mut().register(updated_metadata);

        Ok(TypeDecl {
            name,
            ctor_params,
            fields,
            methods,
            inline_impls,
            span: start_span.merge(end_span),
        })
    }

    /// Parses a field list: `name: Type, ...`
    fn parse_field_list(&mut self) -> Result<Vec<Field>> {
        let mut fields = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let mutable = self.match_token(&TokenKind::Mut);
                self.expect(&TokenKind::Identifier)?;
                let name = self.previous_token().0.text.clone();
                let span = self.previous_span();

                self.expect(&TokenKind::Colon)?;
                let field_type = self.parse_type()?;

                fields.push(Field {
                    name,
                    field_type,
                    mutable,
                    span,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(fields)
    }

    /// Parses a field declaration in type body: `let [mut] name: Type`
    fn parse_field_decl(&mut self) -> Result<Field> {
        let is_const = self.check(&TokenKind::Const);
        self.advance(); // consume 'let' or 'const'

        let mutable = !is_const && self.match_token(&TokenKind::Mut);

        self.expect(&TokenKind::Identifier)?;
        let name = self.previous_token().0.text.clone();
        let span = self.previous_span();

        self.expect(&TokenKind::Colon)?;
        let field_type = self.parse_type()?;

        // Optional semicolon
        self.match_token(&TokenKind::Semicolon);

        Ok(Field {
            name,
            field_type,
            mutable,
            span,
        })
    }

    /// Parses a method declaration: `[static] fun name(params) [-> RetType] { body }`
    fn parse_method_decl(&mut self) -> Result<MethodDecl> {
        let is_static = self.match_token(&TokenKind::Static);
        let start_span = self.expect(&TokenKind::Fun)?;

        self.expect(&TokenKind::Identifier)?;
        let name = self.previous_token().0.text.clone();

        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_parameter_list()?;
        self.expect(&TokenKind::RightParen)?;

        // Parse return type
        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            TypeId::UNIT
        };

        // Parse body
        let body = if self.check(&TokenKind::LeftBrace) {
            crate::ast::FunctionBody::Block(self.parse_block()?)
        } else if self.match_token(&TokenKind::Equal) {
            let expr = self.parse_expression()?;
            self.match_token(&TokenKind::Semicolon);
            crate::ast::FunctionBody::Expression(expr)
        } else {
            let span = self.current_span();
            return Err(Error::Parser(
                "Expected method body (block or expression)".to_string(),
                span,
            ));
        };

        let end_span = self.previous_span();

        Ok(MethodDecl {
            name,
            is_static,
            params,
            return_type,
            body,
            span: start_span.merge(end_span),
        })
    }

    /// Parses an interface declaration: `interface Name { method_sigs... }`
    pub(super) fn parse_interface_decl(&mut self) -> Result<InterfaceDecl> {
        let start_span = self.expect(&TokenKind::Interface)?;

        self.expect(&TokenKind::Identifier)?;
        let name = self.previous_token().0.text.clone();

        self.expect(&TokenKind::LeftBrace)?;
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_method_sig()?);
        }

        let end_span = self.expect(&TokenKind::RightBrace)?;

        Ok(InterfaceDecl {
            name,
            methods,
            span: start_span.merge(end_span),
        })
    }

    /// Parses a method signature: `fun name(params) [: RetType]`
    fn parse_method_sig(&mut self) -> Result<MethodSig> {
        let start_span = self.expect(&TokenKind::Fun)?;

        self.expect(&TokenKind::Identifier)?;
        let name = self.previous_token().0.text.clone();

        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_parameter_list()?;
        self.expect(&TokenKind::RightParen)?;

        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            TypeId::UNIT
        };

        // Optional semicolon
        self.match_token(&TokenKind::Semicolon);

        let end_span = self.previous_span();

        Ok(MethodSig {
            name,
            params,
            return_type,
            span: start_span.merge(end_span),
        })
    }

    /// Parses an impl block: `impl Interface for Type { methods... }`
    pub(super) fn parse_impl_block(&mut self) -> Result<ImplBlock> {
        let start_span = self.expect(&TokenKind::Impl)?;

        // Parse interface name
        self.expect(&TokenKind::Identifier)?;
        let interface = self.previous_token().0.text.clone();

        // Expect 'for'
        self.expect(&TokenKind::For)?;

        // Parse target type
        self.expect(&TokenKind::Identifier)?;
        let target_type = self.previous_token().0.text.clone();

        self.expect(&TokenKind::LeftBrace)?;
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_method_decl()?);
        }

        let end_span = self.expect(&TokenKind::RightBrace)?;

        Ok(ImplBlock {
            target_type,
            interface: Some(interface),
            methods,
            span: start_span.merge(end_span),
        })
    }

    /// Parses an extend block: `extend Type { methods... }`
    pub(super) fn parse_extend_block(&mut self) -> Result<ImplBlock> {
        let start_span = self.expect(&TokenKind::Extend)?;

        self.expect(&TokenKind::Identifier)?;
        let target_type = self.previous_token().0.text.clone();

        self.expect(&TokenKind::LeftBrace)?;
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_method_decl()?);
        }

        let end_span = self.expect(&TokenKind::RightBrace)?;

        Ok(ImplBlock {
            target_type,
            interface: None,
            methods,
            span: start_span.merge(end_span),
        })
    }

    /// Parses an inline impl: `impl Interface { methods... }`
    fn parse_inline_impl(&mut self) -> Result<InlineImpl> {
        let start_span = self.expect(&TokenKind::Impl)?;

        self.expect(&TokenKind::Identifier)?;
        let interface = self.previous_token().0.text.clone();

        self.expect(&TokenKind::LeftBrace)?;
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_method_decl()?);
        }

        let end_span = self.expect(&TokenKind::RightBrace)?;

        Ok(InlineImpl {
            interface,
            methods,
            span: start_span.merge(end_span),
        })
    }

    /// Determines memory strategy based on field types and total size
    fn determine_memory_strategy(
        &self,
        ctor_params: &[Field],
        fields: &[Field],
    ) -> rive_core::type_system::MemoryStrategy {
        use rive_core::type_system::MemoryStrategy;

        // Collect all field types
        let all_fields: Vec<TypeId> = ctor_params
            .iter()
            .chain(fields.iter())
            .map(|f| f.field_type)
            .collect();

        // If any field is non-Copy, use CoW
        for &field_type in &all_fields {
            if let Some(meta) = self.type_registry().get(field_type) {
                if !meta.is_copy() {
                    return MemoryStrategy::CoW;
                }
            }
        }

        // Estimate size: all Copy primitives are 8 bytes each
        let estimated_size = all_fields.len() * 8;
        const SIZE_THRESHOLD: usize = 16 * 1024; // 16 KB

        if estimated_size <= SIZE_THRESHOLD {
            MemoryStrategy::Copy
        } else {
            MemoryStrategy::CoW
        }
    }
}
