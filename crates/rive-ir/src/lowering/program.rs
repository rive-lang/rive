//! Program and function lowering.

use crate::lowering::core::AstLowering;
use crate::{RirBlock, RirFunction, RirModule, RirParameter};
use rive_core::Result;
use rive_parser::ast::{Function as AstFunction, Item, Program};

impl AstLowering {
    /// Lowers a complete program to RIR.
    pub fn lower_program(&mut self, program: &Program) -> Result<RirModule> {
        // First pass: register all function signatures
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let param_types: Vec<_> = func.params.iter().map(|p| p.param_type).collect();
                    let return_type = func.return_type;
                    self.define_function(func.name.clone(), param_types, return_type);
                }
            }
        }

        // Second pass: lower function bodies
        let mut functions = Vec::new();
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let rir_func = self.lower_function(func)?;
                    functions.push(rir_func);
                }
            }
        }

        // Create module with the updated type registry (after all types have been created)
        let mut module = RirModule::new(self.type_registry.clone());
        for func in functions {
            module.add_function(func);
        }

        Ok(module)
    }

    /// Lowers a function declaration.
    pub(crate) fn lower_function(&mut self, func: &AstFunction) -> Result<RirFunction> {
        // Enter function scope
        self.enter_scope();

        // Register parameters in symbol table
        let parameters: Vec<RirParameter> = func
            .params
            .iter()
            .map(|p| {
                let type_id = p.param_type;
                self.define_variable(p.name.clone(), type_id, false);
                let memory_strategy = self.determine_memory_strategy(type_id);
                Ok(RirParameter::new(
                    p.name.clone(),
                    type_id,
                    false, // Parameters are not mutable by default in Rive
                    memory_strategy,
                    p.span,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        let return_type = func.return_type;
        let body = self.lower_block(&func.body)?;

        // Exit function scope
        self.exit_scope();

        Ok(RirFunction::new(
            func.name.clone(),
            parameters,
            return_type,
            body,
            func.span,
        ))
    }

    /// Lowers a block of statements.
    pub(crate) fn lower_block(&mut self, block: &rive_parser::Block) -> Result<RirBlock> {
        let mut rir_block = RirBlock::new(block.span);

        // Check if the last statement is an expression (for implicit return)
        let statements_count = block.statements.len();

        for (i, stmt) in block.statements.iter().enumerate() {
            let is_last = i == statements_count - 1;

            // If this is the last statement and it's an expression statement,
            // treat it as the final expression (implicit return)
            if is_last && let rive_parser::Statement::Expression { expression, .. } = stmt {
                // Check if this expression produces a value (not Unit)
                let should_be_final = !matches!(
                    expression,
                    rive_parser::Expression::Call { .. }
                        | rive_parser::Expression::If(_)
                        | rive_parser::Expression::Match(_)
                );

                if should_be_final {
                    let final_expr = self.lower_expression(expression)?;
                    // Only set as final_expr if it's not Unit type
                    if final_expr.type_id() != rive_core::type_system::TypeId::UNIT {
                        rir_block.final_expr = Some(Box::new(final_expr));
                        continue;
                    }
                }
            }

            let rir_stmt = self.lower_statement(stmt)?;
            rir_block.add_statement(rir_stmt);
        }

        Ok(rir_block)
    }
}
