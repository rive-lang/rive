//! Elvis and safe call operator type checking.

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Result, Span};
use rive_parser::ast::Expression;

impl TypeChecker {
    /// Checks an Elvis operator (null-coalescing): `value ?: fallback`
    ///
    /// # Type Rules
    /// - `value` can be any type (nullable or not)
    /// - If `value: T?`, then `fallback` must be compatible with `T` or `T?`
    /// - If `value: Null`, then `fallback` determines the result type
    /// - If `value: T` (non-nullable), a warning could be issued (not an error)
    /// - Result type: `T` if fallback is `T`, or `T?` if fallback is `T?`
    pub(super) fn check_elvis(
        &mut self,
        value: &Expression,
        fallback: &Expression,
        span: Span,
    ) -> Result<TypeId> {
        let value_type = self.check_expression(value)?;
        let fallback_type = self.check_expression(fallback)?;

        // Special case: if value is Null, result is fallback's type
        if value_type == TypeId::NULL {
            return Ok(fallback_type);
        }

        // Check if value is nullable (T?)
        if let Some(inner_type) = self.get_nullable_inner(value_type) {
            // value is T?, fallback should be compatible with T or T?

            // Case 1: fallback is also T? (same as value)
            if fallback_type == value_type {
                return Ok(fallback_type); // T?
            }

            // Case 2: fallback is T (the inner type)
            if fallback_type == inner_type {
                return Ok(inner_type); // Result is T (non-nullable)
            }

            // Case 3: fallback is compatible with T (can be assigned to T)
            if self.types_compatible(inner_type, fallback_type) {
                // If fallback can convert to T, result is T
                return Ok(inner_type);
            }

            // Case 4: fallback is T2? where T2 is compatible with T
            if let Some(fallback_inner) = self.get_nullable_inner(fallback_type)
                && self.types_compatible(inner_type, fallback_inner)
            {
                return Ok(fallback_type); // Result is T?
            }

            // Type mismatch
            return Err(self.type_mismatch_error(
                "Elvis operator: fallback type must be compatible with the nullable inner type",
                inner_type,
                fallback_type,
                span,
            ));
        }

        // Value is non-nullable (T), Elvis operator is redundant but valid
        // Result is the value's type (T) since it's never null
        // We could emit a warning here in the future
        Ok(value_type)
    }

    /// Checks a safe call operator: `object?.call`
    ///
    /// # Type Rules
    /// - `object` can be any type, but typically should be `T?`
    /// - If `object: T?` and `call: R`, result is `R?`
    /// - If `object: T` (non-nullable), we still return `R?` for consistency
    /// - Safe call always returns a nullable type
    ///
    /// # Note
    /// In the current implementation, `call` is an independent expression.
    /// For full OOP support (methods/fields), this would need refactoring.
    pub(super) fn check_safe_call(
        &mut self,
        object: &Expression,
        call: &Expression,
        _span: Span,
    ) -> Result<TypeId> {
        // Check the object type
        let _object_type = self.check_expression(object)?;

        // Note: We don't strictly require object to be nullable here,
        // as safe call can be used defensively even with non-nullable types.
        // This is consistent with languages like Kotlin.

        // Check the call expression
        let call_type = self.check_expression(call)?;

        // Result is always nullable: if call returns R, safe call returns R?
        if self.get_nullable_inner(call_type).is_some() {
            // Call already returns R?, so safe call also returns R?
            Ok(call_type)
        } else {
            // Call returns R, so safe call returns R?
            Ok(self.symbols.type_registry_mut().create_optional(call_type))
        }
    }
}
