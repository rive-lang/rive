//! Type checking for Rive programs.

use crate::symbol_table::{Symbol, SymbolTable};
use rive_core::{Error, Result, types::Type};
use rive_parser::ast::{Block, Expression, Function, Item, Parameter, Program, Statement};

/// Type checker for Rive programs.
///
/// Performs type checking and semantic validation on AST nodes,
/// ensuring type safety and proper variable usage.
pub struct TypeChecker {
    /// Symbol table for tracking variables and functions
    symbols: SymbolTable,
    /// The expected return type of the current function
    current_function_return_type: Option<Type>,
}

impl TypeChecker {
    /// Creates a new type checker.
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            current_function_return_type: None,
        }
    }

    /// Checks a complete program.
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // First pass: register all function declarations
        for item in &program.items {
            let Item::Function(func) = item;
            self.register_function(func)?;
        }

        // Second pass: check function bodies
        for item in &program.items {
            let Item::Function(func) = item;
            self.check_function(func)?;
        }

        // Ensure main function exists
        if self.symbols.lookup("main").is_none() {
            return Err(Error::Semantic(
                "Program must have a 'main' function".to_string(),
            ));
        }

        Ok(())
    }

    /// Registers a function in the symbol table without checking its body.
    fn register_function(&mut self, func: &Function) -> Result<()> {
        let param_types: Vec<Type> = func.params.iter().map(|p| p.param_type.clone()).collect();
        let return_type = func.return_type.clone();

        let func_type = Type::Function {
            parameters: param_types,
            return_type: Box::new(return_type),
        };

        let symbol = Symbol::new(func.name.clone(), func_type, false);
        self.symbols.define(symbol).map_err(Error::Semantic)?;

        Ok(())
    }

    /// Checks a function declaration.
    fn check_function(&mut self, func: &Function) -> Result<()> {
        // Enter function scope
        self.symbols.enter_scope();

        // Set current function return type
        self.current_function_return_type = Some(func.return_type.clone());

        // Register parameters
        for param in &func.params {
            self.check_parameter(param)?;
        }

        // Check function body
        self.check_block(&func.body)?;

        // Exit function scope
        self.symbols.exit_scope();
        self.current_function_return_type = None;

        Ok(())
    }

    /// Checks a function parameter.
    fn check_parameter(&mut self, param: &Parameter) -> Result<()> {
        let symbol = Symbol::new(param.name.clone(), param.param_type.clone(), false);
        self.symbols.define(symbol).map_err(Error::Semantic)?;
        Ok(())
    }

    /// Checks a block of statements.
    fn check_block(&mut self, block: &Block) -> Result<()> {
        for stmt in &block.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    /// Checks a statement.
    fn check_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let {
                name,
                mutable,
                var_type,
                initializer,
                span,
            } => {
                // Check the initializer expression
                let value_type = self.check_expression(initializer)?;

                // If type annotation exists, verify it matches
                if let Some(annotated_type) = var_type
                    && !self.types_compatible(&value_type, annotated_type)
                {
                    return Err(Error::SemanticWithSpan(
                        format!("Type mismatch: expected '{annotated_type}', found '{value_type}'"),
                        *span,
                    ));
                }

                // Define the variable
                let symbol = Symbol::new(
                    name.clone(),
                    var_type.clone().unwrap_or(value_type),
                    *mutable,
                );
                self.symbols
                    .define(symbol)
                    .map_err(|e| Error::SemanticWithSpan(e, *span))?;

                Ok(())
            }
            Statement::Assignment { name, value, span } => {
                // Check if variable exists and get its properties
                let (is_mutable, expected_type) = {
                    let symbol = self.symbols.lookup(name).ok_or_else(|| {
                        Error::SemanticWithSpan(format!("Undefined variable '{name}'"), *span)
                    })?;
                    (symbol.mutable, symbol.symbol_type.clone())
                };

                // Check if variable is mutable
                if !is_mutable {
                    return Err(Error::SemanticWithSpan(
                        format!("Cannot assign to immutable variable '{name}'"),
                        *span,
                    ));
                }

                // Check the value expression
                let value_type = self.check_expression(value)?;

                // Verify type compatibility
                if !self.types_compatible(&value_type, &expected_type) {
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Type mismatch in assignment: expected '{expected_type}', found '{value_type}'",
                        ),
                        *span,
                    ));
                }

                Ok(())
            }
            Statement::Return { value, span } => {
                let return_type = if let Some(expr) = value {
                    self.check_expression(expr)?
                } else {
                    Type::Unit
                };

                if let Some(expected_type) = &self.current_function_return_type
                    && !self.types_compatible(&return_type, expected_type)
                {
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Return type mismatch: expected '{expected_type}', found '{return_type}'"
                        ),
                        *span,
                    ));
                }

                Ok(())
            }
            Statement::Expression { expression, .. } => {
                self.check_expression(expression)?;
                Ok(())
            }
        }
    }

    /// Checks an expression and returns its type.
    #[allow(clippy::only_used_in_recursion)]
    fn check_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Integer { .. } => Ok(Type::Int),
            Expression::Float { .. } => Ok(Type::Float),
            Expression::String { .. } => Ok(Type::Text),
            Expression::Boolean { .. } => Ok(Type::Bool),
            Expression::Null { span } => Err(Error::SemanticWithSpan(
                "Null literals are not yet supported".to_string(),
                *span,
            )),
            Expression::Unary { span, .. } => Err(Error::SemanticWithSpan(
                "Unary operations are not yet supported".to_string(),
                *span,
            )),
            Expression::Array { elements, span } => {
                if elements.is_empty() {
                    return Err(Error::SemanticWithSpan(
                        "Cannot infer type of empty array".to_string(),
                        *span,
                    ));
                }

                let first_type = self.check_expression(&elements[0])?;
                for (i, elem) in elements.iter().enumerate().skip(1) {
                    let elem_type = self.check_expression(elem)?;
                    if !self.types_compatible(&elem_type, &first_type) {
                        return Err(Error::SemanticWithSpan(
                            format!(
                                "Array element type mismatch at index {i}: expected '{first_type}', found '{elem_type}'"
                            ),
                            *span,
                        ));
                    }
                }

                Ok(Type::Array(Box::new(first_type), elements.len()))
            }
            Expression::Variable { name, span } => {
                if let Some(symbol) = self.symbols.lookup(name) {
                    Ok(symbol.symbol_type.clone())
                } else {
                    Err(Error::SemanticWithSpan(
                        format!("Undefined variable '{name}'"),
                        *span,
                    ))
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                // Check that operands are compatible
                if !self.types_compatible(&left_type, &right_type) {
                    return Err(Error::SemanticWithSpan(
                        format!("Binary operation type mismatch: '{left_type}' and '{right_type}'"),
                        *span,
                    ));
                }

                // Determine result type based on operator
                use rive_parser::ast::BinaryOperator;
                match operator {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide
                    | BinaryOperator::Modulo => {
                        // Arithmetic operators preserve the operand type
                        if !matches!(left_type, Type::Int | Type::Float) {
                            return Err(Error::SemanticWithSpan(
                                format!(
                                    "Arithmetic operation requires Int or Float, found '{left_type}'"
                                ),
                                *span,
                            ));
                        }
                        Ok(left_type)
                    }
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::Less
                    | BinaryOperator::LessEqual
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterEqual => {
                        // Comparison operators return Bool
                        Ok(Type::Bool)
                    }
                    BinaryOperator::And | BinaryOperator::Or => {
                        // Logical operators require Bool operands
                        if !matches!(left_type, Type::Bool) {
                            return Err(Error::SemanticWithSpan(
                                format!("Logical operation requires Bool, found '{left_type}'"),
                                *span,
                            ));
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Call {
                callee,
                arguments,
                span,
            } => {
                let name = callee;
                let args = arguments;
                // Special handling for built-in print function
                if name == "print" {
                    if args.is_empty() {
                        return Err(Error::SemanticWithSpan(
                            "print requires at least one argument".to_string(),
                            *span,
                        ));
                    }
                    // Check all arguments
                    for arg in args {
                        self.check_expression(arg)?;
                    }
                    return Ok(Type::Unit);
                }

                // Look up function symbol and clone its type to avoid borrow issues
                let func_type = self
                    .symbols
                    .lookup(name)
                    .ok_or_else(|| {
                        Error::SemanticWithSpan(format!("Undefined function '{name}'"), *span)
                    })?
                    .symbol_type
                    .clone();

                // Extract function type
                if let Type::Function {
                    parameters,
                    return_type,
                } = func_type
                {
                    // Check argument count
                    if args.len() != parameters.len() {
                        return Err(Error::SemanticWithSpan(
                            format!(
                                "Function '{}' expects {} arguments, found {}",
                                name,
                                parameters.len(),
                                args.len()
                            ),
                            *span,
                        ));
                    }

                    // Check argument types
                    for (i, (arg, expected_type)) in args.iter().zip(parameters.iter()).enumerate()
                    {
                        let arg_type = self.check_expression(arg)?;
                        if !self.types_compatible(&arg_type, expected_type) {
                            return Err(Error::SemanticWithSpan(
                                format!(
                                    "Argument {} type mismatch: expected '{}', found '{}'",
                                    i + 1,
                                    expected_type,
                                    arg_type
                                ),
                                *span,
                            ));
                        }
                    }

                    Ok(*return_type)
                } else {
                    Err(Error::SemanticWithSpan(
                        format!("'{name}' is not a function"),
                        *span,
                    ))
                }
            }
        }
    }

    /// Checks if two types are compatible (equal for now, will support subtyping later).
    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        t1 == t2
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
