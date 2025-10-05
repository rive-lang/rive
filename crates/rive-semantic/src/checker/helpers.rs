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

    /// Checks if two types are compatible for assignment.
    ///
    /// This delegates to the TypeRegistry's compatibility checking,
    /// which handles:
    /// - Exact type matches
    /// - T → T? implicit conversions
    /// - Null → T? implicit conversions
    /// - Other implicit conversions defined by the type system
    pub(crate) fn types_compatible(&self, target: TypeId, source: TypeId) -> bool {
        self.symbols.type_registry().are_compatible(target, source)
    }
}
