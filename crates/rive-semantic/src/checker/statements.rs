//! Statement type checking.

use crate::checker::core::TypeChecker;
use crate::symbol_table::Symbol;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::ast::{Expression, Statement};

impl TypeChecker {
    /// Checks a statement.
    pub(crate) fn check_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Let {
                name,
                mutable,
                var_type,
                infer_nullable,
                initializer,
                span,
            } => self.check_let(
                name,
                *mutable,
                var_type,
                *infer_nullable,
                initializer,
                *span,
            ),

            Statement::Assignment { name, value, span } => {
                self.check_assignment(name, value, *span)
            }

            Statement::Expression { expression, .. } => self.check_expression_statement(expression),

            Statement::Return { value, span } => self.check_return(value.as_ref(), *span),

            Statement::Break(break_stmt) => {
                self.check_break(break_stmt)?;
                Ok(())
            }

            Statement::Continue(continue_stmt) => {
                self.check_continue(continue_stmt)?;
                Ok(())
            }
        }
    }

    /// Checks a let statement.
    fn check_let(
        &mut self,
        name: &str,
        mutable: bool,
        var_type: &Option<rive_core::type_system::TypeId>,
        infer_nullable: bool,
        initializer: &Expression,
        span: rive_core::Span,
    ) -> Result<()> {
        let init_type = self.check_expression(initializer)?;

        // Determine the final variable type
        let var_type_id = if let Some(annotated_type) = var_type {
            // Explicit type annotation
            if !self.types_compatible(*annotated_type, init_type) {
                return Err(self.type_mismatch_error(
                    &format!("Variable '{name}' type mismatch"),
                    *annotated_type,
                    init_type,
                    span,
                ));
            }
            *annotated_type
        } else if infer_nullable {
            // Infer as nullable (e.g., `let x? = expr`)
            self.get_or_create_nullable(init_type)
        } else {
            // Normal type inference
            init_type
        };

        let symbol = Symbol::new(name.to_string(), var_type_id, mutable);
        self.symbols.define(symbol)?;
        Ok(())
    }

    /// Checks an assignment statement.
    fn check_assignment(
        &mut self,
        name: &str,
        value: &Expression,
        span: rive_core::Span,
    ) -> Result<()> {
        // Clone symbol data to avoid borrow conflicts
        let (is_mutable, expected_type) = {
            let var_symbol = self.symbols.lookup(name).ok_or_else(|| {
                Error::SemanticWithSpan(format!("Undefined variable '{name}'"), span)
            })?;
            (var_symbol.mutable, var_symbol.symbol_type)
        };

        if !is_mutable {
            return Err(Error::SemanticWithSpan(
                format!("Cannot assign to immutable variable '{name}'"),
                span,
            ));
        }

        let value_type = self.check_expression(value)?;
        // Check if value_type can be assigned to expected_type
        if !self.types_compatible(expected_type, value_type) {
            return Err(self.type_mismatch_error(
                &format!("Cannot assign to variable '{name}'"),
                expected_type,
                value_type,
                span,
            ));
        }

        Ok(())
    }

    /// Checks an expression statement.
    fn check_expression_statement(&mut self, expression: &Expression) -> Result<()> {
        // Special handling for control flow structures that can be both expressions and statements
        match expression {
            Expression::If(if_expr) => {
                // If used as statement, doesn't require else branch
                self.check_if(if_expr, false)?;
                Ok(())
            }
            Expression::Match(match_expr) => {
                // Match used as statement
                self.check_match(match_expr, false)?;
                Ok(())
            }
            _ => {
                self.check_expression(expression)?;
                Ok(())
            }
        }
    }

    /// Checks a return statement.
    fn check_return(&mut self, value: Option<&Expression>, span: rive_core::Span) -> Result<()> {
        let return_type_id = self.current_function_return_type.ok_or_else(|| {
            Error::SemanticWithSpan("Return statement outside of function".to_string(), span)
        })?;

        let value_type = if let Some(expr) = value {
            self.check_expression(expr)?
        } else {
            TypeId::UNIT
        };

        // Check if value_type can be assigned to return_type_id
        if !self.types_compatible(return_type_id, value_type) {
            return Err(self.type_mismatch_error(
                "Return type mismatch",
                return_type_id,
                value_type,
                span,
            ));
        }

        Ok(())
    }
}
