//! Control flow lowering from AST to RIR.

use crate::lowering::core::AstLowering;
use crate::{RirBlock, RirExpression, RirStatement};
use rive_core::{Error, Result, TypeId};
use rive_parser::Expression;
use rive_parser::control_flow::{Break, Continue, For, If, Loop, While};

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

        // Enter loop context with optional user label
        let label = self.enter_loop(while_loop.label.clone());

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

        // Enter loop context with optional user label
        let label = self.enter_loop(for_loop.label.clone());

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
        // Enter loop context with optional user label
        let label = self.enter_loop(loop_expr.label.clone());

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
        // Resolve user label or use innermost loop
        let label = self.resolve_loop_label(break_stmt.label.clone())?;

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
        // Resolve user label or use innermost loop
        let label = self.resolve_loop_label(continue_stmt.label.clone())?;

        Ok(RirStatement::Continue {
            label,
            span: continue_stmt.span,
        })
    }
}
