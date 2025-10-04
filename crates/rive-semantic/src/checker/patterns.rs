//! Pattern matching type checking.

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::control_flow::{Match, Pattern};

impl TypeChecker {
    /// Checks a match expression.
    pub(crate) fn check_match(
        &mut self,
        match_expr: &Match,
        is_expression: bool,
    ) -> Result<TypeId> {
        let scrutinee_type = self.check_expression(&match_expr.scrutinee)?;

        if match_expr.arms.is_empty() {
            return Err(Error::SemanticWithSpan(
                "Match must have at least one arm".to_string(),
                match_expr.span,
            ));
        }

        let mut arm_types = Vec::new();
        let mut has_wildcard = false;
        let mut has_true = false;
        let mut has_false = false;

        for arm in &match_expr.arms {
            // Check pattern matches scrutinee type
            self.check_pattern(&arm.pattern, scrutinee_type)?;

            match arm.pattern {
                Pattern::Wildcard { .. } => has_wildcard = true,
                Pattern::Boolean { value: true, .. } => has_true = true,
                Pattern::Boolean { value: false, .. } => has_false = true,
                _ => {}
            }

            // Check arm body type
            let arm_type = self.check_expression(&arm.body)?;
            arm_types.push(arm_type);
        }

        // Check exhaustiveness
        let is_exhaustive =
            has_wildcard || (scrutinee_type == TypeId::BOOL && has_true && has_false);

        if !is_exhaustive {
            return Err(Error::SemanticWithSpan(
                "Match must be exhaustive (add a wildcard '_' pattern or cover all cases)"
                    .to_string(),
                match_expr.span,
            ));
        }

        // When used as an expression, all arms must return same type
        if is_expression {
            let first_type = arm_types[0];
            for (i, &arm_type) in arm_types[1..].iter().enumerate() {
                if arm_type != first_type {
                    return Err(self.type_mismatch_error(
                        &format!("Match arm {} type mismatch", i + 2),
                        first_type,
                        arm_type,
                        match_expr.arms[i + 1].span,
                    ));
                }
            }
            Ok(first_type)
        } else {
            // When used as a statement, return Unit
            Ok(TypeId::UNIT)
        }
    }

    /// Checks a pattern against expected type.
    pub(crate) fn check_pattern(&mut self, pattern: &Pattern, expected_type: TypeId) -> Result<()> {
        let pattern_type = match pattern {
            Pattern::Integer { .. } => TypeId::INT,
            Pattern::Float { .. } => TypeId::FLOAT,
            Pattern::String { .. } => TypeId::TEXT,
            Pattern::Boolean { .. } => TypeId::BOOL,
            Pattern::Null { .. } => {
                return Err(Error::SemanticWithSpan(
                    "Null patterns not yet supported".to_string(),
                    pattern.span(),
                ));
            }
            Pattern::Wildcard { .. } => return Ok(()), // Wildcard matches any type
            Pattern::Range { start, end, .. } => {
                return self.check_pattern_range(start, end, expected_type);
            }
        };

        if pattern_type != expected_type {
            return Err(self.type_mismatch_error(
                "Pattern type mismatch",
                expected_type,
                pattern_type,
                pattern.span(),
            ));
        }

        Ok(())
    }

    /// Checks a range pattern.
    fn check_pattern_range(
        &mut self,
        start: &rive_parser::Expression,
        end: &rive_parser::Expression,
        expected_type: TypeId,
    ) -> Result<()> {
        let start_type = self.check_expression(start)?;
        let end_type = self.check_expression(end)?;

        if start_type != expected_type {
            return Err(self.type_mismatch_error(
                "Range start type mismatch",
                expected_type,
                start_type,
                start.span(),
            ));
        }

        if end_type != expected_type {
            return Err(self.type_mismatch_error(
                "Range end type mismatch",
                expected_type,
                end_type,
                end.span(),
            ));
        }

        Ok(())
    }
}
