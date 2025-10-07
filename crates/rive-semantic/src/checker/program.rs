//! Program and function type checking.

use crate::checker::core::TypeChecker;
use crate::symbol_table::Symbol;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::ast::{Function, FunctionBody, Item, Program};

impl TypeChecker {
    /// Checks a complete program.
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // Check that a main function exists
        let has_main = program.items.iter().any(|item| {
            let Item::Function(func) = item;
            func.name == "main"
        });

        if !has_main {
            return Err(Error::Semantic(
                "Program must have a 'main' function".to_string(),
            ));
        }

        // First pass: register all function signatures
        for item in &program.items {
            let Item::Function(func) = item;
            let param_types: Vec<_> = func.params.iter().map(|p| p.param_type).collect();
            let func_type_id = self
                .symbols
                .type_registry_mut()
                .create_function(param_types, func.return_type);

            let symbol = Symbol::new(func.name.clone(), func_type_id, false);
            self.symbols.define(symbol)?;
        }

        // Second pass: type check each function body
        for item in &program.items {
            let Item::Function(func) = item;
            self.check_function(func)?;
        }

        Ok(())
    }

    /// Checks a function declaration.
    pub(crate) fn check_function(&mut self, func: &Function) -> Result<()> {
        // Enter function scope
        self.symbols.enter_scope();
        self.current_function_return_type = Some(func.return_type);

        // Register parameters in the function scope
        for param in &func.params {
            let symbol = Symbol::new(param.name.clone(), param.param_type, false);
            self.symbols.define(symbol)?;
        }

        // Check function body based on its type
        match &func.body {
            FunctionBody::Block(block) => {
                self.check_block(block)?;
            }
            FunctionBody::Expression(expr) => {
                // For expression bodies, check that the expression type matches the return type
                let expr_type = self.check_expression(expr)?;
                if !self.types_compatible(func.return_type, expr_type) {
                    return Err(self.type_mismatch_error(
                        &format!("Function '{}' expression body type mismatch", func.name),
                        func.return_type,
                        expr_type,
                        func.span,
                    ));
                }
            }
        }

        // Exit function scope
        self.symbols.exit_scope();
        self.current_function_return_type = None;

        Ok(())
    }

    /// Checks a block of statements.
    pub(crate) fn check_block(&mut self, block: &rive_parser::Block) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    /// Checks a block and returns its type (considering implicit return).
    pub(crate) fn check_block_with_value(
        &mut self,
        block: &rive_parser::Block,
    ) -> Result<rive_core::type_system::TypeId> {
        if block.statements.is_empty() {
            return Ok(TypeId::UNIT);
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
                Ok(TypeId::UNIT)
            }
        }
    }
}
