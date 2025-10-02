//! RIR module structure - top-level representation of a Rive program.

use rive_core::{
    span::Span,
    type_system::{MemoryStrategy, TypeId, TypeRegistry},
};

use crate::{RirExpression, RirStatement};

/// Top-level RIR module representing a complete Rive program
#[derive(Debug, Clone)]
pub struct RirModule {
    /// All functions in the module
    pub functions: Vec<RirFunction>,
    /// Type registry shared across the module
    pub type_registry: TypeRegistry,
}

impl RirModule {
    /// Creates a new empty RIR module
    #[must_use]
    pub fn new(type_registry: TypeRegistry) -> Self {
        Self {
            functions: Vec::new(),
            type_registry,
        }
    }

    /// Adds a function to the module
    pub fn add_function(&mut self, function: RirFunction) {
        self.functions.push(function);
    }

    /// Finds a function by name
    #[must_use]
    pub fn get_function(&self, name: &str) -> Option<&RirFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Returns the main function if it exists
    #[must_use]
    pub fn main_function(&self) -> Option<&RirFunction> {
        self.get_function("main")
    }
}

/// A function in RIR
#[derive(Debug, Clone)]
pub struct RirFunction {
    /// Function name
    pub name: String,
    /// Function parameters
    pub parameters: Vec<RirParameter>,
    /// Return type
    pub return_type: TypeId,
    /// Function body as a block
    pub body: RirBlock,
    /// Source location
    pub span: Span,
}

impl RirFunction {
    /// Creates a new RIR function
    #[must_use]
    pub fn new(
        name: String,
        parameters: Vec<RirParameter>,
        return_type: TypeId,
        body: RirBlock,
        span: Span,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            body,
            span,
        }
    }

    /// Returns true if this is the main function
    #[must_use]
    pub fn is_main(&self) -> bool {
        self.name == "main"
    }

    /// Returns true if this function returns Unit
    #[must_use]
    pub fn returns_unit(&self) -> bool {
        self.return_type == TypeId::UNIT
    }
}

/// A function parameter
#[derive(Debug, Clone)]
pub struct RirParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_id: TypeId,
    /// Whether this parameter is mutable
    pub is_mutable: bool,
    /// Memory strategy for this parameter
    pub memory_strategy: MemoryStrategy,
    /// Source location
    pub span: Span,
}

impl RirParameter {
    /// Creates a new parameter
    #[must_use]
    pub fn new(
        name: String,
        type_id: TypeId,
        is_mutable: bool,
        memory_strategy: MemoryStrategy,
        span: Span,
    ) -> Self {
        Self {
            name,
            type_id,
            is_mutable,
            memory_strategy,
            span,
        }
    }
}

/// A block of statements
#[derive(Debug, Clone)]
pub struct RirBlock {
    /// Statements in this block
    pub statements: Vec<RirStatement>,
    /// Optional final expression (for blocks that return values)
    pub final_expr: Option<Box<RirExpression>>,
    /// Source location
    pub span: Span,
}

impl RirBlock {
    /// Creates a new empty block
    #[must_use]
    pub fn new(span: Span) -> Self {
        Self {
            statements: Vec::new(),
            final_expr: None,
            span,
        }
    }

    /// Creates a block with statements
    #[must_use]
    pub fn with_statements(statements: Vec<RirStatement>, span: Span) -> Self {
        Self {
            statements,
            final_expr: None,
            span,
        }
    }

    /// Creates a block with statements and a final expression
    #[must_use]
    pub fn with_final_expr(
        statements: Vec<RirStatement>,
        final_expr: RirExpression,
        span: Span,
    ) -> Self {
        Self {
            statements,
            final_expr: Some(Box::new(final_expr)),
            span,
        }
    }

    /// Adds a statement to the block
    pub fn add_statement(&mut self, statement: RirStatement) {
        self.statements.push(statement);
    }

    /// Sets the final expression
    pub fn set_final_expr(&mut self, expr: RirExpression) {
        self.final_expr = Some(Box::new(expr));
    }

    /// Returns true if this block is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty() && self.final_expr.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_core::span::Location;

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_module_creation() {
        let registry = TypeRegistry::new();
        let module = RirModule::new(registry);
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_function_creation() {
        let span = dummy_span();
        let func = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock::new(span),
            span,
        );
        assert_eq!(func.name, "test");
        assert!(func.parameters.is_empty());
        assert!(func.returns_unit());
    }

    #[test]
    fn test_main_function_detection() {
        let span = dummy_span();
        let main_func = RirFunction::new(
            "main".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock::new(span),
            span,
        );
        assert!(main_func.is_main());

        let other_func = RirFunction::new(
            "foo".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock::new(span),
            span,
        );
        assert!(!other_func.is_main());
    }

    #[test]
    fn test_block_operations() {
        let span = dummy_span();
        let mut block = RirBlock::new(span);
        assert!(block.is_empty());

        // Block becomes non-empty after adding statements
        block.add_statement(RirStatement::Expression {
            expr: Box::new(RirExpression::Unit { span }),
            span,
        });
        assert!(!block.is_empty());
    }

    #[test]
    fn test_parameter_creation() {
        let span = dummy_span();
        let param = RirParameter::new(
            "x".to_string(),
            TypeId::INT,
            false,
            MemoryStrategy::Copy,
            span,
        );
        assert_eq!(param.name, "x");
        assert_eq!(param.type_id, TypeId::INT);
        assert!(!param.is_mutable);
    }
}
