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

    /// Checks a while loop.
    pub(crate) fn check_while(&mut self, while_loop: &While) -> Result<TypeId> {
        self.check_bool_condition(&while_loop.condition, "While", while_loop.span)?;

        // Enter loop context
        self.loop_stack.push(LoopContext::new());

        // Check body
        self.check_block(&while_loop.body)?;

        // Exit loop context and determine return type
        let loop_ctx = self.loop_stack.pop().unwrap();
        Ok(loop_ctx.break_type.unwrap_or(TypeId::UNIT))
    }

    /// Checks a for loop.
    pub(crate) fn check_for(&mut self, for_loop: &For) -> Result<TypeId> {
        // Check iterable (currently only ranges)
        let _iterable_type = self.check_expression(&for_loop.iterable)?;

        // Enter new scope for loop variable
        self.symbols.enter_scope();

        // Define loop variable (currently Int for ranges)
        let symbol = crate::symbol_table::Symbol::new(
            for_loop.variable.clone(),
            TypeId::INT,
            false, // Loop variables are immutable
        );
        self.symbols.define(symbol)?;

        // Enter loop context
        self.loop_stack.push(LoopContext::new());

        // Check body
        self.check_block(&for_loop.body)?;

        // Exit loop context
        let loop_ctx = self.loop_stack.pop().unwrap();

        // Exit scope
        self.symbols.exit_scope();

        Ok(loop_ctx.break_type.unwrap_or(TypeId::UNIT))
    }

    /// Checks an infinite loop.
    pub(crate) fn check_loop(&mut self, loop_expr: &Loop) -> Result<TypeId> {
        // Enter loop context
        self.loop_stack.push(LoopContext::new());

        // Check body
        self.check_block(&loop_expr.body)?;

        // Exit loop context
        let loop_ctx = self.loop_stack.pop().unwrap();

        Ok(loop_ctx.break_type.unwrap_or(TypeId::UNIT))
    }

    /// Checks a range expression.
    pub(crate) fn check_range(&mut self, range: &Range) -> Result<TypeId> {
        let start_type = self.check_expression(&range.start)?;
        let end_type = self.check_expression(&range.end)?;

        // For Phase 1, only support Int ranges
        if start_type != TypeId::INT {
            let registry = self.symbols.type_registry();
            let start_str = registry.get_type_name(start_type);
            return Err(Error::SemanticWithSpan(
                format!("Range start must be Int, found '{start_str}'"),
                range.span,
            ));
        }

        if end_type != TypeId::INT {
            let registry = self.symbols.type_registry();
            let end_str = registry.get_type_name(end_type);
            return Err(Error::SemanticWithSpan(
                format!("Range end must be Int, found '{end_str}'"),
                range.span,
            ));
        }

        // Return Int type to represent range
        Ok(TypeId::INT)
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
