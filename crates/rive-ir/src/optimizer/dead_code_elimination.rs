//! Dead code elimination optimization pass.
//!
//! Removes:
//! - Unused variable declarations
//! - Unreachable code after return statements

use std::collections::HashSet;

use crate::{RirExpression, RirFunction, RirModule, RirStatement};

use super::OptimizationPass;

/// Dead code elimination optimization pass
pub struct DeadCodeEliminationPass;

impl OptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &str {
        "DeadCodeElimination"
    }

    fn run(&self, module: &mut RirModule) -> bool {
        let mut changed = false;

        for function in &mut module.functions {
            if eliminate_dead_code(function) {
                changed = true;
            }
        }

        changed
    }
}

/// Eliminates dead code in a function
fn eliminate_dead_code(function: &mut RirFunction) -> bool {
    let mut changed = false;

    // Step 1: Remove unreachable code after returns
    if remove_unreachable_code(&mut function.body.statements) {
        changed = true;
    }

    // Step 2: Find all used variables
    let used_vars = find_used_variables(&function.body.statements);

    // Step 3: Remove unused variable declarations
    if remove_unused_variables(&mut function.body.statements, &used_vars) {
        changed = true;
    }

    changed
}

/// Removes statements after return statements (unreachable code)
fn remove_unreachable_code(statements: &mut Vec<RirStatement>) -> bool {
    let mut changed = false;
    let mut found_return = false;
    let mut keep_index = statements.len();

    for (i, stmt) in statements.iter().enumerate() {
        if found_return {
            keep_index = i;
            changed = true;
            break;
        }

        if matches!(stmt, RirStatement::Return { .. }) {
            found_return = true;
        }
    }

    if changed {
        statements.truncate(keep_index);
    }

    // Recursively process nested blocks
    for stmt in statements.iter_mut() {
        match stmt {
            RirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                if remove_unreachable_code(&mut then_block.statements) {
                    changed = true;
                }
                if let Some(else_blk) = else_block
                    && remove_unreachable_code(&mut else_blk.statements)
                {
                    changed = true;
                }
            }
            RirStatement::While { body, .. } => {
                if remove_unreachable_code(&mut body.statements) {
                    changed = true;
                }
            }
            RirStatement::Block { block, .. } => {
                if remove_unreachable_code(&mut block.statements) {
                    changed = true;
                }
            }
            _ => {}
        }
    }

    changed
}

/// Finds all variables that are used (read from)
fn find_used_variables(statements: &[RirStatement]) -> HashSet<String> {
    let mut used = HashSet::new();

    for stmt in statements {
        collect_used_in_statement(stmt, &mut used);
    }

    used
}

/// Collects used variables from a statement
fn collect_used_in_statement(statement: &RirStatement, used: &mut HashSet<String>) {
    match statement {
        RirStatement::Let { value, .. } => {
            collect_used_in_expression(value, used);
        }
        RirStatement::Assign { value, .. } => {
            collect_used_in_expression(value, used);
        }
        RirStatement::AssignIndex {
            array,
            index,
            value,
            ..
        } => {
            used.insert(array.clone());
            collect_used_in_expression(index, used);
            collect_used_in_expression(value, used);
        }
        RirStatement::Return { value: Some(v), .. } => {
            collect_used_in_expression(v, used);
        }
        RirStatement::Expression { expr, .. } => {
            collect_used_in_expression(expr, used);
        }
        RirStatement::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            collect_used_in_expression(condition, used);
            for stmt in &then_block.statements {
                collect_used_in_statement(stmt, used);
            }
            if let Some(else_blk) = else_block {
                for stmt in &else_blk.statements {
                    collect_used_in_statement(stmt, used);
                }
            }
        }
        RirStatement::While {
            condition, body, ..
        } => {
            collect_used_in_expression(condition, used);
            for stmt in &body.statements {
                collect_used_in_statement(stmt, used);
            }
        }
        RirStatement::Block { block, .. } => {
            for stmt in &block.statements {
                collect_used_in_statement(stmt, used);
            }
        }
        RirStatement::Return { value: None, .. } => {}
    }
}

/// Collects used variables from an expression
fn collect_used_in_expression(expr: &RirExpression, used: &mut HashSet<String>) {
    match expr {
        RirExpression::Variable { name, .. } => {
            used.insert(name.clone());
        }
        RirExpression::Binary { left, right, .. } => {
            collect_used_in_expression(left, used);
            collect_used_in_expression(right, used);
        }
        RirExpression::Unary { operand, .. } => {
            collect_used_in_expression(operand, used);
        }
        RirExpression::Call { arguments, .. } => {
            for arg in arguments {
                collect_used_in_expression(arg, used);
            }
        }
        RirExpression::ArrayLiteral { elements, .. } => {
            for elem in elements {
                collect_used_in_expression(elem, used);
            }
        }
        RirExpression::Index { array, index, .. } => {
            collect_used_in_expression(array, used);
            collect_used_in_expression(index, used);
        }
        // Literals don't use variables
        _ => {}
    }
}

/// Removes unused variable declarations
fn remove_unused_variables(
    statements: &mut Vec<RirStatement>,
    used_vars: &HashSet<String>,
) -> bool {
    let mut changed = false;
    let mut indices_to_remove = Vec::new();

    for (i, stmt) in statements.iter().enumerate() {
        if let RirStatement::Let { name, value, .. } = stmt {
            // Keep variables that are used OR have side effects
            if !used_vars.contains(name) && !has_side_effects(value) {
                indices_to_remove.push(i);
                changed = true;
            }
        }
    }

    // Remove in reverse order to maintain indices
    for &i in indices_to_remove.iter().rev() {
        statements.remove(i);
    }

    // Recursively process nested blocks
    for stmt in statements.iter_mut() {
        match stmt {
            RirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                if remove_unused_variables(&mut then_block.statements, used_vars) {
                    changed = true;
                }
                if let Some(else_blk) = else_block
                    && remove_unused_variables(&mut else_blk.statements, used_vars)
                {
                    changed = true;
                }
            }
            RirStatement::While { body, .. } => {
                if remove_unused_variables(&mut body.statements, used_vars) {
                    changed = true;
                }
            }
            RirStatement::Block { block, .. } => {
                if remove_unused_variables(&mut block.statements, used_vars) {
                    changed = true;
                }
            }
            _ => {}
        }
    }

    changed
}

/// Checks if an expression has side effects (function calls)
fn has_side_effects(expr: &RirExpression) -> bool {
    match expr {
        RirExpression::Call { .. } => true, // Function calls always have potential side effects
        RirExpression::Binary { left, right, .. } => {
            has_side_effects(left) || has_side_effects(right)
        }
        RirExpression::Unary { operand, .. } => has_side_effects(operand),
        RirExpression::ArrayLiteral { elements, .. } => elements.iter().any(has_side_effects),
        RirExpression::Index { array, index, .. } => {
            has_side_effects(array) || has_side_effects(index)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RirBlock, RirFunction};
    use rive_core::{
        span::{Location, Span},
        type_system::{MemoryStrategy, TypeId},
    };

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_remove_unused_variable() {
        let span = dummy_span();
        let mut function = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock {
                statements: vec![
                    RirStatement::Let {
                        name: "unused".to_string(),
                        type_id: TypeId::INT,
                        is_mutable: false,
                        value: Box::new(RirExpression::IntLiteral { value: 42, span }),
                        memory_strategy: MemoryStrategy::Copy,
                        span,
                    },
                    RirStatement::Return { value: None, span },
                ],
                final_expr: None,
                span,
            },
            span,
        );

        assert!(eliminate_dead_code(&mut function));
        assert_eq!(function.body.statements.len(), 1); // Only return remains
    }

    #[test]
    fn test_keep_used_variable() {
        let span = dummy_span();
        let mut function = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::INT,
            RirBlock {
                statements: vec![
                    RirStatement::Let {
                        name: "x".to_string(),
                        type_id: TypeId::INT,
                        is_mutable: false,
                        value: Box::new(RirExpression::IntLiteral { value: 42, span }),
                        memory_strategy: MemoryStrategy::Copy,
                        span,
                    },
                    RirStatement::Return {
                        value: Some(Box::new(RirExpression::Variable {
                            name: "x".to_string(),
                            type_id: TypeId::INT,
                            span,
                        })),
                        span,
                    },
                ],
                final_expr: None,
                span,
            },
            span,
        );

        assert!(!eliminate_dead_code(&mut function));
        assert_eq!(function.body.statements.len(), 2); // Both statements remain
    }

    #[test]
    fn test_remove_unreachable_after_return() {
        let span = dummy_span();
        let mut function = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock {
                statements: vec![
                    RirStatement::Return { value: None, span },
                    RirStatement::Let {
                        name: "unreachable".to_string(),
                        type_id: TypeId::INT,
                        is_mutable: false,
                        value: Box::new(RirExpression::IntLiteral { value: 42, span }),
                        memory_strategy: MemoryStrategy::Copy,
                        span,
                    },
                ],
                final_expr: None,
                span,
            },
            span,
        );

        assert!(eliminate_dead_code(&mut function));
        assert_eq!(function.body.statements.len(), 1); // Only return remains
    }

    #[test]
    fn test_keep_function_call_side_effects() {
        let span = dummy_span();
        let mut function = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock {
                statements: vec![
                    RirStatement::Let {
                        name: "unused".to_string(),
                        type_id: TypeId::INT,
                        is_mutable: false,
                        value: Box::new(RirExpression::Call {
                            function: "foo".to_string(),
                            arguments: vec![],
                            return_type: TypeId::INT,
                            span,
                        }),
                        memory_strategy: MemoryStrategy::Copy,
                        span,
                    },
                    RirStatement::Return { value: None, span },
                ],
                final_expr: None,
                span,
            },
            span,
        );

        // Should NOT remove the let because the function call has side effects
        assert!(!eliminate_dead_code(&mut function));
        assert_eq!(function.body.statements.len(), 2);
    }
}
