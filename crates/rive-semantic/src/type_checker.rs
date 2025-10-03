//! Type checking for Rive programs.

use crate::symbol_table::{Symbol, SymbolTable};
use crate::type_checker_control_flow::LoopContext;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result};
use rive_parser::ast::{Block, Expression, Function, Item, Program, Statement};

/// Type checker for Rive programs.
///
/// Performs type checking and semantic validation on AST nodes,
/// ensuring type safety and proper variable usage.
pub struct TypeChecker {
    /// Symbol table for tracking variables and functions
    pub(crate) symbols: SymbolTable,
    /// The expected return type of the current function
    current_function_return_type: Option<TypeId>,
    /// Stack of loop contexts for break/continue validation
    pub(crate) loop_stack: Vec<LoopContext>,
}

impl TypeChecker {
    /// Creates a new type checker.
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            current_function_return_type: None,
            loop_stack: Vec::new(),
        }
    }

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
            let param_types: Vec<TypeId> = func.params.iter().map(|p| p.param_type).collect();
            let func_type_id = self
                .symbols
                .type_registry_mut()
                .get_or_create_function(param_types, func.return_type);

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
    fn check_function(&mut self, func: &Function) -> Result<()> {
        // Enter function scope
        self.symbols.enter_scope();
        self.current_function_return_type = Some(func.return_type);

        // Register parameters in the function scope
        for param in &func.params {
            let symbol = Symbol::new(param.name.clone(), param.param_type, false);
            self.symbols.define(symbol)?;
        }

        // Check function body
        self.check_block(&func.body)?;

        // Exit function scope
        self.symbols.exit_scope();
        self.current_function_return_type = None;

        Ok(())
    }

    /// Checks a block of statements.
    pub(crate) fn check_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    /// Checks a statement.
    pub(crate) fn check_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Let {
                name,
                mutable,
                var_type,
                initializer,
                span,
            } => {
                let init_type = self.check_expression(initializer)?;

                // If type annotation is present, verify it matches the initializer
                let var_type_id = if let Some(annotated_type) = var_type {
                    if !self.types_compatible(init_type, *annotated_type) {
                        let registry = self.symbols.type_registry();
                        let init_str = registry.get_type_name(init_type);
                        let annot_str = registry.get_type_name(*annotated_type);
                        return Err(Error::SemanticWithSpan(
                            format!(
                                "Type mismatch: variable '{name}' declared as '{annot_str}' but initialized with '{init_str}'"
                            ),
                            *span,
                        ));
                    }
                    *annotated_type
                } else {
                    init_type
                };

                let symbol = Symbol::new(name.clone(), var_type_id, *mutable);
                self.symbols.define(symbol)?;
                Ok(())
            }

            Statement::Assignment { name, value, span } => {
                // Clone symbol data to avoid borrow conflicts
                let (is_mutable, expected_type) = {
                    let var_symbol = self.symbols.lookup(name).ok_or_else(|| {
                        Error::SemanticWithSpan(format!("Undefined variable '{name}'"), *span)
                    })?;
                    (var_symbol.mutable, var_symbol.symbol_type)
                };

                if !is_mutable {
                    return Err(Error::SemanticWithSpan(
                        format!("Cannot assign to immutable variable '{name}'"),
                        *span,
                    ));
                }

                let value_type = self.check_expression(value)?;
                if !self.types_compatible(value_type, expected_type) {
                    let registry = self.symbols.type_registry();
                    let value_str = registry.get_type_name(value_type);
                    let var_str = registry.get_type_name(expected_type);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Type mismatch: cannot assign '{value_str}' to variable of type '{var_str}'"
                        ),
                        *span,
                    ));
                }

                Ok(())
            }

            Statement::Expression { expression, .. } => {
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

            Statement::Return { value, span } => {
                let return_type_id = self.current_function_return_type.ok_or_else(|| {
                    Error::SemanticWithSpan(
                        "Return statement outside of function".to_string(),
                        *span,
                    )
                })?;

                let value_type = if let Some(expr) = value {
                    self.check_expression(expr)?
                } else {
                    self.symbols.type_registry().get_unit()
                };

                if !self.types_compatible(value_type, return_type_id) {
                    let registry = self.symbols.type_registry();
                    let value_str = registry.get_type_name(value_type);
                    let return_str = registry.get_type_name(return_type_id);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Type mismatch: function returns '{return_str}' but found '{value_str}'"
                        ),
                        *span,
                    ));
                }

                Ok(())
            }

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

    /// Checks an expression and returns its type.
    pub(crate) fn check_expression(&mut self, expr: &Expression) -> Result<TypeId> {
        match expr {
            Expression::Integer { .. } => Ok(self.symbols.type_registry().get_int()),
            Expression::Float { .. } => Ok(self.symbols.type_registry().get_float()),
            Expression::String { .. } => Ok(self.symbols.type_registry().get_text()),
            Expression::Boolean { .. } => Ok(self.symbols.type_registry().get_bool()),
            Expression::Null { span } => Err(Error::SemanticWithSpan(
                "Null type not yet supported".to_string(),
                *span,
            )),

            Expression::Variable { name, span } => {
                let symbol = self.symbols.lookup(name).ok_or_else(|| {
                    Error::SemanticWithSpan(format!("Undefined variable '{name}'"), *span)
                })?;
                Ok(symbol.symbol_type)
            }

            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                use rive_parser::BinaryOperator;

                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                // Type compatibility check
                if !self.types_compatible(left_type, right_type) {
                    let registry = self.symbols.type_registry();
                    let left_str = registry.get_type_name(left_type);
                    let right_str = registry.get_type_name(right_type);
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Type mismatch in binary operation: '{left_str}' and '{right_str}'"
                        ),
                        *span,
                    ));
                }

                // Determine result type based on operator
                let result_type = match operator {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide
                    | BinaryOperator::Modulo => left_type,

                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::Less
                    | BinaryOperator::LessEqual
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::And
                    | BinaryOperator::Or => self.symbols.type_registry().get_bool(),
                };

                Ok(result_type)
            }

            Expression::Unary {
                operator,
                operand,
                span,
            } => {
                use rive_parser::UnaryOperator;

                let operand_type = self.check_expression(operand)?;

                match operator {
                    UnaryOperator::Negate => {
                        let int_type = self.symbols.type_registry().get_int();
                        let float_type = self.symbols.type_registry().get_float();
                        if !self.types_compatible(operand_type, int_type)
                            && !self.types_compatible(operand_type, float_type)
                        {
                            let registry = self.symbols.type_registry();
                            let type_str = registry.get_type_name(operand_type);
                            return Err(Error::SemanticWithSpan(
                                format!("Cannot negate type '{type_str}'"),
                                *span,
                            ));
                        }
                        Ok(operand_type)
                    }
                    UnaryOperator::Not => {
                        let bool_type = self.symbols.type_registry().get_bool();
                        if !self.types_compatible(operand_type, bool_type) {
                            let registry = self.symbols.type_registry();
                            let type_str = registry.get_type_name(operand_type);
                            return Err(Error::SemanticWithSpan(
                                format!("Cannot apply logical NOT to type '{type_str}'"),
                                *span,
                            ));
                        }
                        Ok(bool_type)
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
                    return Ok(self.symbols.type_registry().get_unit());
                }

                // Look up function symbol and clone its type to avoid borrow issues
                let func_type_id = self
                    .symbols
                    .lookup(callee)
                    .ok_or_else(|| {
                        Error::SemanticWithSpan(format!("Undefined function '{callee}'"), *span)
                    })?
                    .symbol_type;

                // Extract function type
                let (param_types, return_type) = {
                    let registry = self.symbols.type_registry();
                    let metadata = registry.get_type_metadata(func_type_id);

                    use rive_core::type_system::TypeKind;
                    if let TypeKind::Function {
                        parameters,
                        return_type,
                    } = &metadata.kind
                    {
                        (parameters.clone(), *return_type)
                    } else {
                        return Err(Error::SemanticWithSpan(
                            format!("'{callee}' is not a function"),
                            *span,
                        ));
                    }
                };

                // Check argument count
                if args.len() != param_types.len() {
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Function '{callee}' expects {} arguments, but {} were provided",
                            param_types.len(),
                            args.len()
                        ),
                        *span,
                    ));
                }

                // Clone arguments to avoid borrow issues during checking
                let args_clone = arguments.clone();

                // Check argument types
                for (i, (expected_type, arg)) in
                    param_types.iter().zip(args_clone.iter()).enumerate()
                {
                    let arg_type = self.check_expression(arg)?;
                    if !self.types_compatible(arg_type, *expected_type) {
                        let registry = self.symbols.type_registry();
                        let arg_str = registry.get_type_name(arg_type);
                        let expected_str = registry.get_type_name(*expected_type);
                        return Err(Error::SemanticWithSpan(
                            format!(
                                "Argument {} type mismatch: expected '{expected_str}', found '{arg_str}'",
                                i + 1
                            ),
                            *span,
                        ));
                    }
                }

                Ok(return_type)
            }

            Expression::Array { elements, span } => {
                if elements.is_empty() {
                    return Err(Error::SemanticWithSpan(
                        "Empty arrays are not supported".to_string(),
                        *span,
                    ));
                }

                // Check all elements have the same type
                let first_type = self.check_expression(&elements[0])?;
                for (i, elem) in elements.iter().enumerate().skip(1) {
                    let elem_type = self.check_expression(elem)?;
                    if !self.types_compatible(elem_type, first_type) {
                        let registry = self.symbols.type_registry();
                        let first_str = registry.get_type_name(first_type);
                        let elem_str = registry.get_type_name(elem_type);
                        return Err(Error::SemanticWithSpan(
                            format!(
                                "Array element type mismatch: expected '{first_str}', found '{elem_str}' at index {i}"
                            ),
                            *span,
                        ));
                    }
                }

                // Create array type
                let array_type = self
                    .symbols
                    .type_registry_mut()
                    .get_or_create_array(first_type, elements.len());
                Ok(array_type)
            }

            // Control flow expressions
            Expression::If(if_expr) => {
                // If is used as expression, so needs value
                self.check_if(if_expr, true)
            }

            Expression::While(while_loop) => self.check_while(while_loop),

            Expression::For(for_loop) => self.check_for(for_loop),

            Expression::Loop(loop_expr) => self.check_loop(loop_expr),

            Expression::Match(match_expr) => self.check_match(match_expr, true),

            Expression::Range(range) => self.check_range(range),
        }
    }

    /// Checks if two types are compatible.
    fn types_compatible(&self, a: TypeId, b: TypeId) -> bool {
        a == b
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_lexer::tokenize;
    use rive_parser::parse_with_types;

    fn check_program_source(source: &str) -> Result<()> {
        let tokens = tokenize(source).unwrap();
        let (program, type_registry) = parse_with_types(&tokens).unwrap();
        let mut type_checker = TypeChecker::new();
        type_checker.symbols = SymbolTable::with_registry(type_registry);
        type_checker.check_program(&program)
    }

    #[test]
    fn test_simple_function() {
        let source = "fun main() { let x: Int = 42 }";
        assert!(check_program_source(source).is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let source = "fun main() { let x: Int = \"hello\" }";
        assert!(check_program_source(source).is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let source = "fun main() { let x = y }";
        assert!(check_program_source(source).is_err());
    }

    #[test]
    fn test_function_call() {
        let source = r#"
            fun add(x: Int, y: Int): Int { return x + y }
            fun main() { let result = add(1, 2) }
        "#;
        assert!(check_program_source(source).is_ok());
    }

    #[test]
    fn test_immutable_assignment() {
        let source = "fun main() { let x = 42 x = 43 }";
        assert!(check_program_source(source).is_err());
    }

    #[test]
    fn test_mutable_assignment() {
        let source = "fun main() { let mut x = 42 x = 43 }";
        assert!(check_program_source(source).is_ok());
    }
}
