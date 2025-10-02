//! Builder utilities for constructing RIR structures.

use rive_core::{
    span::Span,
    type_system::{MemoryStrategy, TypeId},
};

use crate::{BinaryOp, RirBlock, RirExpression, RirFunction, RirModule, RirStatement, UnaryOp};

/// Builder for constructing RIR modules
pub struct RirBuilder {
    module: RirModule,
}

impl RirBuilder {
    /// Creates a new RIR builder
    #[must_use]
    pub fn new(module: RirModule) -> Self {
        Self { module }
    }

    /// Adds a function to the module
    pub fn add_function(&mut self, function: RirFunction) -> &mut Self {
        self.module.add_function(function);
        self
    }

    /// Builds and returns the module
    #[must_use]
    pub fn build(self) -> RirModule {
        self.module
    }

    /// Returns a mutable reference to the module
    pub fn module_mut(&mut self) -> &mut RirModule {
        &mut self.module
    }
}

/// Builder for constructing RIR blocks
pub struct BlockBuilder {
    block: RirBlock,
}

impl BlockBuilder {
    /// Creates a new block builder
    #[must_use]
    pub fn new(span: Span) -> Self {
        Self {
            block: RirBlock::new(span),
        }
    }

    /// Adds a statement to the block
    pub fn add_statement(&mut self, statement: RirStatement) -> &mut Self {
        self.block.add_statement(statement);
        self
    }

    /// Adds a let statement
    pub fn add_let(
        &mut self,
        name: String,
        type_id: TypeId,
        is_mutable: bool,
        value: RirExpression,
        memory_strategy: MemoryStrategy,
        span: Span,
    ) -> &mut Self {
        self.add_statement(RirStatement::Let {
            name,
            type_id,
            is_mutable,
            value: Box::new(value),
            memory_strategy,
            span,
        })
    }

    /// Adds an assignment statement
    pub fn add_assign(&mut self, name: String, value: RirExpression, span: Span) -> &mut Self {
        self.add_statement(RirStatement::Assign {
            name,
            value: Box::new(value),
            span,
        })
    }

    /// Adds a return statement
    pub fn add_return(&mut self, value: Option<RirExpression>, span: Span) -> &mut Self {
        self.add_statement(RirStatement::Return {
            value: value.map(Box::new),
            span,
        })
    }

    /// Adds an expression statement
    pub fn add_expression(&mut self, expr: RirExpression, span: Span) -> &mut Self {
        self.add_statement(RirStatement::Expression {
            expr: Box::new(expr),
            span,
        })
    }

    /// Sets the final expression of the block
    pub fn set_final_expr(&mut self, expr: RirExpression) -> &mut Self {
        self.block.set_final_expr(expr);
        self
    }

    /// Builds and returns the block
    #[must_use]
    pub fn build(self) -> RirBlock {
        self.block
    }
}

/// Helper functions for creating common expressions
pub struct ExprBuilder;

impl ExprBuilder {
    /// Creates an integer literal
    #[must_use]
    pub fn int(value: i64, span: Span) -> RirExpression {
        RirExpression::IntLiteral { value, span }
    }

    /// Creates a float literal
    #[must_use]
    pub fn float(value: f64, span: Span) -> RirExpression {
        RirExpression::FloatLiteral { value, span }
    }

    /// Creates a string literal
    #[must_use]
    pub fn string(value: String, span: Span) -> RirExpression {
        RirExpression::StringLiteral { value, span }
    }

    /// Creates a boolean literal
    #[must_use]
    pub fn bool(value: bool, span: Span) -> RirExpression {
        RirExpression::BoolLiteral { value, span }
    }

    /// Creates a unit expression
    #[must_use]
    pub fn unit(span: Span) -> RirExpression {
        RirExpression::Unit { span }
    }

    /// Creates a variable reference
    #[must_use]
    pub fn var(name: String, type_id: TypeId, span: Span) -> RirExpression {
        RirExpression::Variable {
            name,
            type_id,
            span,
        }
    }

    /// Creates a binary operation
    #[must_use]
    pub fn binary(
        op: BinaryOp,
        left: RirExpression,
        right: RirExpression,
        result_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            result_type,
            span,
        }
    }

    /// Creates a unary operation
    #[must_use]
    pub fn unary(
        op: UnaryOp,
        operand: RirExpression,
        result_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Unary {
            op,
            operand: Box::new(operand),
            result_type,
            span,
        }
    }

    /// Creates a function call
    #[must_use]
    pub fn call(
        function: String,
        arguments: Vec<RirExpression>,
        return_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Call {
            function,
            arguments,
            return_type,
            span,
        }
    }

    /// Creates an array literal
    #[must_use]
    pub fn array(elements: Vec<RirExpression>, element_type: TypeId, span: Span) -> RirExpression {
        RirExpression::ArrayLiteral {
            elements,
            element_type,
            span,
        }
    }

    /// Creates an array index expression
    #[must_use]
    pub fn index(
        array: RirExpression,
        index: RirExpression,
        element_type: TypeId,
        span: Span,
    ) -> RirExpression {
        RirExpression::Index {
            array: Box::new(array),
            index: Box::new(index),
            element_type,
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_core::{span::Location, type_system::TypeRegistry};

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_expr_builder() {
        let span = dummy_span();

        let int_expr = ExprBuilder::int(42, span);
        assert_eq!(int_expr.type_id(), TypeId::INT);

        let bool_expr = ExprBuilder::bool(true, span);
        assert_eq!(bool_expr.type_id(), TypeId::BOOL);

        let var_expr = ExprBuilder::var("x".to_string(), TypeId::INT, span);
        assert_eq!(var_expr.type_id(), TypeId::INT);
    }

    #[test]
    fn test_block_builder() {
        let span = dummy_span();
        let mut builder = BlockBuilder::new(span);

        builder
            .add_let(
                "x".to_string(),
                TypeId::INT,
                false,
                ExprBuilder::int(42, span),
                MemoryStrategy::Copy,
                span,
            )
            .add_return(
                Some(ExprBuilder::var("x".to_string(), TypeId::INT, span)),
                span,
            );

        let block = builder.build();
        assert_eq!(block.statements.len(), 2);
    }

    #[test]
    fn test_module_builder() {
        let span = dummy_span();
        let registry = TypeRegistry::new();
        let module = RirModule::new(registry);
        let mut builder = RirBuilder::new(module);

        let func = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock::new(span),
            span,
        );

        builder.add_function(func);
        let module = builder.build();

        assert_eq!(module.functions.len(), 1);
        assert!(module.get_function("test").is_some());
    }

    #[test]
    fn test_complex_expression_building() {
        let span = dummy_span();

        // Build: (1 + 2) * 3
        let left = ExprBuilder::binary(
            BinaryOp::Add,
            ExprBuilder::int(1, span),
            ExprBuilder::int(2, span),
            TypeId::INT,
            span,
        );

        let expr = ExprBuilder::binary(
            BinaryOp::Multiply,
            left,
            ExprBuilder::int(3, span),
            TypeId::INT,
            span,
        );

        assert_eq!(expr.type_id(), TypeId::INT);
    }
}
