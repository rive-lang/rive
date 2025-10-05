//! Control flow type checking (if, while, for, loop, break, continue, range).

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::control_flow::{Break, Continue, For, If, Loop, While};

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

        // Find target loop by label
        let target_loop_idx = self.find_loop_by_label(&break_stmt.label, break_stmt.span)?;
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

        // Find target loop by label
        self.find_loop_by_label(&continue_stmt.label, continue_stmt.span)?;

        Ok(TypeId::UNIT)
    }

    /// Finds a loop context by label (or returns innermost if no label).
    fn find_loop_by_label(&self, label: &Option<String>, span: rive_core::Span) -> Result<usize> {
        if let Some(label_name) = label {
            // Find loop with matching label
            for (idx, ctx) in self.loop_stack.iter().enumerate().rev() {
                if ctx.label.as_ref() == Some(label_name) {
                    return Ok(idx);
                }
            }
            // Label not found
            Err(Error::SemanticWithSpan(
                format!("Label '{label_name}' not found"),
                span,
            ))
        } else {
            // No label, use innermost loop
            Ok(self.loop_stack.len() - 1)
        }
    }

    /// Checks a while loop expression.
    /// Returns Optional<T> where T is the break value type, or Optional<Unit> if no break with value.
    pub(crate) fn check_while_expr(&mut self, while_loop: &While) -> Result<TypeId> {
        // Check condition
        self.check_bool_condition(&while_loop.condition, "While", while_loop.span)?;

        // Enter loop context
        let loop_ctx = crate::checker::loops::LoopContext::new(while_loop.label.clone());
        self.loop_stack.push(loop_ctx);

        // Check body statements
        for stmt in &while_loop.body.statements {
            self.check_statement(stmt)?;
        }

        // Exit loop context and get result type
        let loop_ctx = self.loop_stack.pop().unwrap();
        let result_type = if let Some(break_type) = loop_ctx.break_type {
            // Has break with value: return Optional<T>
            self.get_or_create_nullable(break_type)
        } else {
            // No break with value: return Optional<Unit>
            self.get_or_create_nullable(TypeId::UNIT)
        };

        Ok(result_type)
    }

    /// Checks a for loop expression.
    /// Returns Optional<T> where T is the break value type, or Optional<Unit> if no break with value.
    pub(crate) fn check_for_expr(&mut self, for_loop: &For) -> Result<TypeId> {
        // Check iterable (should be a range)
        self.check_expression(&for_loop.iterable)?;

        // Enter new scope for loop variable
        self.symbols.enter_scope();

        // Define loop variable (Int type for ranges, immutable)
        let symbol =
            crate::symbol_table::Symbol::new(for_loop.variable.clone(), TypeId::INT, false);
        self.symbols.define(symbol)?;

        // Enter loop context
        let loop_ctx = crate::checker::loops::LoopContext::new(for_loop.label.clone());
        self.loop_stack.push(loop_ctx);

        // Check body statements
        for stmt in &for_loop.body.statements {
            self.check_statement(stmt)?;
        }

        // Exit loop context and get result type
        let loop_ctx = self.loop_stack.pop().unwrap();
        let result_type = if let Some(break_type) = loop_ctx.break_type {
            // Has break with value: return Optional<T>
            self.get_or_create_nullable(break_type)
        } else {
            // No break with value: return Optional<Unit>
            self.get_or_create_nullable(TypeId::UNIT)
        };

        // Exit scope
        self.symbols.exit_scope();

        Ok(result_type)
    }

    /// Checks an infinite loop expression.
    /// Returns Optional<T> where T is the break value type, or Optional<Unit> if no break with value.
    pub(crate) fn check_loop_expr(&mut self, loop_expr: &Loop) -> Result<TypeId> {
        // Enter loop context
        let loop_ctx = crate::checker::loops::LoopContext::new(loop_expr.label.clone());
        self.loop_stack.push(loop_ctx);

        // Check body statements
        for stmt in &loop_expr.body.statements {
            self.check_statement(stmt)?;
        }

        // Exit loop context and get result type
        let loop_ctx = self.loop_stack.pop().unwrap();
        let result_type = if let Some(break_type) = loop_ctx.break_type {
            // Has break with value: return Optional<T>
            self.get_or_create_nullable(break_type)
        } else {
            // No break with value: return Optional<Unit>
            self.get_or_create_nullable(TypeId::UNIT)
        };

        Ok(result_type)
    }
}
