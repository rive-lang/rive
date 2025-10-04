//! Expression type checking.

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::ast::Expression;

impl TypeChecker {
    /// Checks an expression and returns its type.
    pub(crate) fn check_expression(&mut self, expr: &Expression) -> Result<TypeId> {
        match expr {
            Expression::Integer { .. } => Ok(TypeId::INT),
            Expression::Float { .. } => Ok(TypeId::FLOAT),
            Expression::String { .. } => Ok(TypeId::TEXT),
            Expression::Boolean { .. } => Ok(TypeId::BOOL),
            Expression::Null { span } => Err(Error::SemanticWithSpan(
                "Null type not yet supported".to_string(),
                *span,
            )),

            Expression::Variable { name, span } => {
                let symbol = self.symbols.lookup(name).ok_or_else(|| {
                    Error::SemanticWithSpan(format!("Undefined variable '{name}'"), *span)
                })?;
                Ok(symbol.symbol_type)
            }

            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => self.check_binary_op(left, operator, right, *span),

            Expression::Unary {
                operator,
                operand,
                span,
            } => self.check_unary_op(operator, operand, *span),

            Expression::Call {
                callee,
                arguments,
                span,
            } => self.check_call(callee, arguments, *span),

            Expression::Array { elements, span } => self.check_array(elements, *span),

            // Control flow expressions
            Expression::If(if_expr) => self.check_if(if_expr, true),
            Expression::While(while_loop) => self.check_while(while_loop),
            Expression::For(for_loop) => self.check_for(for_loop),
            Expression::Loop(loop_expr) => self.check_loop(loop_expr),
            Expression::Match(match_expr) => self.check_match(match_expr, true),
            Expression::Range(range) => self.check_range(range),
            Expression::Block(block) => self.check_block_expression(block),
        }
    }

    /// Checks a binary operation.
    fn check_binary_op(
        &mut self,
        left: &Expression,
        operator: &rive_parser::BinaryOperator,
        right: &Expression,
        span: rive_core::Span,
    ) -> Result<TypeId> {
        use rive_parser::BinaryOperator;

        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        // Type compatibility check
        if !self.types_compatible(left_type, right_type) {
            return Err(self.type_mismatch_error(
                "Binary operation type mismatch",
                left_type,
                right_type,
                span,
            ));
        }

        // Determine result type based on operator
        let result_type = match operator {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => left_type,

            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::And
            | BinaryOperator::Or => TypeId::BOOL,
        };

        Ok(result_type)
    }

    /// Checks a unary operation.
    fn check_unary_op(
        &mut self,
        operator: &rive_parser::UnaryOperator,
        operand: &Expression,
        span: rive_core::Span,
    ) -> Result<TypeId> {
        use rive_parser::UnaryOperator;

        let operand_type = self.check_expression(operand)?;

        match operator {
            UnaryOperator::Negate => {
                if !self.types_compatible(operand_type, TypeId::INT)
                    && !self.types_compatible(operand_type, TypeId::FLOAT)
                {
                    let registry = self.symbols.type_registry();
                    let type_str = registry.get_type_name(operand_type);
                    return Err(Error::SemanticWithSpan(
                        format!("Cannot negate type '{type_str}'"),
                        span,
                    ));
                }
                Ok(operand_type)
            }
            UnaryOperator::Not => {
                if !self.types_compatible(operand_type, TypeId::BOOL) {
                    let registry = self.symbols.type_registry();
                    let type_str = registry.get_type_name(operand_type);
                    return Err(Error::SemanticWithSpan(
                        format!("Cannot apply logical NOT to type '{type_str}'"),
                        span,
                    ));
                }
                Ok(TypeId::BOOL)
            }
        }
    }

    /// Checks a function call.
    fn check_call(
        &mut self,
        callee: &str,
        arguments: &[Expression],
        span: rive_core::Span,
    ) -> Result<TypeId> {
        // Special handling for built-in print function
        if callee == "print" {
            if arguments.is_empty() {
                return Err(Error::SemanticWithSpan(
                    "print requires at least one argument".to_string(),
                    span,
                ));
            }
            // Check all arguments
            for arg in arguments {
                self.check_expression(arg)?;
            }
            return Ok(TypeId::UNIT);
        }

        // Look up function symbol
        let func_type_id = self
            .symbols
            .lookup(callee)
            .ok_or_else(|| Error::SemanticWithSpan(format!("Undefined function '{callee}'"), span))?
            .symbol_type;

        // Extract function type
        let (param_types, return_type) = {
            let registry = self.symbols.type_registry();
            let metadata = registry.get_type_metadata(func_type_id);

            use rive_core::type_system::TypeKind;
            if let TypeKind::Function {
                parameters,
                return_type,
            } = &metadata.kind
            {
                (parameters.clone(), *return_type)
            } else {
                return Err(Error::SemanticWithSpan(
                    format!("'{callee}' is not a function"),
                    span,
                ));
            }
        };

        // Check argument count
        if arguments.len() != param_types.len() {
            return Err(Error::SemanticWithSpan(
                format!(
                    "Function '{callee}' expects {} arguments, but {} were provided",
                    param_types.len(),
                    arguments.len()
                ),
                span,
            ));
        }

        // Clone arguments to avoid borrow issues during checking
        let args_clone = arguments.to_vec();

        // Check argument types
        for (i, (expected_type, arg)) in param_types.iter().zip(args_clone.iter()).enumerate() {
            let arg_type = self.check_expression(arg)?;
            if !self.types_compatible(arg_type, *expected_type) {
                return Err(self.type_mismatch_error(
                    &format!("Argument {} type mismatch", i + 1),
                    *expected_type,
                    arg_type,
                    span,
                ));
            }
        }

        Ok(return_type)
    }

    /// Checks an array literal.
    fn check_array(&mut self, elements: &[Expression], span: rive_core::Span) -> Result<TypeId> {
        if elements.is_empty() {
            return Err(Error::SemanticWithSpan(
                "Empty arrays are not supported".to_string(),
                span,
            ));
        }

        // Check all elements have the same type
        let first_type = self.check_expression(&elements[0])?;
        for (i, elem) in elements.iter().enumerate().skip(1) {
            let elem_type = self.check_expression(elem)?;
            if !self.types_compatible(elem_type, first_type) {
                return Err(self.type_mismatch_error(
                    &format!("Array element at index {i} type mismatch"),
                    first_type,
                    elem_type,
                    span,
                ));
            }
        }

        // Create array type
        let array_type = self
            .symbols
            .type_registry_mut()
            .create_array(first_type, elements.len());
        Ok(array_type)
    }

    /// Checks a block expression and returns its type.
    fn check_block_expression(&mut self, block: &rive_parser::Block) -> Result<TypeId> {
        // Check all statements in the block
        for statement in &block.statements {
            self.check_statement(statement)?;
        }

        // Check if there's a final expression
        if let Some(rive_parser::Statement::Expression { expression, .. }) = block.statements.last()
        {
            return self.check_expression(expression);
        }

        // No final expression, block has Unit type
        Ok(TypeId::UNIT)
    }
}
