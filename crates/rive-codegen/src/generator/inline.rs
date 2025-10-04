//! Inline optimization heuristics.

use rive_ir::{RirBlock, RirExpression, RirFunction, RirStatement};

/// Determines if a function should be inlined based on heuristics.
///
/// Inline heuristics:
/// - Small functions (≤ 5 statements)
/// - Simple expressions (no complex control flow)
/// - No recursive calls
/// - Not the main function (main should not be inlined)
pub fn should_inline_function(function: &RirFunction) -> bool {
    if function.name == "main" {
        return false;
    }

    if count_statements(&function.body) > 5 {
        return false;
    }

    if has_complex_control_flow(&function.body) {
        return false;
    }

    if has_recursive_calls(function) {
        return false;
    }

    true
}

/// Counts the number of statements in a block.
fn count_statements(block: &RirBlock) -> usize {
    let mut count = block.statements.len();

    for stmt in &block.statements {
        match stmt {
            RirStatement::Block { block, .. } => count += count_statements(block),
            RirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                count += count_statements(then_block);
                if let Some(else_block) = else_block {
                    count += count_statements(else_block);
                }
            }
            RirStatement::While { body, .. }
            | RirStatement::For { body, .. }
            | RirStatement::Loop { body, .. } => count += count_statements(body),
            RirStatement::Match { arms, .. } => {
                for (_, arm_body) in arms {
                    count += count_statements(arm_body);
                }
            }
            _ => {}
        }
    }

    count
}

/// Checks if a block has complex control flow patterns.
fn has_complex_control_flow(block: &RirBlock) -> bool {
    for stmt in &block.statements {
        match stmt {
            RirStatement::Block { block, .. } => {
                if has_complex_control_flow(block) {
                    return true;
                }
            }
            RirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                if has_complex_control_flow(then_block) {
                    return true;
                }
                if let Some(else_block) = else_block
                    && has_complex_control_flow(else_block)
                {
                    return true;
                }
            }
            RirStatement::While { .. } | RirStatement::For { .. } | RirStatement::Loop { .. } => {
                return true;
            }
            RirStatement::Match { arms, .. } => {
                for (_, arm_body) in arms {
                    if has_complex_control_flow(arm_body) {
                        return true;
                    }
                }
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Checks if a function contains recursive calls.
fn has_recursive_calls(function: &RirFunction) -> bool {
    check_recursive_calls_in_block(&function.body, &function.name)
}

/// Recursively checks for recursive calls in a block.
fn check_recursive_calls_in_block(block: &RirBlock, function_name: &str) -> bool {
    for stmt in &block.statements {
        match stmt {
            RirStatement::Expression { expr, .. } => {
                if check_recursive_calls_in_expr(expr, function_name) {
                    return true;
                }
            }
            RirStatement::Let { value, .. } | RirStatement::Assign { value, .. } => {
                if check_recursive_calls_in_expr(value, function_name) {
                    return true;
                }
            }
            RirStatement::Return { value, .. } => {
                if let Some(value) = value
                    && check_recursive_calls_in_expr(value, function_name)
                {
                    return true;
                }
            }
            RirStatement::Block { block, .. } => {
                if check_recursive_calls_in_block(block, function_name) {
                    return true;
                }
            }
            RirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                if check_recursive_calls_in_block(then_block, function_name) {
                    return true;
                }
                if let Some(else_block) = else_block
                    && check_recursive_calls_in_block(else_block, function_name)
                {
                    return true;
                }
            }
            RirStatement::While { body, .. }
            | RirStatement::For { body, .. }
            | RirStatement::Loop { body, .. } => {
                if check_recursive_calls_in_block(body, function_name) {
                    return true;
                }
            }
            RirStatement::Match { arms, .. } => {
                for (_, arm_body) in arms {
                    if check_recursive_calls_in_block(arm_body, function_name) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Checks for recursive calls in an expression.
fn check_recursive_calls_in_expr(expr: &RirExpression, function_name: &str) -> bool {
    match expr {
        RirExpression::Call { function, .. } => function == function_name,
        RirExpression::Binary { left, right, .. } => {
            check_recursive_calls_in_expr(left, function_name)
                || check_recursive_calls_in_expr(right, function_name)
        }
        RirExpression::Unary { operand, .. } => {
            check_recursive_calls_in_expr(operand, function_name)
        }
        RirExpression::ArrayLiteral { elements, .. } => elements
            .iter()
            .any(|elem| check_recursive_calls_in_expr(elem, function_name)),
        _ => false,
    }
}
