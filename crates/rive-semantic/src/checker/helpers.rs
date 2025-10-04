//! Helper functions to reduce code duplication.

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result, Span};

impl TypeChecker {
    /// Creates a type mismatch error with formatted type names.
    pub(crate) fn type_mismatch_error(
        &self,
        message: &str,
        expected: TypeId,
        found: TypeId,
        span: Span,
    ) -> Error {
        let registry = self.symbols.type_registry();
        let expected_str = registry.get_type_name(expected);
        let found_str = registry.get_type_name(found);
        Error::SemanticWithSpan(
            format!("{message}: expected '{expected_str}', found '{found_str}'"),
            span,
        )
    }

    /// Validates that a condition expression is Bool type.
    pub(crate) fn check_bool_condition(
        &mut self,
        condition: &rive_parser::Expression,
        context: &str,
        span: Span,
    ) -> Result<()> {
        let condition_type = self.check_expression(condition)?;

        if condition_type != TypeId::BOOL {
            let registry = self.symbols.type_registry();
            let cond_str = registry.get_type_name(condition_type);
            return Err(Error::SemanticWithSpan(
                format!("{context} condition must be Bool, found '{cond_str}'"),
                span,
            ));
        }

        Ok(())
    }

    /// Validates loop depth for break/continue statements.
    pub(crate) fn validate_loop_depth(&self, depth: Option<u32>, span: Span) -> Result<usize> {
        let actual_depth = depth.unwrap_or(1) as usize;

        if actual_depth == 0 {
            return Err(Error::SemanticWithSpan(
                "Depth must be at least 1".to_string(),
                span,
            ));
        }

        if actual_depth > self.loop_stack.len() {
            return Err(Error::SemanticWithSpan(
                format!(
                    "Depth {} exceeds loop nesting level {}",
                    actual_depth,
                    self.loop_stack.len()
                ),
                span,
            ));
        }

        Ok(actual_depth)
    }

    /// Checks if two types are compatible.
    pub(crate) fn types_compatible(&self, a: TypeId, b: TypeId) -> bool {
        a == b
    }
}
