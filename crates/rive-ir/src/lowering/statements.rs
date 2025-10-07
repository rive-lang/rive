//! Statement lowering.

use crate::lowering::core::AstLowering;
use crate::{RirExpression, RirStatement};
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
                infer_nullable,
                initializer,
                span,
            } => self.lower_variable_declaration(
                name,
                *mutable,
                var_type,
                *infer_nullable,
                initializer,
                *span,
            ),

            AstStatement::Const {
                name,
                var_type,
                infer_nullable,
                initializer,
                span,
            } => self.lower_variable_declaration(
                name,
                false,
                var_type,
                *infer_nullable,
                initializer,
                *span,
            ),

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

    /// Lowers a variable declaration (let or const).
    fn lower_variable_declaration(
        &mut self,
        name: &str,
        mutable: bool,
        var_type: &Option<rive_core::type_system::TypeId>,
        infer_nullable: bool,
        initializer: &rive_parser::Expression,
        span: rive_core::Span,
    ) -> Result<RirStatement> {
        let value = self.lower_expression(initializer)?;

        // Determine the final type
        let type_id = if let Some(explicit_type) = var_type {
            // Explicit type annotation
            *explicit_type
        } else if infer_nullable {
            // Infer as nullable (e.g., `let x? = expr`)
            self.get_or_create_nullable(value.type_id())
        } else {
            // Normal type inference
            value.type_id()
        };

        // Check if we need T -> T? conversion
        let final_value = if value.type_id() != type_id {
            // Check if this is T -> T? conversion
            if let Some(inner_type) = self.get_nullable_inner(type_id)
                && value.type_id() == inner_type
            {
                // Create WrapOptional node for T -> T? conversion
                RirExpression::WrapOptional {
                    value: Box::new(value),
                    result_type: type_id,
                    span,
                }
            } else {
                value
            }
        } else {
            value
        };

        // Register variable in symbol table
        self.define_variable(name.to_string(), type_id, mutable);

        // Determine memory strategy based on type
        let memory_strategy = self.determine_memory_strategy(type_id);

        Ok(RirStatement::Let {
            name: name.to_string(),
            type_id,
            is_mutable: mutable,
            value: Box::new(final_value),
            memory_strategy,
            span,
        })
    }
}
