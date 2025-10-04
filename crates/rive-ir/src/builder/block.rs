//! Builder for constructing RIR blocks.

use rive_core::{
    span::Span,
    type_system::{MemoryStrategy, TypeId},
};

use crate::{RirBlock, RirExpression, RirStatement};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExprBuilder;
    use rive_core::span::Location;

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
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
}
