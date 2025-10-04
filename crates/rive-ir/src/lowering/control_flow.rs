//! Control flow lowering from AST to RIR.

use crate::lowering::core::AstLowering;
use crate::{RirBlock, RirExpression, RirPattern, RirStatement};
use rive_core::{Error, Result, TypeId};
use rive_parser::Expression;
use rive_parser::control_flow::{Break, Continue, For, If, Loop, Match, Pattern, While};

impl AstLowering {
    /// Lowers an if expression to RIR.
    pub(crate) fn lower_if_expr(&mut self, if_expr: &If) -> Result<RirExpression> {
        let condition = Box::new(self.lower_expression(&if_expr.condition)?);
        let then_block = self.lower_block(&if_expr.then_block)?;

        // If must have else to be an expression
        let else_block = if let Some(else_blk) = &if_expr.else_block {
            self.lower_block(else_blk)?
        } else {
            // This should have been caught by semantic analysis
            return Err(Error::Semantic(
                "If expression must have else branch".to_string(),
            ));
        };

        // Get result type from the then block's last expression
        let result_type = then_block
            .final_expr
            .as_ref()
            .map(|e| e.type_id())
            .unwrap_or(TypeId::UNIT);

        Ok(RirExpression::If {
            condition,
            then_block,
            else_block,
            result_type,
            span: if_expr.span,
        })
    }

    /// Lowers an if as a statement.
    pub(crate) fn lower_if_stmt(&mut self, if_expr: &If) -> Result<RirStatement> {
        let condition = Box::new(self.lower_expression(&if_expr.condition)?);
        let then_block = self.lower_block(&if_expr.then_block)?;

        // Handle else-if chain by converting to nested if-else
        let else_block = if !if_expr.else_if_branches.is_empty() || if_expr.else_block.is_some() {
            // Build nested if-else from else-if chain
            let mut current_else = if_expr
                .else_block
                .as_ref()
                .map(|b| self.lower_block(b))
                .transpose()?;

            // Process else-if branches in reverse order
            for else_if in if_expr.else_if_branches.iter().rev() {
                let else_if_cond = Box::new(self.lower_expression(&else_if.condition)?);
                let else_if_then = self.lower_block(&else_if.block)?;

                let nested_if = RirStatement::If {
                    condition: else_if_cond,
                    then_block: else_if_then,
                    else_block: current_else,
                    span: else_if.span,
                };

                current_else = Some(RirBlock {
                    statements: vec![nested_if],
                    final_expr: None,
                    span: else_if.span,
                });
            }

            current_else
        } else {
            None
        };

        Ok(RirStatement::If {
            condition,
            then_block,
            else_block,
            span: if_expr.span,
        })
    }

    /// Lowers a while loop expression to RIR.
    pub(crate) fn lower_while_expr(&mut self, while_loop: &While) -> Result<RirExpression> {
        let condition = Box::new(self.lower_expression(&while_loop.condition)?);

        // Enter loop context for label generation
        let label = self.enter_loop();

        let body = self.lower_block(&while_loop.body)?;

        // Exit loop context
        self.exit_loop();

        // Get result type from break with value or Unit if none
        let result_type = self.infer_loop_result_type(&body);

        Ok(RirExpression::While {
            condition,
            body,
            label,
            result_type,
            span: while_loop.span,
        })
    }

    /// Lowers a for loop expression to RIR.
    pub(crate) fn lower_for_expr(&mut self, for_loop: &For) -> Result<RirExpression> {
        // Extract range from iterable
        let (start, end, inclusive) = match &*for_loop.iterable {
            Expression::Range(range) => {
                let start = self.lower_expression(&range.start)?;
                let end = self.lower_expression(&range.end)?;
                (Box::new(start), Box::new(end), range.inclusive)
            }
            _ => {
                return Err(Error::Semantic(
                    "For loop iterable must be a range".to_string(),
                ));
            }
        };

        // Enter new scope for loop variable
        self.enter_scope();

        // Define loop variable (Int type for ranges, immutable)
        self.define_variable(for_loop.variable.clone(), TypeId::INT, false);

        // Enter loop context for label generation
        let label = self.enter_loop();

        let body = self.lower_block(&for_loop.body)?;

        // Exit loop context
        self.exit_loop();

        // Exit scope
        self.exit_scope();

        // Get result type from break with value or Unit if none
        let result_type = self.infer_loop_result_type(&body);

        Ok(RirExpression::For {
            variable: for_loop.variable.clone(),
            start,
            end,
            inclusive,
            body,
            label,
            result_type,
            span: for_loop.span,
        })
    }

    /// Lowers an infinite loop expression to RIR.
    pub(crate) fn lower_loop_expr(&mut self, loop_expr: &Loop) -> Result<RirExpression> {
        // Enter loop context for label generation
        let label = self.enter_loop();

        let body = self.lower_block(&loop_expr.body)?;

        // Exit loop context
        self.exit_loop();

        // Get result type from break with value or Unit if none
        let result_type = self.infer_loop_result_type(&body);

        Ok(RirExpression::Loop {
            body,
            label,
            result_type,
            span: loop_expr.span,
        })
    }

    /// Lowers a break statement to RIR.
    pub(crate) fn lower_break(&mut self, break_stmt: &Break) -> Result<RirStatement> {
        let depth = break_stmt.depth.unwrap_or(1) as usize;

        // Convert depth to label
        let label = self.get_loop_label(depth)?;

        let value = break_stmt
            .value
            .as_ref()
            .map(|v| self.lower_expression(v))
            .transpose()?
            .map(Box::new);

        Ok(RirStatement::Break {
            label,
            value,
            span: break_stmt.span,
        })
    }

    /// Lowers a continue statement to RIR.
    pub(crate) fn lower_continue(&mut self, continue_stmt: &Continue) -> Result<RirStatement> {
        let depth = continue_stmt.depth.unwrap_or(1) as usize;

        // Convert depth to label
        let label = self.get_loop_label(depth)?;

        Ok(RirStatement::Continue {
            label,
            span: continue_stmt.span,
        })
    }

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
