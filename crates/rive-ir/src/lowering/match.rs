//! Match expression and pattern lowering from AST to RIR.

use crate::lowering::core::AstLowering;
use crate::{RirBlock, RirExpression, RirPattern, RirStatement};
use rive_core::{Error, Result, TypeId};
use rive_parser::control_flow::{Match, Pattern};

impl AstLowering {
    /// Lowers a match expression to RIR.
    pub(crate) fn lower_match_expr(&mut self, match_expr: &Match) -> Result<RirExpression> {
        let scrutinee = Box::new(self.lower_expression(&match_expr.scrutinee)?);

        let arms: Result<Vec<_>> = match_expr
            .arms
            .iter()
            .map(|arm| {
                // Enter new scope for each arm
                self.enter_scope();

                // Extract bindings from pattern and define them in the scope
                self.extract_pattern_bindings(&arm.pattern)?;

                let pattern = self.lower_pattern(&arm.pattern)?;
                let body = Box::new(self.lower_expression(&arm.body)?);

                // Exit scope after processing the arm
                self.exit_scope();

                Ok((pattern, body))
            })
            .collect();

        let arms = arms?;

        // Get result type from first arm (all arms have same type after semantic analysis)
        let result_type = arms
            .first()
            .map(|(_, expr)| expr.type_id())
            .unwrap_or(TypeId::UNIT);

        Ok(RirExpression::Match {
            scrutinee,
            arms,
            result_type,
            span: match_expr.span,
        })
    }

    /// Lowers a match as a statement.
    pub(crate) fn lower_match_stmt(&mut self, match_expr: &Match) -> Result<RirStatement> {
        let scrutinee = Box::new(self.lower_expression(&match_expr.scrutinee)?);

        let arms: Result<Vec<_>> = match_expr
            .arms
            .iter()
            .map(|arm| {
                // Enter new scope for each arm
                self.enter_scope();

                // Extract bindings from pattern and define them in the scope
                self.extract_pattern_bindings(&arm.pattern)?;

                let pattern = self.lower_pattern(&arm.pattern)?;

                // Convert expression to block
                let body_expr = self.lower_expression(&arm.body)?;
                let body = RirBlock {
                    statements: vec![RirStatement::Expression {
                        expr: Box::new(body_expr),
                        span: arm.span,
                    }],
                    final_expr: None,
                    span: arm.span,
                };

                // Exit scope after processing the arm
                self.exit_scope();

                Ok((pattern, body))
            })
            .collect();

        Ok(RirStatement::Match {
            scrutinee,
            arms: arms?,
            span: match_expr.span,
        })
    }

    /// Lowers a pattern to RIR.
    pub(crate) fn lower_pattern(&mut self, pattern: &Pattern) -> Result<RirPattern> {
        Ok(match pattern {
            Pattern::Integer { value, span } => RirPattern::IntLiteral {
                value: *value,
                span: *span,
            },
            Pattern::Float { value, span } => RirPattern::FloatLiteral {
                value: *value,
                span: *span,
            },
            Pattern::String { value, span } => RirPattern::StringLiteral {
                value: value.clone(),
                span: *span,
            },
            Pattern::Boolean { value, span } => RirPattern::BoolLiteral {
                value: *value,
                span: *span,
            },
            Pattern::Null { span: _ } => {
                return Err(Error::Semantic(
                    "Null patterns not yet supported".to_string(),
                ));
            }
            Pattern::Wildcard { span } => RirPattern::Wildcard { span: *span },
            Pattern::Range {
                start,
                end,
                inclusive,
                span,
            } => {
                let start_expr = self.lower_expression(start)?;
                let end_expr = self.lower_expression(end)?;
                RirPattern::RangePattern {
                    start: Box::new(start_expr),
                    end: Box::new(end_expr),
                    inclusive: *inclusive,
                    span: *span,
                }
            }
            Pattern::EnumVariant {
                enum_name,
                variant_name,
                bindings,
                span,
            } => {
                use rive_core::type_system::TypeKind;

                // Get enum type ID
                let enum_type_id = self
                    .type_registry
                    .get_by_name(enum_name)
                    .ok_or_else(|| Error::Semantic(format!("Unknown enum '{}'", enum_name)))?;

                // Get the variant's field names from type registry
                let type_meta = self.type_registry.get_type_metadata(enum_type_id);
                let field_names: Vec<String> = match &type_meta.kind {
                    TypeKind::Enum { variants, .. } => {
                        let variant = variants
                            .iter()
                            .find(|v| &v.name == variant_name)
                            .ok_or_else(|| {
                                Error::Semantic(format!(
                                    "Variant '{}' not found in enum '{}'",
                                    variant_name, enum_name
                                ))
                            })?;
                        variant
                            .fields
                            .as_ref()
                            .map(|fields| fields.iter().map(|(name, _)| name.clone()).collect())
                            .unwrap_or_default()
                    }
                    _ => Vec::new(),
                };

                // Convert bindings: map pattern bindings to actual field names
                let rir_bindings = bindings.as_ref().map(|b| {
                    b.iter()
                        .enumerate()
                        .map(|(i, (pattern_field, binding))| {
                            // Check if pattern_field matches the actual field name
                            let actual_field_name = field_names
                                .get(i)
                                .cloned()
                                .unwrap_or_else(|| pattern_field.clone());
                            let binding_name = if &actual_field_name == pattern_field {
                                // Named matching: use binding if provided, otherwise use field name
                                binding.as_ref().unwrap_or(pattern_field).clone()
                            } else {
                                // Positional matching: pattern_field is the binding name
                                pattern_field.clone()
                            };
                            (actual_field_name, binding_name)
                        })
                        .collect()
                });

                RirPattern::EnumVariant {
                    enum_type_id,
                    variant_name: variant_name.clone(),
                    bindings: rir_bindings,
                    span: *span,
                }
            }
            Pattern::Multiple { patterns, .. } => {
                // Expand multiple patterns - this should be handled at a higher level
                // For now, just lower the first pattern
                if let Some(first) = patterns.first() {
                    return self.lower_pattern(first);
                } else {
                    return Err(Error::Semantic("Empty multiple pattern".to_string()));
                }
            }
            Pattern::Guarded { pattern, .. } => {
                // Guards should be handled at a higher level
                // For now, just lower the inner pattern
                return self.lower_pattern(pattern);
            }
        })
    }

    /// Extracts variable bindings from a pattern and defines them in the current scope.
    fn extract_pattern_bindings(&mut self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::EnumVariant { bindings, .. } => {
                if let Some(bindings) = bindings {
                    for (field_name, binding) in bindings {
                        // If binding_name is None, use field_name as binding (per AST documentation)
                        let binding_name = binding.as_ref().unwrap_or(field_name);

                        // Get the field type from the enum variant
                        // For now, assume Text type for simplicity
                        // TODO: Look up actual field type from enum definition
                        let field_type = TypeId::TEXT;
                        self.define_variable(binding_name.clone(), field_type, false);
                    }
                }
            }
            Pattern::Guarded { pattern, .. } => {
                // Recursively extract bindings from the inner pattern
                self.extract_pattern_bindings(pattern)?;
            }
            Pattern::Multiple { patterns, .. } => {
                // Extract bindings from all patterns
                for pat in patterns {
                    self.extract_pattern_bindings(pat)?;
                }
            }
            _ => {
                // Other patterns don't have bindings
            }
        }
        Ok(())
    }
}
