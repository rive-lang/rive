//! AST to RIR lowering - converts parsed AST into RIR for optimization.

use rive_core::{
    Error, Result,
    type_system::{MemoryStrategy, TypeId, TypeRegistry},
};
use rive_parser::Expression;
use rive_parser::ast::{
    BinaryOperator, Block as AstBlock, Expression as AstExpression, Function as AstFunction, Item,
    Program, Statement as AstStatement, UnaryOperator,
};
use rive_parser::control_flow::{Break, Continue, For, If, Loop, Match, Pattern, Range, While};
use std::collections::HashMap;

use crate::{
    BinaryOp, RirBlock, RirExpression, RirFunction, RirModule, RirParameter, RirPattern,
    RirStatement, UnaryOp,
};

/// Symbol information during lowering
#[derive(Debug, Clone)]
struct SymbolInfo {
    type_id: TypeId,
    #[allow(dead_code)] // Will be used in future for mutability checks
    mutable: bool,
}

/// Converts AST to RIR with type information
pub struct AstLowering {
    pub(crate) type_registry: TypeRegistry,
    /// Local symbol table for tracking variables and their types
    symbols: Vec<HashMap<String, SymbolInfo>>,
    /// Function signatures for function calls
    functions: HashMap<String, (Vec<TypeId>, TypeId)>,
    /// Current loop nesting depth
    pub(crate) loop_depth: usize,
    /// Stack of loop labels for break/continue
    pub(crate) loop_labels: Vec<Option<String>>,
}

impl AstLowering {
    /// Creates a new AST lowering instance
    #[must_use]
    pub fn new(type_registry: TypeRegistry) -> Self {
        Self {
            type_registry,
            symbols: vec![HashMap::new()], // Global scope
            functions: HashMap::new(),
            loop_depth: 0,
            loop_labels: Vec::new(),
        }
    }

    /// Enters a new scope
    fn enter_scope(&mut self) {
        self.symbols.push(HashMap::new());
    }

    /// Exits the current scope
    fn exit_scope(&mut self) {
        self.symbols.pop();
    }

    /// Defines a variable in the current scope
    fn define_variable(&mut self, name: String, type_id: TypeId, mutable: bool) {
        if let Some(scope) = self.symbols.last_mut() {
            scope.insert(name, SymbolInfo { type_id, mutable });
        }
    }

    /// Looks up a variable in all scopes
    fn lookup_variable(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.symbols.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Defines a function signature
    fn define_function(&mut self, name: String, params: Vec<TypeId>, return_type: TypeId) {
        self.functions.insert(name, (params, return_type));
    }

    /// Looks up a function signature
    fn lookup_function(&self, name: &str) -> Option<&(Vec<TypeId>, TypeId)> {
        self.functions.get(name)
    }

    /// Lowers a complete program to RIR
    pub fn lower_program(&mut self, program: &Program) -> Result<RirModule> {
        let mut module = RirModule::new(self.type_registry.clone());

        // First pass: register all function signatures
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let param_types: Vec<TypeId> =
                        func.params.iter().map(|p| p.param_type).collect();
                    let return_type = func.return_type;
                    self.define_function(func.name.clone(), param_types, return_type);
                }
            }
        }

        // Second pass: lower function bodies
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
        // Enter function scope
        self.enter_scope();

        // Register parameters in symbol table
        let parameters: Vec<RirParameter> = func
            .params
            .iter()
            .map(|p| {
                let type_id = p.param_type;
                self.define_variable(p.name.clone(), type_id, false);
                let memory_strategy = self.determine_memory_strategy(type_id);
                Ok(RirParameter::new(
                    p.name.clone(),
                    type_id,
                    false, // Parameters are not mutable by default in Rive
                    memory_strategy,
                    p.span,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        let return_type = func.return_type;
        let body = self.lower_block(&func.body)?;

        // Exit function scope
        self.exit_scope();

        Ok(RirFunction::new(
            func.name.clone(),
            parameters,
            return_type,
            body,
            func.span,
        ))
    }

    /// Lowers a block of statements
    fn lower_block(&mut self, block: &AstBlock) -> Result<RirBlock> {
        let mut rir_block = RirBlock::new(block.span);

        // Check if the last statement is an expression (for implicit return)
        let statements_count = block.statements.len();

        for (i, stmt) in block.statements.iter().enumerate() {
            let is_last = i == statements_count - 1;

            // If this is the last statement and it's an expression statement,
            // treat it as the final expression (implicit return)
            // But only for actual value expressions, not:
            // - Function calls (which return Unit)
            // - If/Match expressions (which should be handled as statements unless explicitly used as expressions)
            if is_last && let AstStatement::Expression { expression, .. } = stmt {
                // Check if this expression produces a value (not Unit)
                // Exclude Call, If, and Match as they are typically statements
                let should_be_final = !matches!(
                    expression,
                    AstExpression::Call { .. } | AstExpression::If(_) | AstExpression::Match(_)
                );

                if should_be_final {
                    let final_expr = self.lower_expression(expression)?;
                    // Only set as final_expr if it's not Unit type
                    if final_expr.type_id() != self.type_registry.get_unit() {
                        rir_block.final_expr = Some(Box::new(final_expr));
                        continue;
                    }
                }
            }

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
                // Type is already a TypeId from the parser
                let type_id = var_type.unwrap_or_else(|| value.type_id());

                // Register variable in symbol table
                self.define_variable(name.clone(), type_id, *mutable);

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
                // Special handling for control flow that can be statements
                match expression {
                    AstExpression::If(if_expr) => self.lower_if_stmt(if_expr),
                    AstExpression::Match(match_expr) => self.lower_match_stmt(match_expr),
                    _ => {
                        let rir_expr = self.lower_expression(expression)?;
                        Ok(RirStatement::Expression {
                            expr: Box::new(rir_expr),
                            span: *span,
                        })
                    }
                }
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

            AstStatement::Break(break_stmt) => self.lower_break(break_stmt),
            AstStatement::Continue(continue_stmt) => self.lower_continue(continue_stmt),
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
                // Look up variable type from symbol table
                let type_id = self
                    .lookup_variable(name)
                    .map(|info| info.type_id)
                    .ok_or_else(|| Error::Semantic(format!("Undefined variable '{name}'")))?;

                Ok(RirExpression::Variable {
                    name: name.clone(),
                    type_id,
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

                // Look up function return type from function signatures
                // Special case for built-in print function
                let return_type = if callee == "print" {
                    TypeId::UNIT
                } else {
                    self.lookup_function(callee)
                        .map(|(_, return_type)| *return_type)
                        .ok_or_else(|| Error::Semantic(format!("Undefined function '{callee}'")))?
                };

                Ok(RirExpression::Call {
                    function: callee.clone(),
                    arguments: args,
                    return_type,
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

            AstExpression::If(if_expr) => self.lower_if_expr(if_expr),
            AstExpression::While(while_loop) => {
                // While as expression: lower directly as expression
                self.lower_while_expr(while_loop)
            }
            AstExpression::For(for_loop) => {
                // For as expression: lower directly as expression
                self.lower_for_expr(for_loop)
            }
            AstExpression::Loop(loop_expr) => {
                // Loop as expression: lower directly as expression
                self.lower_loop_expr(loop_expr)
            }
            AstExpression::Match(match_expr) => self.lower_match_expr(match_expr),
            AstExpression::Range(range) => self.lower_range(range),
            AstExpression::Block(block) => self.lower_block_expr(block),
        }
    }

    /// Lowers a block expression to RIR.
    fn lower_block_expr(&mut self, block: &rive_parser::Block) -> Result<RirExpression> {
        let rir_block = self.lower_block(block)?;

        // Check if the block has a final expression
        let (result, result_type) = if let Some(ref final_expr) = rir_block.final_expr {
            (rir_block.final_expr.clone(), final_expr.type_id())
        } else {
            (None, self.type_registry.get_unit())
        };

        Ok(RirExpression::Block {
            block: rir_block,
            result,
            result_type,
            span: block.span,
        })
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

// Include control flow lowering implementation
include!("lowering_control_flow.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use rive_core::span::{Location, Span};
    use rive_parser::ast::*;

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_lower_simple_function() {
        let span = dummy_span();
        let registry = TypeRegistry::new();
        let unit_type = registry.get_unit();

        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: unit_type,
                body: Block {
                    statements: vec![Statement::Return { value: None, span }],
                    span,
                },
                span,
            })],
        };

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
        let registry = TypeRegistry::new();
        let unit_type = registry.get_unit();
        let int_type = registry.get_int();

        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: unit_type,
                body: Block {
                    statements: vec![Statement::Let {
                        name: "x".to_string(),
                        mutable: false,
                        var_type: Some(int_type),
                        initializer: Expression::Integer { value: 42, span },
                        span,
                    }],
                    span,
                },
                span,
            })],
        };

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
