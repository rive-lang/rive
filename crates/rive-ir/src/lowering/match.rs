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
                let pattern = self.lower_pattern(&arm.pattern)?;
                let body = Box::new(self.lower_expression(&arm.body)?);
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
        })
    }
}
