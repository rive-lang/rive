//! Control flow type checking for Rive.
//!
//! This module implements type checking for control flow constructs:
//! - If expressions/statements
//! - Loop constructs (while, for, loop)
//! - Match expressions
//! - Break and continue statements
//! - Range expressions

use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::control_flow::{Break, Continue, For, If, Loop, Match, Pattern, Range, While};

use crate::type_checker::TypeChecker;

/// Context for loop type checking
#[derive(Debug, Clone)]
pub(crate) struct LoopContext {
    /// Type that this loop returns (if break has value)
    pub break_type: Option<TypeId>,
    /// Whether break statement was seen
    pub has_break: bool,
}

impl LoopContext {
    /// Creates a new loop context
    pub fn new() -> Self {
        Self {
            break_type: None,
            has_break: false,
        }
    }
}

impl TypeChecker {
    /// Checks an if expression/statement.
    ///
    /// Rules:
    /// - Condition must be Bool
    /// - All branches must have the same type
    /// - If used as expression (result needed), must have else
    pub(crate) fn check_if(&mut self, if_expr: &If, needs_value: bool) -> Result<TypeId> {
        let condition_type = self.check_expression(&if_expr.condition)?;
        let bool_type = self.symbols.type_registry().get_bool();

        if condition_type != bool_type {
            let registry = self.symbols.type_registry();
            let cond_str = registry.get_type_name(condition_type);
            return Err(Error::SemanticWithSpan(
                format!("If condition must be Bool, found '{cond_str}'"),
                if_expr.span,
            ));
        }

        // Check then block
        let then_type = self.check_block_with_value(&if_expr.then_block)?;

        // Check else-if branches
        for else_if in &if_expr.else_if_branches {
            let else_if_cond_type = self.check_expression(&else_if.condition)?;
            if else_if_cond_type != bool_type {
                let registry = self.symbols.type_registry();
                let cond_str = registry.get_type_name(else_if_cond_type);
                return Err(Error::SemanticWithSpan(
                    format!("Else-if condition must be Bool, found '{cond_str}'"),
                    else_if.span,
                ));
            }

            let else_if_type = self.check_block_with_value(&else_if.block)?;
            if else_if_type != then_type {
                let registry = self.symbols.type_registry();
                let then_str = registry.get_type_name(then_type);
                let else_if_str = registry.get_type_name(else_if_type);
                return Err(Error::SemanticWithSpan(
                    format!(
                        "All if branches must have same type: expected '{then_str}', found '{else_if_str}'"
                    ),
                    else_if.span,
                ));
            }
        }

        // Check else block
        if let Some(else_block) = &if_expr.else_block {
            let else_type = self.check_block_with_value(else_block)?;
            if else_type != then_type {
                let registry = self.symbols.type_registry();
                let then_str = registry.get_type_name(then_type);
                let else_str = registry.get_type_name(else_type);
                return Err(Error::SemanticWithSpan(
                    format!(
                        "All if branches must have same type: expected '{then_str}', found '{else_str}'"
                    ),
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
            Ok(self.symbols.type_registry().get_unit())
        }
    }

    /// Checks a while loop.
    pub(crate) fn check_while(&mut self, while_loop: &While) -> Result<TypeId> {
        let condition_type = self.check_expression(&while_loop.condition)?;
        let bool_type = self.symbols.type_registry().get_bool();

        if condition_type != bool_type {
            let registry = self.symbols.type_registry();
            let cond_str = registry.get_type_name(condition_type);
            return Err(Error::SemanticWithSpan(
                format!("While condition must be Bool, found '{cond_str}'"),
                while_loop.span,
            ));
        }

        // Enter loop context
        self.loop_stack.push(LoopContext::new());

        // Check body
        self.check_block(&while_loop.body)?;

        // Exit loop context and determine return type
        let loop_ctx = self.loop_stack.pop().unwrap();
        Ok(loop_ctx
            .break_type
            .unwrap_or_else(|| self.symbols.type_registry().get_unit()))
    }

    /// Checks a for loop.
    pub(crate) fn check_for(&mut self, for_loop: &For) -> Result<TypeId> {
        // Check iterable (currently only ranges)
        let _iterable_type = self.check_expression(&for_loop.iterable)?;

        // For Phase 1, we only support ranges
        // Future: Check for iterable trait/interface

        // Enter new scope for loop variable
        self.symbols.enter_scope();

        // Define loop variable (currently Int for ranges)
        let int_type = self.symbols.type_registry().get_int();
        let symbol = crate::symbol_table::Symbol::new(
            for_loop.variable.clone(),
            int_type,
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

        Ok(loop_ctx
            .break_type
            .unwrap_or_else(|| self.symbols.type_registry().get_unit()))
    }

    /// Checks an infinite loop.
    pub(crate) fn check_loop(&mut self, loop_expr: &Loop) -> Result<TypeId> {
        // Enter loop context
        self.loop_stack.push(LoopContext::new());

        // Check body
        self.check_block(&loop_expr.body)?;

        // Exit loop context
        let loop_ctx = self.loop_stack.pop().unwrap();

        // Infinite loop must have break to be useful
        if !loop_ctx.has_break {
            // Warning could be issued here, but not an error
        }

        Ok(loop_ctx
            .break_type
            .unwrap_or_else(|| self.symbols.type_registry().get_unit()))
    }

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
        let is_exhaustive = has_wildcard
            || (scrutinee_type == self.symbols.type_registry().get_bool() && has_true && has_false);

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
                    let registry = self.symbols.type_registry();
                    let first_str = registry.get_type_name(first_type);
                    let arm_str = registry.get_type_name(arm_type);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "All match arms must have same type: expected '{first_str}', found '{arm_str}' in arm {}",
                            i + 2
                        ),
                        match_expr.arms[i + 1].span,
                    ));
                }
            }
            Ok(first_type)
        } else {
            // When used as a statement, return Unit
            Ok(self.symbols.type_registry().get_unit())
        }
    }

    /// Checks a pattern against expected type.
    fn check_pattern(&mut self, pattern: &Pattern, expected_type: TypeId) -> Result<()> {
        let pattern_type = match pattern {
            Pattern::Integer { .. } => self.symbols.type_registry().get_int(),
            Pattern::Float { .. } => self.symbols.type_registry().get_float(),
            Pattern::String { .. } => self.symbols.type_registry().get_text(),
            Pattern::Boolean { .. } => self.symbols.type_registry().get_bool(),
            Pattern::Null { .. } => {
                return Err(Error::SemanticWithSpan(
                    "Null patterns not yet supported".to_string(),
                    pattern.span(),
                ));
            }
            Pattern::Wildcard { .. } => return Ok(()), // Wildcard matches any type
            Pattern::Range { start, end, .. } => {
                // Check that start and end expressions are compatible with expected type
                let start_type = self.check_expression(start)?;
                let end_type = self.check_expression(end)?;

                if start_type != expected_type {
                    let registry = self.symbols.type_registry();
                    let expected_str = registry.get_type_name(expected_type);
                    let start_str = registry.get_type_name(start_type);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Range start type mismatch: expected '{expected_str}', found '{start_str}'"
                        ),
                        start.span(),
                    ));
                }

                if end_type != expected_type {
                    let registry = self.symbols.type_registry();
                    let expected_str = registry.get_type_name(expected_type);
                    let end_str = registry.get_type_name(end_type);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Range end type mismatch: expected '{expected_str}', found '{end_str}'"
                        ),
                        end.span(),
                    ));
                }

                return Ok(());
            }
        };

        if pattern_type != expected_type {
            let registry = self.symbols.type_registry();
            let expected_str = registry.get_type_name(expected_type);
            let pattern_str = registry.get_type_name(pattern_type);
            return Err(Error::SemanticWithSpan(
                format!("Pattern type mismatch: expected '{expected_str}', found '{pattern_str}'"),
                pattern.span(),
            ));
        }

        Ok(())
    }

    /// Checks a range expression.
    pub(crate) fn check_range(&mut self, range: &Range) -> Result<TypeId> {
        let start_type = self.check_expression(&range.start)?;
        let end_type = self.check_expression(&range.end)?;

        let int_type = self.symbols.type_registry().get_int();

        // For Phase 1, only support Int ranges
        if start_type != int_type {
            let registry = self.symbols.type_registry();
            let start_str = registry.get_type_name(start_type);
            return Err(Error::SemanticWithSpan(
                format!("Range start must be Int, found '{start_str}'"),
                range.span,
            ));
        }

        if end_type != int_type {
            let registry = self.symbols.type_registry();
            let end_str = registry.get_type_name(end_type);
            return Err(Error::SemanticWithSpan(
                format!("Range end must be Int, found '{end_str}'"),
                range.span,
            ));
        }

        // Return a range type (for now, we'll use Int to represent it)
        // Future: Create a dedicated Range<T> type
        Ok(int_type)
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
        let actual_depth = break_stmt.depth.unwrap_or(1);
        if actual_depth == 0 {
            return Err(Error::SemanticWithSpan(
                "Break depth must be at least 1".to_string(),
                break_stmt.span,
            ));
        }
        if actual_depth as usize > self.loop_stack.len() {
            return Err(Error::SemanticWithSpan(
                format!(
                    "Break depth {} exceeds loop nesting level {}",
                    actual_depth,
                    self.loop_stack.len()
                ),
                break_stmt.span,
            ));
        }

        // Get target loop
        let target_loop_idx = self.loop_stack.len() - actual_depth as usize;
        self.loop_stack[target_loop_idx].has_break = true;

        // Check value type consistency
        if let Some(val_expr) = &break_stmt.value {
            let val_type = self.check_expression(val_expr)?;

            let existing_type = self.loop_stack[target_loop_idx].break_type;

            if let Some(existing) = existing_type {
                if val_type != existing {
                    let registry = self.symbols.type_registry();
                    let existing_str = registry.get_type_name(existing);
                    let val_str = registry.get_type_name(val_type);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "All break values in a loop must have the same type: expected '{existing_str}', found '{val_str}'"
                        ),
                        break_stmt.span,
                    ));
                }
            } else {
                // First break with value in this loop
                self.loop_stack[target_loop_idx].break_type = Some(val_type);
            }

            Ok(val_type)
        } else {
            // Break without value
            let existing_type = self.loop_stack[target_loop_idx].break_type;
            let unit_type = self.symbols.type_registry().get_unit();

            if let Some(existing) = existing_type {
                if existing != unit_type {
                    let registry = self.symbols.type_registry();
                    let existing_str = registry.get_type_name(existing);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Cannot mix 'break' and 'break with value': this loop expects breaks with type '{existing_str}'"
                        ),
                        break_stmt.span,
                    ));
                }
            } else {
                // First break in this loop, set to Unit
                self.loop_stack[target_loop_idx].break_type = Some(unit_type);
            }

            Ok(unit_type)
        }
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
        let actual_depth = continue_stmt.depth.unwrap_or(1);
        if actual_depth == 0 {
            return Err(Error::SemanticWithSpan(
                "Continue depth must be at least 1".to_string(),
                continue_stmt.span,
            ));
        }
        if actual_depth as usize > self.loop_stack.len() {
            return Err(Error::SemanticWithSpan(
                format!(
                    "Continue depth {} exceeds loop nesting level {}",
                    actual_depth,
                    self.loop_stack.len()
                ),
                continue_stmt.span,
            ));
        }

        Ok(self.symbols.type_registry().get_unit())
    }

    /// Checks a block and returns its type (considering implicit return).
    pub(crate) fn check_block_with_value(&mut self, block: &rive_parser::Block) -> Result<TypeId> {
        if block.statements.is_empty() {
            return Ok(self.symbols.type_registry().get_unit());
        }

        let num_stmts = block.statements.len();

        // Check all but last
        for stmt in &block.statements[..num_stmts - 1] {
            self.check_statement(stmt)?;
        }

        // Check last statement
        let last_stmt = &block.statements[num_stmts - 1];

        match last_stmt {
            rive_parser::Statement::Expression { expression, .. } => {
                // Last expression is implicit return
                self.check_expression(expression)
            }
            _ => {
                // Last statement is not expression, block returns Unit
                self.check_statement(last_stmt)?;
                Ok(self.symbols.type_registry().get_unit())
            }
        }
    }
}
