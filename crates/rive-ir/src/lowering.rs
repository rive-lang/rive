//! AST to RIR lowering - converts parsed AST into RIR for optimization.

use rive_core::{
    Result,
    type_system::{MemoryStrategy, TypeId, TypeRegistry},
};
use rive_parser::ast::{
    BinaryOperator, Block as AstBlock, Expression as AstExpression, Function as AstFunction, Item,
    Parameter as AstParameter, Program, Statement as AstStatement, UnaryOperator,
};

use crate::{
    BinaryOp, RirBlock, RirExpression, RirFunction, RirModule, RirParameter, RirStatement, UnaryOp,
};

/// Converts AST to RIR
pub struct AstLowering {
    type_registry: TypeRegistry,
}

impl AstLowering {
    /// Creates a new AST lowering instance
    #[must_use]
    pub fn new(type_registry: TypeRegistry) -> Self {
        Self { type_registry }
    }

    /// Lowers a complete program to RIR
    pub fn lower_program(&mut self, program: &Program) -> Result<RirModule> {
        let mut module = RirModule::new(self.type_registry.clone());

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let rir_func = self.lower_function(func)?;
                    module.add_function(rir_func);
                }
            }
        }

        Ok(module)
    }

    /// Lowers a function declaration
    fn lower_function(&mut self, func: &AstFunction) -> Result<RirFunction> {
        let parameters = func
            .params
            .iter()
            .map(|p| self.lower_parameter(p))
            .collect::<Result<Vec<_>>>()?;

        let return_type = self.type_to_type_id(&func.return_type);
        let body = self.lower_block(&func.body)?;

        Ok(RirFunction::new(
            func.name.clone(),
            parameters,
            return_type,
            body,
            func.span,
        ))
    }

    /// Lowers a function parameter
    fn lower_parameter(&self, param: &AstParameter) -> Result<RirParameter> {
        let type_id = self.type_to_type_id(&param.param_type);
        let memory_strategy = self.determine_memory_strategy(type_id);
        Ok(RirParameter::new(
            param.name.clone(),
            type_id,
            false, // Parameters are not mutable by default in Rive
            memory_strategy,
            param.span,
        ))
    }

    /// Lowers a block of statements
    fn lower_block(&mut self, block: &AstBlock) -> Result<RirBlock> {
        let mut rir_block = RirBlock::new(block.span);

        for stmt in &block.statements {
            let rir_stmt = self.lower_statement(stmt)?;
            rir_block.add_statement(rir_stmt);
        }

        Ok(rir_block)
    }

    /// Lowers a statement
    fn lower_statement(&mut self, stmt: &AstStatement) -> Result<RirStatement> {
        match stmt {
            AstStatement::Let {
                name,
                mutable,
                var_type,
                initializer,
                span,
            } => {
                let value = self.lower_expression(initializer)?;
                let type_id = if let Some(typ) = var_type {
                    self.type_to_type_id(typ)
                } else {
                    value.type_id()
                };

                // Determine memory strategy based on type
                let memory_strategy = self.determine_memory_strategy(type_id);

                Ok(RirStatement::Let {
                    name: name.clone(),
                    type_id,
                    is_mutable: *mutable,
                    value: Box::new(value),
                    memory_strategy,
                    span: *span,
                })
            }

            AstStatement::Assignment { name, value, span } => {
                let rir_value = self.lower_expression(value)?;
                Ok(RirStatement::Assign {
                    name: name.clone(),
                    value: Box::new(rir_value),
                    span: *span,
                })
            }

            AstStatement::Expression { expression, span } => {
                let rir_expr = self.lower_expression(expression)?;
                Ok(RirStatement::Expression {
                    expr: Box::new(rir_expr),
                    span: *span,
                })
            }

            AstStatement::Return { value, span } => {
                let rir_value = value
                    .as_ref()
                    .map(|v| self.lower_expression(v))
                    .transpose()?;
                Ok(RirStatement::Return {
                    value: rir_value.map(Box::new),
                    span: *span,
                })
            }
        }
    }

    /// Lowers an expression
    fn lower_expression(&mut self, expr: &AstExpression) -> Result<RirExpression> {
        match expr {
            AstExpression::Integer { value, span } => Ok(RirExpression::IntLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::Float { value, span } => Ok(RirExpression::FloatLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::String { value, span } => Ok(RirExpression::StringLiteral {
                value: value.clone(),
                span: *span,
            }),

            AstExpression::Boolean { value, span } => Ok(RirExpression::BoolLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::Null { span } => Ok(RirExpression::Unit { span: *span }),

            AstExpression::Variable { name, span } => {
                // TODO: Look up variable type from symbol table
                Ok(RirExpression::Variable {
                    name: name.clone(),
                    type_id: TypeId::INT, // Placeholder
                    span: *span,
                })
            }

            AstExpression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_expr = self.lower_expression(left)?;
                let right_expr = self.lower_expression(right)?;
                let op = self.lower_binary_op(operator);
                let result_type = self.infer_binary_result_type(&left_expr, &right_expr, op);

                Ok(RirExpression::Binary {
                    op,
                    left: Box::new(left_expr),
                    right: Box::new(right_expr),
                    result_type,
                    span: *span,
                })
            }

            AstExpression::Unary {
                operator,
                operand,
                span,
            } => {
                let operand_expr = self.lower_expression(operand)?;
                let op = self.lower_unary_op(operator);
                let result_type = operand_expr.type_id();

                Ok(RirExpression::Unary {
                    op,
                    operand: Box::new(operand_expr),
                    result_type,
                    span: *span,
                })
            }

            AstExpression::Call {
                callee,
                arguments,
                span,
            } => {
                let args = arguments
                    .iter()
                    .map(|arg| self.lower_expression(arg))
                    .collect::<Result<Vec<_>>>()?;

                // TODO: Look up function return type from symbol table
                Ok(RirExpression::Call {
                    function: callee.clone(),
                    arguments: args,
                    return_type: TypeId::UNIT, // Placeholder
                    span: *span,
                })
            }

            AstExpression::Array { elements, span } => {
                let rir_elements = elements
                    .iter()
                    .map(|e| self.lower_expression(e))
                    .collect::<Result<Vec<_>>>()?;

                let element_type = if let Some(first) = rir_elements.first() {
                    first.type_id()
                } else {
                    TypeId::INT // Default for empty arrays
                };

                Ok(RirExpression::ArrayLiteral {
                    elements: rir_elements,
                    element_type,
                    span: *span,
                })
            }
        }
    }

    /// Converts AST binary operator to RIR binary operator
    fn lower_binary_op(&self, op: &BinaryOperator) -> BinaryOp {
        match op {
            BinaryOperator::Add => BinaryOp::Add,
            BinaryOperator::Subtract => BinaryOp::Subtract,
            BinaryOperator::Multiply => BinaryOp::Multiply,
            BinaryOperator::Divide => BinaryOp::Divide,
            BinaryOperator::Modulo => BinaryOp::Modulo,
            BinaryOperator::Equal => BinaryOp::Equal,
            BinaryOperator::NotEqual => BinaryOp::NotEqual,
            BinaryOperator::Less => BinaryOp::LessThan,
            BinaryOperator::LessEqual => BinaryOp::LessEqual,
            BinaryOperator::Greater => BinaryOp::GreaterThan,
            BinaryOperator::GreaterEqual => BinaryOp::GreaterEqual,
            BinaryOperator::And => BinaryOp::And,
            BinaryOperator::Or => BinaryOp::Or,
        }
    }

    /// Converts AST unary operator to RIR unary operator
    fn lower_unary_op(&self, op: &UnaryOperator) -> UnaryOp {
        match op {
            UnaryOperator::Negate => UnaryOp::Negate,
            UnaryOperator::Not => UnaryOp::Not,
        }
    }

    /// Converts AST type to TypeId
    fn type_to_type_id(&self, typ: &rive_core::types::Type) -> TypeId {
        match typ {
            rive_core::types::Type::Int => TypeId::INT,
            rive_core::types::Type::Float => TypeId::FLOAT,
            rive_core::types::Type::Bool => TypeId::BOOL,
            rive_core::types::Type::Text => TypeId::TEXT,
            rive_core::types::Type::Unit => TypeId::UNIT,
            rive_core::types::Type::Array(..) => TypeId::INT, // Placeholder
            rive_core::types::Type::Optional(_) => TypeId::INT, // Placeholder
            rive_core::types::Type::Function { .. } => TypeId::INT, // Placeholder
        }
    }

    /// Infers the result type of a binary operation
    fn infer_binary_result_type(
        &self,
        left: &RirExpression,
        _right: &RirExpression,
        op: BinaryOp,
    ) -> TypeId {
        if op.is_comparison() || op.is_logical() {
            TypeId::BOOL
        } else {
            left.type_id()
        }
    }

    /// Determines the memory strategy for a given type
    fn determine_memory_strategy(&self, type_id: TypeId) -> MemoryStrategy {
        // Simple heuristic: primitives are Copy, everything else uses CoW
        match type_id {
            TypeId::INT | TypeId::FLOAT | TypeId::BOOL | TypeId::UNIT => MemoryStrategy::Copy,
            TypeId::TEXT => MemoryStrategy::CoW, // Strings use copy-on-write
            _ => MemoryStrategy::CoW,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_core::span::{Location, Span};
    use rive_core::types::Type;
    use rive_parser::ast::*;

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_lower_simple_function() {
        let span = dummy_span();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Unit,
                body: Block {
                    statements: vec![Statement::Return { value: None, span }],
                    span,
                },
                span,
            })],
        };

        let registry = TypeRegistry::new();
        let mut lowering = AstLowering::new(registry);
        let result = lowering.lower_program(&program);

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "test");
    }

    #[test]
    fn test_lower_let_statement() {
        let span = dummy_span();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Unit,
                body: Block {
                    statements: vec![Statement::Let {
                        name: "x".to_string(),
                        mutable: false,
                        var_type: Some(Type::Int),
                        initializer: Expression::Integer { value: 42, span },
                        span,
                    }],
                    span,
                },
                span,
            })],
        };

        let registry = TypeRegistry::new();
        let mut lowering = AstLowering::new(registry);
        let result = lowering.lower_program(&program);

        assert!(result.is_ok());
        let module = result.unwrap();
        let func = &module.functions[0];
        assert_eq!(func.body.statements.len(), 1);
    }

    #[test]
    fn test_lower_binary_expression() {
        let span = dummy_span();
        let mut lowering = AstLowering::new(TypeRegistry::new());

        let expr = Expression::Binary {
            left: Box::new(Expression::Integer { value: 1, span }),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Integer { value: 2, span }),
            span,
        };

        let result = lowering.lower_expression(&expr);
        assert!(result.is_ok());

        let rir_expr = result.unwrap();
        assert!(matches!(rir_expr, RirExpression::Binary { .. }));
    }

    #[test]
    fn test_memory_strategy_determination() {
        let registry = TypeRegistry::new();
        let lowering = AstLowering::new(registry);

        assert_eq!(
            lowering.determine_memory_strategy(TypeId::INT),
            MemoryStrategy::Copy
        );
        assert_eq!(
            lowering.determine_memory_strategy(TypeId::TEXT),
            MemoryStrategy::CoW
        );
    }
}
