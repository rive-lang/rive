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
        let has_main = program.items.iter().any(|item| match item {
            Item::Function(func) => func.name == "main",
            _ => false,
        });

        if !has_main {
            return Err(Error::Semantic(
                "Program must have a 'main' function".to_string(),
            ));
        }

        // First pass: register all function signatures and type declarations
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let param_types: Vec<_> = func.params.iter().map(|p| p.param_type).collect();
                    let func_type_id = self
                        .symbols
                        .type_registry_mut()
                        .create_function(param_types, func.return_type);

                    let symbol = Symbol::new(func.name.clone(), func_type_id, false);
                    self.symbols.define(symbol)?;
                }
                Item::TypeDecl(type_decl) => {
                    // Register static methods as global functions
                    for method in &type_decl.methods {
                        if method.is_static {
                            let qualified_name = format!("{}_{}", type_decl.name, method.name);
                            let param_types: Vec<_> =
                                method.params.iter().map(|p| p.param_type).collect();
                            let func_type_id = self
                                .symbols
                                .type_registry_mut()
                                .create_function(param_types, method.return_type);
                            let symbol = Symbol::new(qualified_name, func_type_id, false);
                            self.symbols.define(symbol)?;
                        }
                    }

                    // Register inline impl static methods
                    for inline_impl in &type_decl.inline_impls {
                        for method in &inline_impl.methods {
                            if method.is_static {
                                let qualified_name = format!("{}_{}", type_decl.name, method.name);
                                let param_types: Vec<_> =
                                    method.params.iter().map(|p| p.param_type).collect();
                                let func_type_id = self
                                    .symbols
                                    .type_registry_mut()
                                    .create_function(param_types, method.return_type);
                                let symbol = Symbol::new(qualified_name, func_type_id, false);
                                self.symbols.define(symbol)?;
                            }
                        }
                    }
                }
                Item::InterfaceDecl(_interface) => {
                    // Interface declarations define method signatures (no implementation)
                }
                Item::ImplBlock(impl_block) => {
                    // Register static methods in impl blocks
                    for method in &impl_block.methods {
                        if method.is_static {
                            let qualified_name =
                                format!("{}_{}", impl_block.target_type, method.name);
                            let param_types: Vec<_> =
                                method.params.iter().map(|p| p.param_type).collect();
                            let func_type_id = self
                                .symbols
                                .type_registry_mut()
                                .create_function(param_types, method.return_type);
                            let symbol = Symbol::new(qualified_name, func_type_id, false);
                            self.symbols.define(symbol)?;
                        }
                    }
                }
                Item::EnumDecl(enum_decl) => {
                    // Register enum static methods
                    for method in &enum_decl.methods {
                        if method.is_static {
                            let qualified_name = format!("{}_{}", enum_decl.name, method.name);
                            let param_types: Vec<_> =
                                method.params.iter().map(|p| p.param_type).collect();
                            let func_type_id = self
                                .symbols
                                .type_registry_mut()
                                .create_function(param_types, method.return_type);
                            let symbol = Symbol::new(qualified_name, func_type_id, false);
                            self.symbols.define(symbol)?;
                        }
                    }
                }
            }
        }

        // Second pass: type check each function body
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.check_function(func)?;
                }
                Item::TypeDecl(type_decl) => {
                    self.check_type_methods(type_decl)?;
                }
                Item::InterfaceDecl(interface) => {
                    self.check_interface(interface)?;
                }
                Item::ImplBlock(impl_block) => {
                    self.check_impl_block(impl_block)?;
                }
                Item::EnumDecl(enum_decl) => {
                    // Check enum methods
                    for method in &enum_decl.methods {
                        self.check_method_decl(method, &enum_decl.name)?;
                    }
                }
            }
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

    /// Type check methods in a type declaration
    fn check_type_methods(&mut self, type_decl: &rive_parser::ast::TypeDecl) -> Result<()> {
        for method in &type_decl.methods {
            self.check_method_decl(method, &type_decl.name)?;
        }

        // Check inline impl methods
        for inline_impl in &type_decl.inline_impls {
            for method in &inline_impl.methods {
                self.check_method_decl(method, &type_decl.name)?;
            }
        }

        Ok(())
    }

    /// Type check an interface declaration
    fn check_interface(&mut self, _interface: &rive_parser::ast::InterfaceDecl) -> Result<()> {
        // Interface signatures are already validated by the parser
        // TODO: Store interface requirements for later impl checking
        Ok(())
    }

    /// Type check an impl block
    fn check_impl_block(&mut self, impl_block: &rive_parser::ast::ImplBlock) -> Result<()> {
        for method in &impl_block.methods {
            self.check_method_decl(method, &impl_block.target_type)?;
        }
        Ok(())
    }

    /// Type check a method declaration
    fn check_method_decl(
        &mut self,
        method: &rive_parser::ast::MethodDecl,
        type_name: &str,
    ) -> Result<()> {
        // Enter a new scope for the method
        self.symbols.enter_scope();

        // Set current function return type
        self.current_function_return_type = Some(method.return_type);

        // Add 'self' parameter for instance methods
        if !method.is_static {
            // Get the type ID for this type
            let type_id = self
                .symbols
                .type_registry()
                .get_by_name(type_name)
                .ok_or_else(|| Error::Semantic(format!("Type '{}' not found", type_name)))?;

            // Register 'self' in symbol table
            let self_symbol = crate::symbol_table::Symbol::new("self".to_string(), type_id, false);
            self.symbols.define(self_symbol)?;

            // Also register all fields as accessible directly
            let metadata = self
                .symbols
                .type_registry()
                .get_type_metadata(type_id)
                .clone();
            if let rive_core::type_system::TypeKind::Struct { fields, .. } = &metadata.kind {
                for (field_name, field_type) in fields {
                    let field_symbol =
                        crate::symbol_table::Symbol::new(field_name.clone(), *field_type, false);
                    self.symbols.define(field_symbol)?;
                }
            }
        }

        // Add method parameters to scope
        for param in &method.params {
            let symbol = crate::symbol_table::Symbol::new(
                param.name.clone(),
                param.param_type,
                false, // parameters are immutable
            );
            self.symbols.define(symbol)?;
        }

        // Type check the method body
        let body_type = match &method.body {
            rive_parser::ast::FunctionBody::Block(block) => self.check_block_with_value(block)?,
            rive_parser::ast::FunctionBody::Expression(expr) => self.check_expression(expr)?,
        };

        // Verify return type matches (unless it's Unit, which is compatible with anything)
        if method.return_type != TypeId::UNIT && body_type != method.return_type {
            // Allow Unit body for any return type (empty methods)
            if body_type != TypeId::UNIT {
                return Err(Error::Semantic(format!(
                    "Method '{}::{}' return type mismatch: expected {:?}, got {:?}",
                    type_name, method.name, method.return_type, body_type
                )));
            }
        }

        // Exit method scope
        self.symbols.exit_scope();
        self.current_function_return_type = None;

        Ok(())
    }
}
