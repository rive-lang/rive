//! Recursive descent parser implementation.

use crate::ast::{
    BinaryOperator, Block, Expression, Function, Item, Parameter, Program, Statement, UnaryOperator,
};
use rive_core::type_system::{TypeId, TypeRegistry};
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

    /// Consumes the parser and returns the type registry.
    pub fn into_type_registry(self) -> TypeRegistry {
        self.type_registry
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

    /// Parses a function declaration.
    fn parse_function(&mut self) -> Result<Function> {
        let start_span = self.expect(&TokenKind::Fun)?;

        let name = self.expect_identifier()?;

        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_parameter_list()?;
        self.expect(&TokenKind::RightParen)?;

        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            self.type_registry.get_unit()
        };

        let body = self.parse_block()?;
        let end_span = body.span;

        Ok(Function {
            name,
            params,
            return_type,
            body,
            span: start_span.merge(end_span),
        })
    }

    /// Parses a parameter list.
    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>> {
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

    /// Parses a type annotation and returns a TypeId.
    fn parse_type(&mut self) -> Result<TypeId> {
        let token = self.peek();

        match &token.0.kind {
            TokenKind::TypeInt => {
                self.advance();
                Ok(self.type_registry.get_int())
            }
            TokenKind::TypeFloat => {
                self.advance();
                Ok(self.type_registry.get_float())
            }
            TokenKind::TypeText => {
                self.advance();
                Ok(self.type_registry.get_text())
            }
            TokenKind::TypeBool => {
                self.advance();
                Ok(self.type_registry.get_bool())
            }
            TokenKind::TypeOptional => {
                self.advance();
                self.expect(&TokenKind::Less)?;
                let inner_type = self.parse_type()?;
                self.expect(&TokenKind::Greater)?;
                Ok(self.type_registry.get_or_create_optional(inner_type))
            }
            TokenKind::LeftBracket => {
                self.advance();
                let element_type = self.parse_type()?;
                self.expect(&TokenKind::Semicolon)?;

                self.expect(&TokenKind::Integer)?;
                let size = self.tokens[self.current - 1]
                    .0
                    .text
                    .parse::<usize>()
                    .map_err(|_| {
                        let span = self.previous_span();
                        Error::Parser("Invalid array size".to_string(), span)
                    })?;

                self.expect(&TokenKind::RightBracket)?;
                Ok(self.type_registry.get_or_create_array(element_type, size))
            }
            _ => {
                let span = self.current_span();
                Err(Error::Parser(
                    format!("Expected type, found '{}'", token.0.text),
                    span,
                ))
            }
        }
    }

    /// Parses a block of statements.
    fn parse_block(&mut self) -> Result<Block> {
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

    /// Parses a statement.
    fn parse_statement(&mut self) -> Result<Statement> {
        if self.check(&TokenKind::Let) {
            self.parse_let_statement()
        } else if self.check(&TokenKind::Return) {
            self.parse_return_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    /// Parses a let statement.
    fn parse_let_statement(&mut self) -> Result<Statement> {
        let start_span = self.expect(&TokenKind::Let)?;

        let mutable = self.match_token(&TokenKind::Mut);
        let name = self.expect_identifier()?;

        let var_type = if self.check(&TokenKind::Colon) {
            self.advance();
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
    fn parse_expression_statement(&mut self) -> Result<Statement> {
        // Check if this is an assignment statement (identifier followed by =)
        if self.check(&TokenKind::Identifier) {
            let start_span = self.current_span();
            let name = self.peek().0.text.clone();

            // Look ahead to see if there's an = after the identifier
            if self.current + 1 < self.tokens.len()
                && self.tokens[self.current + 1].0.kind == TokenKind::Equal
            {
                // This is an assignment
                self.advance(); // consume identifier
                self.advance(); // consume =
                let value = self.parse_expression()?;
                let span = start_span.merge(value.span());

                return Ok(Statement::Assignment { name, value, span });
            }
        }

        // Otherwise, it's an expression statement
        let expression = self.parse_expression()?;
        let span = expression.span();

        Ok(Statement::Expression { expression, span })
    }

    /// Parses an expression.
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or()
    }

    /// Parses logical OR expression.
    fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;

        while self.match_token(&TokenKind::PipePipe) {
            let operator = BinaryOperator::Or;
            let right = self.parse_and()?;
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

    /// Parses logical AND expression.
    fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::AmpersandAmpersand) {
            let operator = BinaryOperator::And;
            let right = self.parse_equality()?;
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

    /// Parses equality expression.
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;

        while let Some(operator) = self.match_tokens(&[TokenKind::EqualEqual, TokenKind::BangEqual])
        {
            let op = match operator {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses comparison expression.
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;

        while let Some(operator) = self.match_tokens(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let op = match operator {
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
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses addition/subtraction expression.
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;

        while let Some(operator) = self.match_tokens(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = match operator {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses multiplication/division/modulo expression.
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        while let Some(operator) =
            self.match_tokens(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent])
        {
            let op = match operator {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let span = expr.span().merge(right.span());
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parses unary expression.
    fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(operator) = self.match_tokens(&[TokenKind::Minus, TokenKind::Bang]) {
            let start_span = self.previous_span();
            let op = match operator {
                TokenKind::Minus => UnaryOperator::Negate,
                TokenKind::Bang => UnaryOperator::Not,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            let span = start_span.merge(operand.span());
            return Ok(Expression::Unary {
                operator: op,
                operand: Box::new(operand),
                span,
            });
        }

        self.parse_call()
    }

    /// Parses function call or primary expression.
    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        while self.check(&TokenKind::LeftParen) {
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

    /// Parses a primary expression (literals, variables, arrays, parenthesized).
    fn parse_primary(&mut self) -> Result<Expression> {
        let span = self.current_span();
        let token = self.peek().clone();

        match &token.0.kind {
            TokenKind::Integer => {
                self.advance();
                let value = token.0.text.parse::<i64>().map_err(|_| {
                    Error::Parser(format!("Invalid integer: {}", token.0.text), span)
                })?;
                Ok(Expression::Integer { value, span })
            }
            TokenKind::Float => {
                self.advance();
                let value =
                    token.0.text.parse::<f64>().map_err(|_| {
                        Error::Parser(format!("Invalid float: {}", token.0.text), span)
                    })?;
                Ok(Expression::Float { value, span })
            }
            TokenKind::String => {
                self.advance();
                let value = token.0.text[1..token.0.text.len() - 1].to_string();
                Ok(Expression::String { value, span })
            }
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
            TokenKind::Identifier | TokenKind::Print => {
                self.advance();
                Ok(Expression::Variable {
                    name: token.0.text.clone(),
                    span,
                })
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                self.advance();
                let elements = self.parse_array_elements()?;
                let end_span = self.expect(&TokenKind::RightBracket)?;
                Ok(Expression::Array {
                    elements,
                    span: span.merge(end_span),
                })
            }
            _ => {
                let span = self.current_span();
                Err(Error::Parser(
                    format!("Unexpected token '{}'", token.0.text),
                    span,
                ))
            }
        }
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

    // Helper methods

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> &(Token, Span) {
        if self.is_at_end() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.current]
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().0.kind == kind
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_tokens(&mut self, kinds: &[TokenKind]) -> Option<TokenKind> {
        for kind in kinds {
            if self.check(kind) {
                let matched = kind.clone();
                self.advance();
                return Some(matched);
            }
        }
        None
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Span> {
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

    fn expect_identifier(&mut self) -> Result<String> {
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

    fn current_span(&self) -> Span {
        if self.is_at_end() {
            self.tokens[self.tokens.len() - 1].1
        } else {
            self.tokens[self.current].1
        }
    }

    fn previous_span(&self) -> Span {
        if self.current > 0 {
            self.tokens[self.current - 1].1
        } else {
            self.current_span()
        }
    }
}
