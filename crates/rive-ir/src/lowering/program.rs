//! Program and function lowering.

use crate::lowering::core::AstLowering;
use crate::{RirBlock, RirFunction, RirModule, RirParameter};
use rive_core::Result;
use rive_parser::ast::{Function as AstFunction, FunctionBody, Item, Program};

impl AstLowering {
    /// Lowers a complete program to RIR.
    pub fn lower_program(&mut self, program: &Program) -> Result<RirModule> {
        // First pass: register all function signatures and type declarations
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let param_types: Vec<_> = func.params.iter().map(|p| p.param_type).collect();
                    let return_type = func.return_type;
                    self.define_function(func.name.clone(), param_types, return_type);
                }
                Item::TypeDecl(type_decl) => {
                    // Register methods in the function table
                    for method in &type_decl.methods {
                        let func_name = if method.is_static {
                            format!("{}_{}", type_decl.name, method.name)
                        } else {
                            format!("{}_instance_{}", type_decl.name, method.name)
                        };
                        let params: Vec<rive_core::type_system::TypeId> =
                            method.params.iter().map(|p| p.param_type).collect();
                        self.define_function(func_name, params, method.return_type);
                    }

                    // Register inline impl methods
                    for inline_impl in &type_decl.inline_impls {
                        for method in &inline_impl.methods {
                            let func_name = if method.is_static {
                                format!("{}_{}", type_decl.name, method.name)
                            } else {
                                format!("{}_instance_{}", type_decl.name, method.name)
                            };
                            let params: Vec<rive_core::type_system::TypeId> =
                                method.params.iter().map(|p| p.param_type).collect();
                            self.define_function(func_name, params, method.return_type);
                        }
                    }
                }
                Item::InterfaceDecl(_interface) => {
                    // Interface declarations define method signatures
                    // No code generation needed
                }
                Item::ImplBlock(impl_block) => {
                    // Register impl block methods in the function table
                    for method in &impl_block.methods {
                        let func_name = if method.is_static {
                            format!("{}_{}", impl_block.target_type, method.name)
                        } else {
                            format!("{}_instance_{}", impl_block.target_type, method.name)
                        };
                        let params: Vec<rive_core::type_system::TypeId> =
                            method.params.iter().map(|p| p.param_type).collect();
                        self.define_function(func_name, params, method.return_type);
                    }
                }
                Item::EnumDecl(enum_decl) => {
                    // Register enum methods in the function table
                    for method in &enum_decl.methods {
                        let func_name = if method.is_static {
                            format!("{}_{}", enum_decl.name, method.name)
                        } else {
                            format!("{}_instance_{}", enum_decl.name, method.name)
                        };
                        let params: Vec<rive_core::type_system::TypeId> =
                            method.params.iter().map(|p| p.param_type).collect();
                        self.define_function(func_name, params, method.return_type);
                    }
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
                Item::TypeDecl(type_decl) => {
                    // Get type ID
                    let type_id = self.type_registry.get_by_name(&type_decl.name).unwrap();

                    // Lower type methods to RIR functions
                    for method in &type_decl.methods {
                        let func_name = if method.is_static {
                            format!("{}_{}", type_decl.name, method.name)
                        } else {
                            format!("{}_instance_{}", type_decl.name, method.name)
                        };
                        let rir_func = self.lower_method(method, &func_name, type_id)?;
                        functions.push(rir_func);
                    }

                    // Lower inline impl methods
                    for inline_impl in &type_decl.inline_impls {
                        for method in &inline_impl.methods {
                            let func_name = if method.is_static {
                                format!("{}_{}", type_decl.name, method.name)
                            } else {
                                format!("{}_instance_{}", type_decl.name, method.name)
                            };
                            let rir_func = self.lower_method(method, &func_name, type_id)?;
                            functions.push(rir_func);
                        }
                    }
                }
                Item::InterfaceDecl(_interface) => {
                    // Interface declarations don't generate code directly
                }
                Item::ImplBlock(impl_block) => {
                    // Get type ID
                    let type_id = self
                        .type_registry
                        .get_by_name(&impl_block.target_type)
                        .unwrap();

                    // Lower impl block methods to RIR functions
                    for method in &impl_block.methods {
                        let func_name = if method.is_static {
                            format!("{}_{}", impl_block.target_type, method.name)
                        } else {
                            format!("{}_instance_{}", impl_block.target_type, method.name)
                        };
                        let rir_func = self.lower_method(method, &func_name, type_id)?;
                        functions.push(rir_func);
                    }
                }
                Item::EnumDecl(enum_decl) => {
                    // Get enum type ID
                    let type_id = self.type_registry.get_by_name(&enum_decl.name).unwrap();

                    // Lower enum methods to RIR functions
                    for method in &enum_decl.methods {
                        let func_name = if method.is_static {
                            format!("{}_{}", enum_decl.name, method.name)
                        } else {
                            format!("{}_instance_{}", enum_decl.name, method.name)
                        };
                        let rir_func = self.lower_method(method, &func_name, type_id)?;
                        functions.push(rir_func);
                    }
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

    /// Lowers a method declaration to RIR.
    fn lower_method(
        &mut self,
        method: &rive_parser::ast::MethodDecl,
        func_name: &str,
        type_id: rive_core::type_system::TypeId,
    ) -> Result<RirFunction> {
        self.enter_scope();

        // For instance methods, add 'self' as the first parameter
        let mut parameters = Vec::new();
        if !method.is_static {
            let memory_strategy = self.determine_memory_strategy(type_id);
            let self_param = RirParameter::new(
                "self".to_string(),
                type_id,
                false, // self is immutable by default
                memory_strategy,
                method.span,
            );
            parameters.push(self_param);
            self.define_variable("self".to_string(), type_id, false);

            // Don't register fields as variables - they should be accessed via self.field
        }

        // Add user-defined parameters
        let user_params: Vec<RirParameter> = method
            .params
            .iter()
            .map(|p| {
                let type_id = p.param_type;
                self.define_variable(p.name.clone(), type_id, false);
                let memory_strategy = self.determine_memory_strategy(type_id);
                Ok(RirParameter::new(
                    p.name.clone(),
                    type_id,
                    false,
                    memory_strategy,
                    p.span,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        parameters.extend(user_params);

        let return_type = method.return_type;

        // Lower the method body
        let body = match &method.body {
            rive_parser::ast::FunctionBody::Block(block) => self.lower_block(block)?,
            rive_parser::ast::FunctionBody::Expression(expr) => {
                // For expression bodies, create a block with just the expression as final_expr
                let mut rir_block = RirBlock::new(expr.span());
                let final_expr = self.lower_expression(expr)?;
                rir_block.final_expr = Some(Box::new(final_expr));
                rir_block
            }
        };

        self.exit_scope();

        Ok(RirFunction::new(
            func_name.to_string(),
            parameters,
            return_type,
            body,
            method.span,
        ))
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

        // Lower function body based on its type
        let body = match &func.body {
            FunctionBody::Block(block) => self.lower_block(block)?,
            FunctionBody::Expression(expr) => {
                // For expression bodies, create a block with just the expression as final_expr
                let mut rir_block = RirBlock::new(expr.span());
                let final_expr = self.lower_expression(expr)?;
                rir_block.final_expr = Some(Box::new(final_expr));
                rir_block
            }
        };

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
