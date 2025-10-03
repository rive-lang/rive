//! Constant folding optimization pass.
//!
//! Evaluates constant expressions at compile time.
//! Example: `2 + 3 * 4` becomes `14`

use crate::{BinaryOp, RirExpression, RirFunction, RirModule, RirStatement, UnaryOp};

use super::OptimizationPass;

/// Constant folding optimization pass
pub struct ConstantFoldingPass;

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn run(&self, module: &mut RirModule) -> bool {
        let mut changed = false;

        for function in &mut module.functions {
            if fold_function(function) {
                changed = true;
            }
        }

        changed
    }
}

/// Folds constants in a function
fn fold_function(function: &mut RirFunction) -> bool {
    let mut changed = false;

    for statement in &mut function.body.statements {
        if fold_statement(statement) {
            changed = true;
        }
    }

    changed
}

/// Folds constants in a statement
fn fold_statement(statement: &mut RirStatement) -> bool {
    match statement {
        RirStatement::Let { value, .. } => fold_expression(value),
        RirStatement::Assign { value, .. } => fold_expression(value),
        RirStatement::AssignIndex { index, value, .. } => {
            let mut changed = fold_expression(index);
            if fold_expression(value) {
                changed = true;
            }
            changed
        }
        RirStatement::Return { value: Some(v), .. } => fold_expression(v),
        RirStatement::Expression { expr, .. } => fold_expression(expr),
        RirStatement::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            let mut changed = fold_expression(condition);
            for stmt in &mut then_block.statements {
                if fold_statement(stmt) {
                    changed = true;
                }
            }
            if let Some(else_blk) = else_block {
                for stmt in &mut else_blk.statements {
                    if fold_statement(stmt) {
                        changed = true;
                    }
                }
            }
            changed
        }
        RirStatement::While {
            condition, body, ..
        } => {
            let mut changed = fold_expression(condition);
            for stmt in &mut body.statements {
                if fold_statement(stmt) {
                    changed = true;
                }
            }
            changed
        }
        RirStatement::Block { block, .. } => {
            let mut changed = false;
            for stmt in &mut block.statements {
                if fold_statement(stmt) {
                    changed = true;
                }
            }
            changed
        }
        RirStatement::Return { value: None, .. } => false,

        // TODO: Phase 6 - Implement constant folding for control flow
        RirStatement::For { .. }
        | RirStatement::Loop { .. }
        | RirStatement::Break { .. }
        | RirStatement::Continue { .. }
        | RirStatement::Match { .. } => false,
    }
}

/// Folds constants in an expression (returns true if changed)
fn fold_expression(expr: &mut RirExpression) -> bool {
    match expr {
        RirExpression::Binary {
            op,
            left,
            right,
            result_type,
            span,
        } => {
            let mut changed = fold_expression(left);
            if fold_expression(right) {
                changed = true;
            }

            // Try to evaluate the binary operation
            if let Some(folded) = try_fold_binary(*op, left, right, *result_type, *span) {
                *expr = folded;
                return true;
            }

            changed
        }

        RirExpression::Unary {
            op,
            operand,
            result_type,
            span,
        } => {
            let changed = fold_expression(operand);

            // Try to evaluate the unary operation
            if let Some(folded) = try_fold_unary(*op, operand, *result_type, *span) {
                *expr = folded;
                return true;
            }

            changed
        }

        RirExpression::Call { arguments, .. } => {
            let mut changed = false;
            for arg in arguments {
                if fold_expression(arg) {
                    changed = true;
                }
            }
            changed
        }

        RirExpression::ArrayLiteral { elements, .. } => {
            let mut changed = false;
            for elem in elements {
                if fold_expression(elem) {
                    changed = true;
                }
            }
            changed
        }

        RirExpression::Index { array, index, .. } => {
            let mut changed = fold_expression(array);
            if fold_expression(index) {
                changed = true;
            }
            changed
        }

        // Literals and variables don't need folding
        _ => false,
    }
}

/// Tries to fold a binary operation if both operands are constants
fn try_fold_binary(
    op: BinaryOp,
    left: &RirExpression,
    right: &RirExpression,
    _result_type: rive_core::type_system::TypeId,
    span: rive_core::span::Span,
) -> Option<RirExpression> {
    use RirExpression::*;

    match (left, right) {
        // Integer arithmetic
        (IntLiteral { value: l, .. }, IntLiteral { value: r, .. }) => {
            let result = match op {
                BinaryOp::Add => l.checked_add(*r)?,
                BinaryOp::Subtract => l.checked_sub(*r)?,
                BinaryOp::Multiply => l.checked_mul(*r)?,
                BinaryOp::Divide if *r != 0 => l.checked_div(*r)?,
                BinaryOp::Modulo if *r != 0 => l.checked_rem(*r)?,
                BinaryOp::Equal => {
                    return Some(BoolLiteral {
                        value: l == r,
                        span,
                    });
                }
                BinaryOp::NotEqual => {
                    return Some(BoolLiteral {
                        value: l != r,
                        span,
                    });
                }
                BinaryOp::LessThan => return Some(BoolLiteral { value: l < r, span }),
                BinaryOp::LessEqual => {
                    return Some(BoolLiteral {
                        value: l <= r,
                        span,
                    });
                }
                BinaryOp::GreaterThan => return Some(BoolLiteral { value: l > r, span }),
                BinaryOp::GreaterEqual => {
                    return Some(BoolLiteral {
                        value: l >= r,
                        span,
                    });
                }
                _ => return None,
            };
            Some(IntLiteral {
                value: result,
                span,
            })
        }

        // Float arithmetic
        (FloatLiteral { value: l, .. }, FloatLiteral { value: r, .. }) => {
            let result = match op {
                BinaryOp::Add => l + r,
                BinaryOp::Subtract => l - r,
                BinaryOp::Multiply => l * r,
                BinaryOp::Divide => l / r,
                BinaryOp::Modulo => l % r,
                BinaryOp::Equal => {
                    return Some(BoolLiteral {
                        value: l == r,
                        span,
                    });
                }
                BinaryOp::NotEqual => {
                    return Some(BoolLiteral {
                        value: l != r,
                        span,
                    });
                }
                BinaryOp::LessThan => return Some(BoolLiteral { value: l < r, span }),
                BinaryOp::LessEqual => {
                    return Some(BoolLiteral {
                        value: l <= r,
                        span,
                    });
                }
                BinaryOp::GreaterThan => return Some(BoolLiteral { value: l > r, span }),
                BinaryOp::GreaterEqual => {
                    return Some(BoolLiteral {
                        value: l >= r,
                        span,
                    });
                }
                _ => return None,
            };
            Some(FloatLiteral {
                value: result,
                span,
            })
        }

        // Boolean logic
        (BoolLiteral { value: l, .. }, BoolLiteral { value: r, .. }) => {
            let result = match op {
                BinaryOp::And => *l && *r,
                BinaryOp::Or => *l || *r,
                BinaryOp::Equal => l == r,
                BinaryOp::NotEqual => l != r,
                _ => return None,
            };
            Some(BoolLiteral {
                value: result,
                span,
            })
        }

        // String comparison
        (StringLiteral { value: l, .. }, StringLiteral { value: r, .. }) => {
            let result = match op {
                BinaryOp::Equal => l == r,
                BinaryOp::NotEqual => l != r,
                _ => return None,
            };
            Some(BoolLiteral {
                value: result,
                span,
            })
        }

        _ => None,
    }
}

/// Tries to fold a unary operation if the operand is a constant
fn try_fold_unary(
    op: UnaryOp,
    operand: &RirExpression,
    _result_type: rive_core::type_system::TypeId,
    span: rive_core::span::Span,
) -> Option<RirExpression> {
    use RirExpression::*;

    match operand {
        IntLiteral { value, .. } => match op {
            UnaryOp::Negate => Some(IntLiteral {
                value: value.checked_neg()?,
                span,
            }),
            _ => None,
        },

        FloatLiteral { value, .. } => match op {
            UnaryOp::Negate => Some(FloatLiteral {
                value: -value,
                span,
            }),
            _ => None,
        },

        BoolLiteral { value, .. } => match op {
            UnaryOp::Not => Some(BoolLiteral {
                value: !value,
                span,
            }),
            _ => None,
        },

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_core::{
        span::{Location, Span},
        type_system::TypeId,
    };

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_fold_integer_add() {
        let span = dummy_span();
        let mut expr = RirExpression::Binary {
            op: BinaryOp::Add,
            left: Box::new(RirExpression::IntLiteral { value: 2, span }),
            right: Box::new(RirExpression::IntLiteral { value: 3, span }),
            result_type: TypeId::INT,
            span,
        };

        assert!(fold_expression(&mut expr));
        assert!(matches!(expr, RirExpression::IntLiteral { value: 5, .. }));
    }

    #[test]
    fn test_fold_nested_arithmetic() {
        let span = dummy_span();
        // 2 + (3 * 4) should become 14 in one pass
        let mut expr = RirExpression::Binary {
            op: BinaryOp::Add,
            left: Box::new(RirExpression::IntLiteral { value: 2, span }),
            right: Box::new(RirExpression::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(RirExpression::IntLiteral { value: 3, span }),
                right: Box::new(RirExpression::IntLiteral { value: 4, span }),
                result_type: TypeId::INT,
                span,
            }),
            result_type: TypeId::INT,
            span,
        };

        // Single pass should fold everything: 3*4=12, then 2+12=14
        assert!(fold_expression(&mut expr));
        assert!(matches!(expr, RirExpression::IntLiteral { value: 14, .. }));
    }

    #[test]
    fn test_fold_comparison() {
        let span = dummy_span();
        let mut expr = RirExpression::Binary {
            op: BinaryOp::LessThan,
            left: Box::new(RirExpression::IntLiteral { value: 2, span }),
            right: Box::new(RirExpression::IntLiteral { value: 3, span }),
            result_type: TypeId::BOOL,
            span,
        };

        assert!(fold_expression(&mut expr));
        assert!(matches!(
            expr,
            RirExpression::BoolLiteral { value: true, .. }
        ));
    }

    #[test]
    fn test_fold_boolean_logic() {
        let span = dummy_span();
        let mut expr = RirExpression::Binary {
            op: BinaryOp::And,
            left: Box::new(RirExpression::BoolLiteral { value: true, span }),
            right: Box::new(RirExpression::BoolLiteral { value: false, span }),
            result_type: TypeId::BOOL,
            span,
        };

        assert!(fold_expression(&mut expr));
        assert!(matches!(
            expr,
            RirExpression::BoolLiteral { value: false, .. }
        ));
    }

    #[test]
    fn test_fold_unary_negate() {
        let span = dummy_span();
        let mut expr = RirExpression::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(RirExpression::IntLiteral { value: 42, span }),
            result_type: TypeId::INT,
            span,
        };

        assert!(fold_expression(&mut expr));
        assert!(matches!(expr, RirExpression::IntLiteral { value: -42, .. }));
    }

    #[test]
    fn test_fold_unary_not() {
        let span = dummy_span();
        let mut expr = RirExpression::Unary {
            op: UnaryOp::Not,
            operand: Box::new(RirExpression::BoolLiteral { value: true, span }),
            result_type: TypeId::BOOL,
            span,
        };

        assert!(fold_expression(&mut expr));
        assert!(matches!(
            expr,
            RirExpression::BoolLiteral { value: false, .. }
        ));
    }

    #[test]
    fn test_no_fold_with_variables() {
        let span = dummy_span();
        let mut expr = RirExpression::Binary {
            op: BinaryOp::Add,
            left: Box::new(RirExpression::Variable {
                name: "x".to_string(),
                type_id: TypeId::INT,
                span,
            }),
            right: Box::new(RirExpression::IntLiteral { value: 3, span }),
            result_type: TypeId::INT,
            span,
        };

        assert!(!fold_expression(&mut expr));
    }
}
