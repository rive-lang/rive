//! Control flow type checking (if, while, for, loop, break, continue, range).

use crate::checker::{core::TypeChecker, loops::LoopContext};
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::control_flow::{Break, Continue, For, If, Loop, Range, While};

impl TypeChecker {
    /// Checks an if expression/statement.
    pub(crate) fn check_if(&mut self, if_expr: &If, needs_value: bool) -> Result<TypeId> {
        self.check_bool_condition(&if_expr.condition, "If", if_expr.span)?;

        // Check then block
        let then_type = self.check_block_with_value(&if_expr.then_block)?;

        // Check else-if branches
        for else_if in &if_expr.else_if_branches {
            self.check_bool_condition(&else_if.condition, "Else-if", else_if.span)?;

            let else_if_type = self.check_block_with_value(&else_if.block)?;
            if else_if_type != then_type {
                return Err(self.type_mismatch_error(
                    "All if branches must have same type",
                    then_type,
                    else_if_type,
                    else_if.span,
                ));
            }
        }

        // Check else block
        if let Some(else_block) = &if_expr.else_block {
            let else_type = self.check_block_with_value(else_block)?;
            if else_type != then_type {
                return Err(self.type_mismatch_error(
                    "All if branches must have same type",
                    then_type,
                    else_type,
                    if_expr.span,
                ));
            }
            Ok(then_type)
        } else {
            // No else block
            if needs_value {
                return Err(Error::SemanticWithSpan(
                    "If expression must have else branch when used as expression".to_string(),
                    if_expr.span,
                ));
            }
            // Statement context, return Unit
            Ok(TypeId::UNIT)
        }
    }

    /// Checks a break statement.
    pub(crate) fn check_break(&mut self, break_stmt: &Break) -> Result<TypeId> {
        // Validate we're in a loop
        if self.loop_stack.is_empty() {
            return Err(Error::SemanticWithSpan(
                "Break can only be used inside a loop".to_string(),
                break_stmt.span,
            ));
        }

        // Validate depth
        let actual_depth = self.validate_loop_depth(break_stmt.depth, break_stmt.span)?;

        // Get target loop
        let target_loop_idx = self.loop_stack.len() - actual_depth;
        self.loop_stack[target_loop_idx].has_break = true;

        // Check value type consistency
        if let Some(val_expr) = &break_stmt.value {
            let val_type = self.check_expression(val_expr)?;
            self.validate_break_type(target_loop_idx, Some(val_type), break_stmt.span)?;
            Ok(val_type)
        } else {
            self.validate_break_type(target_loop_idx, None, break_stmt.span)?;
            Ok(TypeId::UNIT)
        }
    }

    /// Validates break value type consistency.
    fn validate_break_type(
        &mut self,
        target_loop_idx: usize,
        val_type: Option<TypeId>,
        span: rive_core::Span,
    ) -> Result<()> {
        let val_type = val_type.unwrap_or(TypeId::UNIT);
        let existing_type = self.loop_stack[target_loop_idx].break_type;

        if let Some(existing) = existing_type {
            if val_type != existing {
                return Err(self.type_mismatch_error(
                    "All break values in a loop must have the same type",
                    existing,
                    val_type,
                    span,
                ));
            }
        } else {
            // First break in this loop
            self.loop_stack[target_loop_idx].break_type = Some(val_type);
        }

        Ok(())
    }

    /// Checks a continue statement.
    pub(crate) fn check_continue(&mut self, continue_stmt: &Continue) -> Result<TypeId> {
        // Validate we're in a loop
        if self.loop_stack.is_empty() {
            return Err(Error::SemanticWithSpan(
                "Continue can only be used inside a loop".to_string(),
                continue_stmt.span,
            ));
        }

        // Validate depth
        self.validate_loop_depth(continue_stmt.depth, continue_stmt.span)?;

        Ok(TypeId::UNIT)
    }
}
