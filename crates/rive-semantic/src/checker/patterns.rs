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
        let mut covered_variants = std::collections::HashSet::new();

        for arm in &match_expr.arms {
            // Enter a new scope for this arm to isolate pattern bindings
            self.symbols.enter_scope();

            // Check pattern matches scrutinee type and define bindings
            self.check_pattern(&arm.pattern, scrutinee_type)?;

            match &arm.pattern {
                Pattern::Wildcard { .. } => has_wildcard = true,
                Pattern::Boolean { value: true, .. } => has_true = true,
                Pattern::Boolean { value: false, .. } => has_false = true,
                Pattern::EnumVariant { variant_name, .. } => {
                    covered_variants.insert(variant_name.clone());
                }
                Pattern::Multiple { patterns, .. } => {
                    // For multiple patterns, collect all enum variants
                    for pat in patterns {
                        if let Pattern::EnumVariant { variant_name, .. } = pat {
                            covered_variants.insert(variant_name.clone());
                        }
                    }
                }
                _ => {}
            }

            // Check arm body type (with pattern bindings in scope)
            let arm_type = self.check_expression(&arm.body)?;
            arm_types.push(arm_type);

            // Exit the arm scope
            self.symbols.exit_scope();
        }

        // Check exhaustiveness
        let is_exhaustive = has_wildcard
            || (scrutinee_type == TypeId::BOOL && has_true && has_false)
            || self.check_enum_exhaustiveness(scrutinee_type, &covered_variants)?;

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
            Pattern::EnumVariant {
                enum_name,
                variant_name,
                bindings,
                span,
            } => {
                return self.check_enum_variant_pattern(
                    enum_name,
                    variant_name,
                    bindings,
                    expected_type,
                    *span,
                );
            }
            Pattern::Multiple { patterns, .. } => {
                // Check all patterns in the multiple pattern
                for pat in patterns {
                    self.check_pattern(pat, expected_type)?;
                }
                return Ok(());
            }
            Pattern::Guarded { pattern, guard, .. } => {
                // Check the inner pattern
                self.check_pattern(pattern, expected_type)?;
                // Check that guard is boolean
                let guard_type = self.check_expression(guard)?;
                if guard_type != TypeId::BOOL {
                    return Err(self.type_mismatch_error(
                        "Guard condition must be boolean",
                        TypeId::BOOL,
                        guard_type,
                        guard.span(),
                    ));
                }
                return Ok(());
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

    /// Checks an enum variant pattern.
    fn check_enum_variant_pattern(
        &mut self,
        enum_name: &str,
        variant_name: &str,
        bindings: &Option<Vec<(String, Option<String>)>>,
        expected_type: TypeId,
        span: rive_core::Span,
    ) -> Result<()> {
        use rive_core::type_system::TypeKind;

        // Get the enum type
        let enum_type_id = self
            .symbols
            .type_registry()
            .get_by_name(enum_name)
            .ok_or_else(|| {
                Error::SemanticWithSpan(format!("Unknown enum '{}'", enum_name), span)
            })?;

        // Check that the expected type matches the enum type
        if enum_type_id != expected_type {
            return Err(self.type_mismatch_error(
                "Enum pattern type mismatch",
                expected_type,
                enum_type_id,
                span,
            ));
        }

        // Get the enum metadata and clone the variant fields
        let variant_fields = {
            let enum_metadata = self.symbols.type_registry().get_type_metadata(enum_type_id);

            // Find the variant
            match &enum_metadata.kind {
                TypeKind::Enum { variants, .. } => {
                    let variant = variants
                        .iter()
                        .find(|v| v.name == variant_name)
                        .ok_or_else(|| {
                            Error::SemanticWithSpan(
                                format!("Enum '{}' has no variant '{}'", enum_name, variant_name),
                                span,
                            )
                        })?;
                    variant.fields.clone()
                }
                _ => {
                    return Err(Error::SemanticWithSpan(
                        format!("'{}' is not an enum", enum_name),
                        span,
                    ));
                }
            }
        };

        // Check bindings match variant fields
        match (&variant_fields, bindings) {
            (None, None) => {
                // Variant has no fields and pattern has no bindings - OK
                Ok(())
            }
            (None, Some(_)) => Err(Error::SemanticWithSpan(
                format!(
                    "Variant '{}' has no fields but pattern has bindings",
                    variant_name
                ),
                span,
            )),
            (Some(_), None) => Err(Error::SemanticWithSpan(
                format!(
                    "Variant '{}' has fields but pattern has no bindings",
                    variant_name
                ),
                span,
            )),
            (Some(fields), Some(pattern_bindings)) => {
                // Check that the number of bindings matches the number of fields
                if fields.len() != pattern_bindings.len() {
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Variant '{}' has {} fields but pattern has {} bindings",
                            variant_name,
                            fields.len(),
                            pattern_bindings.len()
                        ),
                        span,
                    ));
                }

                // Define bindings in the symbol table
                // Support both positional matching (binding names can be anything)
                // and named matching (field names must match)
                for ((field_name, field_type), (pattern_field, binding_name)) in
                    fields.iter().zip(pattern_bindings.iter())
                {
                    // If pattern_field matches field_name, it's explicit field matching
                    // Otherwise, treat it as positional matching where pattern_field is the binding name
                    let binding = if field_name == pattern_field {
                        // Explicit field matching: use binding_name if provided, otherwise field_name
                        binding_name.as_ref().unwrap_or(pattern_field).clone()
                    } else {
                        // Positional matching: pattern_field is the binding name
                        // binding_name should be None in this case
                        if binding_name.is_some() {
                            return Err(Error::SemanticWithSpan(
                                "Cannot use 'as' with positional pattern matching".to_string(),
                                span,
                            ));
                        }
                        pattern_field.clone()
                    };

                    // Define the binding in the symbol table
                    let symbol = crate::symbol_table::Symbol::new(binding, *field_type, false);
                    self.symbols.define(symbol)?;
                }

                Ok(())
            }
        }
    }

    /// Checks if all enum variants are covered in a match expression.
    fn check_enum_exhaustiveness(
        &self,
        scrutinee_type: TypeId,
        covered_variants: &std::collections::HashSet<String>,
    ) -> Result<bool> {
        use rive_core::type_system::TypeKind;

        // Get type metadata
        let type_metadata = self
            .symbols
            .type_registry()
            .get_type_metadata(scrutinee_type);

        // Check if scrutinee is an enum
        if let TypeKind::Enum { variants, .. } = &type_metadata.kind {
            // Check if all variants are covered
            let all_variants: std::collections::HashSet<String> =
                variants.iter().map(|v| v.name.clone()).collect();

            Ok(covered_variants == &all_variants)
        } else {
            // Not an enum, so exhaustiveness doesn't apply
            Ok(false)
        }
    }
}
