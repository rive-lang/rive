//! Statement lowering.

use crate::RirStatement;
use crate::lowering::core::AstLowering;
use rive_core::Result;
use rive_parser::ast::Statement as AstStatement;

impl AstLowering {
    /// Lowers a statement.
    pub(crate) fn lower_statement(&mut self, stmt: &AstStatement) -> Result<RirStatement> {
        match stmt {
            AstStatement::Let {
                name,
                mutable,
                var_type,
                initializer,
                span,
            } => {
                let value = self.lower_expression(initializer)?;
                // Type is already a TypeId from the parser
                let type_id = var_type.unwrap_or_else(|| value.type_id());

                // Register variable in symbol table
                self.define_variable(name.clone(), type_id, *mutable);

                // Determine memory strategy based on type
                let memory_strategy = self.determine_memory_strategy(type_id);

                Ok(RirStatement::Let {
                    name: name.clone(),
                    type_id,
                    is_mutable: *mutable,
                    value: Box::new(value),
                    memory_strategy,
                    span: *span,
                })
            }

            AstStatement::Assignment { name, value, span } => {
                let rir_value = self.lower_expression(value)?;
                Ok(RirStatement::Assign {
                    name: name.clone(),
                    value: Box::new(rir_value),
                    span: *span,
                })
            }

            AstStatement::Expression { expression, span } => {
                // Special handling for control flow that can be statements
                match expression {
                    rive_parser::Expression::If(if_expr) => self.lower_if_stmt(if_expr),
                    rive_parser::Expression::Match(match_expr) => self.lower_match_stmt(match_expr),
                    _ => {
                        let rir_expr = self.lower_expression(expression)?;
                        Ok(RirStatement::Expression {
                            expr: Box::new(rir_expr),
                            span: *span,
                        })
                    }
                }
            }

            AstStatement::Return { value, span } => {
                let rir_value = value
                    .as_ref()
                    .map(|v| self.lower_expression(v))
                    .transpose()?;
                Ok(RirStatement::Return {
                    value: rir_value.map(Box::new),
                    span: *span,
                })
            }

            AstStatement::Break(break_stmt) => self.lower_break(break_stmt),
            AstStatement::Continue(continue_stmt) => self.lower_continue(continue_stmt),
        }
    }
}
