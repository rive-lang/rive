//! Expression lowering.

use crate::RirExpression;
use crate::lowering::core::AstLowering;
use rive_core::{Error, Result};
use rive_parser::Expression as AstExpression;

impl AstLowering {
    /// Lowers an expression.
    pub(crate) fn lower_expression(&mut self, expr: &AstExpression) -> Result<RirExpression> {
        match expr {
            AstExpression::Integer { value, span } => Ok(RirExpression::IntLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::Float { value, span } => Ok(RirExpression::FloatLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::String { value, span } => Ok(RirExpression::StringLiteral {
                value: value.clone(),
                span: *span,
            }),

            AstExpression::Boolean { value, span } => Ok(RirExpression::BoolLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::Null { span } => Ok(RirExpression::Unit { span: *span }),

            AstExpression::Variable { name, span } => {
                // Look up variable type from symbol table
                let type_id = self
                    .lookup_variable(name)
                    .map(|info| info.type_id)
                    .ok_or_else(|| Error::Semantic(format!("Undefined variable '{name}'")))?;

                Ok(RirExpression::Variable {
                    name: name.clone(),
                    type_id,
                    span: *span,
                })
            }

            AstExpression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_expr = self.lower_expression(left)?;
                let right_expr = self.lower_expression(right)?;
                let op = self.lower_binary_op(operator);
                let result_type = self.infer_binary_result_type(&left_expr, &right_expr, op);

                Ok(RirExpression::Binary {
                    op,
                    left: Box::new(left_expr),
                    right: Box::new(right_expr),
                    result_type,
                    span: *span,
                })
            }

            AstExpression::Unary {
                operator,
                operand,
                span,
            } => {
                let operand_expr = self.lower_expression(operand)?;
                let op = self.lower_unary_op(operator);
                let result_type = operand_expr.type_id();

                Ok(RirExpression::Unary {
                    op,
                    operand: Box::new(operand_expr),
                    result_type,
                    span: *span,
                })
            }

            AstExpression::Call {
                callee,
                arguments,
                span,
            } => {
                let args = arguments
                    .iter()
                    .map(|arg| self.lower_expression(arg))
                    .collect::<Result<Vec<_>>>()?;

                // Look up function return type from function signatures
                // Special case for built-in print function
                let return_type = if callee == "print" {
                    rive_core::type_system::TypeId::UNIT
                } else {
                    self.lookup_function(callee)
                        .map(|(_, return_type)| *return_type)
                        .ok_or_else(|| Error::Semantic(format!("Undefined function '{callee}'")))?
                };

                Ok(RirExpression::Call {
                    function: callee.clone(),
                    arguments: args,
                    return_type,
                    span: *span,
                })
            }

            AstExpression::Array { elements, span } => {
                let rir_elements = elements
                    .iter()
                    .map(|e| self.lower_expression(e))
                    .collect::<Result<Vec<_>>>()?;

                let element_type = if let Some(first) = rir_elements.first() {
                    first.type_id()
                } else {
                    rive_core::type_system::TypeId::INT // Default for empty arrays
                };

                Ok(RirExpression::ArrayLiteral {
                    elements: rir_elements,
                    element_type,
                    span: *span,
                })
            }

            AstExpression::If(if_expr) => self.lower_if_expr(if_expr),
            AstExpression::While(while_loop) => self.lower_while_expr(while_loop),
            AstExpression::For(for_loop) => self.lower_for_expr(for_loop),
            AstExpression::Loop(loop_expr) => self.lower_loop_expr(loop_expr),
            AstExpression::Match(match_expr) => self.lower_match_expr(match_expr),
            AstExpression::Range(_) => Err(Error::Semantic(
                "Range expressions can only be used in for loops".to_string(),
            )),
            AstExpression::Block(block) => self.lower_block_expr(block),
        }
    }

    /// Lowers a block expression to RIR.
    pub(crate) fn lower_block_expr(&mut self, block: &rive_parser::Block) -> Result<RirExpression> {
        let rir_block = self.lower_block(block)?;

        // Check if the block has a final expression
        let (result, result_type) = if let Some(ref final_expr) = rir_block.final_expr {
            (rir_block.final_expr.clone(), final_expr.type_id())
        } else {
            (None, rive_core::type_system::TypeId::UNIT)
        };

        Ok(RirExpression::Block {
            block: rir_block,
            result,
            result_type,
            span: block.span,
        })
    }
}
