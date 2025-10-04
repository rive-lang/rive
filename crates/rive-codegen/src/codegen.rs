//! Core code generation logic.
//!
//! Generates Rust code from the Rive Intermediate Representation (RIR).
//! The RIR includes explicit memory strategy annotations that guide code generation.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{
    Error, Result,
    type_system::{MemoryStrategy, TypeId},
};
use rive_ir::{BinaryOp, RirBlock, RirExpression, RirFunction, RirModule, RirStatement, UnaryOp};

/// Code generator for Rive programs.
pub struct CodeGenerator {}

impl CodeGenerator {
    /// Creates a new code generator.
    pub fn new() -> Self {
        Self {}
    }

    /// Generates Rust code from a RIR module.
    pub fn generate(&mut self, module: &RirModule) -> Result<String> {
        let mut items = Vec::new();

        for function in &module.functions {
            items.push(self.generate_function(function)?);
        }

        let tokens = quote! {
            #(#items)*
        };

        // Parse TokenStream into syn::File for pretty-printing
        let syntax_tree = syn::parse2::<syn::File>(tokens)
            .map_err(|e| Error::Codegen(format!("Failed to parse generated code: {e}")))?;

        // Format using prettyplease
        Ok(prettyplease::unparse(&syntax_tree))
    }

    /// Generates code for a RIR function.
    fn generate_function(&mut self, function: &RirFunction) -> Result<TokenStream> {
        let name = format_ident!("{}", function.name);
        let params = self.generate_parameters(&function.parameters)?;
        let return_type = self.generate_return_type(function.return_type);
        let body = self.generate_block(&function.body)?;

        // Determine if function should be inlined
        let should_inline = self.should_inline_function(function);

        if should_inline {
            Ok(quote! {
                #[inline]
                fn #name(#(#params),*) #return_type {
                    #body
                }
            })
        } else {
            Ok(quote! {
                fn #name(#(#params),*) #return_type {
                    #body
                }
            })
        }
    }

    /// Generates function parameters.
    fn generate_parameters(&self, params: &[rive_ir::RirParameter]) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|param| {
                let name = format_ident!("{}", param.name);
                let ty = self.rust_type(param.type_id, param.memory_strategy)?;
                Ok(quote! { #name: #ty })
            })
            .collect()
    }

    /// Generates return type annotation.
    fn generate_return_type(&self, type_id: TypeId) -> TokenStream {
        if type_id == TypeId::UNIT {
            quote! {}
        } else {
            // Return types typically use Copy or CoW strategy, not RcRefCell
            let rust_ty = self.rust_type(type_id, MemoryStrategy::Copy).unwrap();
            quote! {-> #rust_ty}
        }
    }

    /// Generates code for a binary operation operand, adding parentheses if needed based on precedence.
    fn generate_binary_operand(
        &mut self,
        operand: &RirExpression,
        parent_op: &rive_ir::BinaryOp,
        is_left: bool,
    ) -> Result<TokenStream> {
        // Check if operand is a binary expression that needs parentheses
        if let RirExpression::Binary { op: child_op, .. } = operand {
            let parent_prec = self.operator_precedence(parent_op);
            let child_prec = self.operator_precedence(child_op);
            
            // Add parentheses if:
            // 1. Child has lower precedence than parent
            // 2. Same precedence but right operand (for left-associative operators)
            let needs_parens = child_prec < parent_prec 
                || (child_prec == parent_prec && !is_left && !self.is_right_associative(parent_op));
            
            let expr = self.generate_expression(operand)?;
            if needs_parens {
                return Ok(quote! { (#expr) });
            }
            return Ok(expr);
        }
        
        // Not a binary expression, no parentheses needed
        self.generate_expression(operand)
    }

    /// Returns the precedence of an operator (higher number = higher precedence).
    fn operator_precedence(&self, op: &rive_ir::BinaryOp) -> u8 {
        use rive_ir::BinaryOp;
        match op {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Equal | BinaryOp::NotEqual => 3,
            BinaryOp::LessThan | BinaryOp::LessEqual | BinaryOp::GreaterThan | BinaryOp::GreaterEqual => 4,
            BinaryOp::Add | BinaryOp::Subtract => 5,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 6,
        }
    }

    /// Returns true if the operator is right-associative.
    fn is_right_associative(&self, _op: &rive_ir::BinaryOp) -> bool {
        // All binary operators in Rive are left-associative
        false
    }

    /// Generates code for a RIR block.
    fn generate_block(&mut self, block: &RirBlock) -> Result<TokenStream> {
        let mut statements = Vec::new();

        for stmt in &block.statements {
            statements.push(self.generate_statement(stmt)?);
        }

        // Handle final expression if present (without semicolon for implicit return)
        let result = if let Some(final_expr) = &block.final_expr {
            let expr = self.generate_expression(final_expr)?;
            quote! {
                #(#statements)*
                #expr
            }
        } else {
            quote! {
                #(#statements)*
            }
        };

        Ok(result)
    }

    /// Generates code for a RIR statement.
    fn generate_statement(&mut self, stmt: &RirStatement) -> Result<TokenStream> {
        match stmt {
            RirStatement::Let {
                name,
                is_mutable,
                value,
                ..
            } => {
                let var_name = format_ident!("{}", name);
                let expr = self.generate_expression(value)?;

                if *is_mutable {
                    Ok(quote! {
                        let mut #var_name = #expr;
                    })
                } else {
                    Ok(quote! {
                        let #var_name = #expr;
                    })
                }
            }
            RirStatement::Assign { name, value, .. } => {
                let var_name = format_ident!("{}", name);
                let expr = self.generate_expression(value)?;

                Ok(quote! {
                    #var_name = #expr;
                })
            }
            RirStatement::AssignIndex {
                array,
                index,
                value,
                ..
            } => {
                let array_name = format_ident!("{}", array);
                let index_expr = self.generate_expression(index)?;
                let value_expr = self.generate_expression(value)?;

                Ok(quote! {
                    #array_name[#index_expr] = #value_expr;
                })
            }
            RirStatement::Expression { expr, .. } => {
                let expression = self.generate_expression(expr)?;
                Ok(quote! {
                    #expression;
                })
            }
            RirStatement::Return { value, .. } => {
                if let Some(expr) = value {
                    let generated_expr = self.generate_expression(expr)?;
                    Ok(quote! {
                        return #generated_expr;
                    })
                } else {
                    Ok(quote! {
                        return;
                    })
                }
            }
            RirStatement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond = self.generate_expression(condition)?;
                let then_body = self.generate_block(then_block)?;

                if let Some(else_blk) = else_block {
                    let else_body = self.generate_block(else_blk)?;
                    Ok(quote! {
                        if #cond {
                            #then_body
                        } else {
                            #else_body
                        }
                    })
                } else {
                    Ok(quote! {
                        if #cond {
                            #then_body
                        }
                    })
                }
            }
            RirStatement::While {
                condition,
                body,
                label,
                ..
            } => self.generate_while(condition, body, label),
            RirStatement::Block { block, .. } => {
                let body = self.generate_block(block)?;
                Ok(quote! {
                    {
                        #body
                    }
                })
            }

            RirStatement::For {
                variable,
                start,
                end,
                inclusive,
                body,
                label,
                ..
            } => self.generate_for(variable, start, end, *inclusive, body, label),

            RirStatement::Loop { body, label, .. } => self.generate_loop(body, label),

            RirStatement::Break { label, value, .. } => self.generate_break(label, value),

            RirStatement::Continue { label, .. } => self.generate_continue(label),

            RirStatement::Match {
                scrutinee, arms, ..
            } => self.generate_match_stmt(scrutinee, arms),
        }
    }

    /// Generates code for a RIR expression.
    #[allow(clippy::only_used_in_recursion)]
    fn generate_expression(&mut self, expr: &RirExpression) -> Result<TokenStream> {
        match expr {
            RirExpression::Unit { .. } => Ok(quote! { () }),
            RirExpression::IntLiteral { value, .. } => {
                let lit = proc_macro2::Literal::i64_unsuffixed(*value);
                Ok(quote! { #lit })
            }
            RirExpression::FloatLiteral { value, .. } => {
                let lit = proc_macro2::Literal::f64_unsuffixed(*value);
                Ok(quote! { #lit })
            }
            RirExpression::StringLiteral { value, .. } => {
                let lit = proc_macro2::Literal::string(value);
                Ok(quote! { #lit.to_string() })
            }
            RirExpression::BoolLiteral { value, .. } => {
                if *value {
                    Ok(quote! { true })
                } else {
                    Ok(quote! { false })
                }
            }
            RirExpression::Variable { name, .. } => {
                let var_name = format_ident!("{}", name);
                Ok(quote! { #var_name })
            }
            RirExpression::Binary {
                op,
                left,
                right,
                result_type,
                ..
            } => {
                // Special handling for string concatenation
                if *op == BinaryOp::Add && *result_type == rive_core::type_system::TypeId::TEXT {
                    let left_expr = self.generate_expression(left)?;
                    let right_expr = self.generate_expression(right)?;
                    return Ok(quote! { format!("{}{}", #left_expr, #right_expr) });
                }

                // Generate left and right expressions with appropriate parentheses
                let left_expr = self.generate_binary_operand(left, op, true)?;
                let right_expr = self.generate_binary_operand(right, op, false)?;

                let operator = match op {
                    BinaryOp::Add => quote! { + },
                    BinaryOp::Subtract => quote! { - },
                    BinaryOp::Multiply => quote! { * },
                    BinaryOp::Divide => quote! { / },
                    BinaryOp::Modulo => quote! { % },
                    BinaryOp::Equal => quote! { == },
                    BinaryOp::NotEqual => quote! { != },
                    BinaryOp::LessThan => quote! { < },
                    BinaryOp::LessEqual => quote! { <= },
                    BinaryOp::GreaterThan => quote! { > },
                    BinaryOp::GreaterEqual => quote! { >= },
                    BinaryOp::And => quote! { && },
                    BinaryOp::Or => quote! { || },
                };

                Ok(quote! { #left_expr #operator #right_expr })
            }
            RirExpression::Unary { op, operand, .. } => {
                let operand_expr = self.generate_expression(operand)?;

                let operator = match op {
                    UnaryOp::Negate => quote! { - },
                    UnaryOp::Not => quote! { ! },
                };

                Ok(quote! { (#operator #operand_expr) })
            }
            RirExpression::Call {
                function,
                arguments,
                ..
            } => {
                // Special handling for print function
                if function == "print" {
                    if arguments.is_empty() {
                        return Err(Error::Codegen(
                            "print() requires at least one argument".to_string(),
                        ));
                    }

                    // Generate println! with multiple arguments
                    let args = arguments
                        .iter()
                        .map(|arg| self.generate_expression(arg))
                        .collect::<Result<Vec<_>>>()?;

                    // Create format string - use {} for strings, {:?} for others
                    let format_parts: Vec<String> = arguments
                        .iter()
                        .map(|arg| {
                            // Check if this is a string type
                            if arg.type_id() == rive_core::type_system::TypeId::TEXT {
                                "{}".to_string()
                            } else {
                                "{:?}".to_string()
                            }
                        })
                        .collect();
                    // Don't add spaces between arguments - let user control formatting
                    let format_str = format_parts.join("");

                    return Ok(quote! {
                        println!(#format_str, #(#args),*)
                    });
                }

                let func_name = format_ident!("{}", function);
                let args = arguments
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect::<Result<Vec<_>>>()?;

                Ok(quote! {
                    #func_name(#(#args),*)
                })
            }
            RirExpression::ArrayLiteral { elements, .. } => {
                let elems = elements
                    .iter()
                    .map(|elem| self.generate_expression(elem))
                    .collect::<Result<Vec<_>>>()?;

                // TODO: Use memory strategy annotation to determine array type
                // For now, always use fixed-size arrays
                Ok(quote! {
                    [#(#elems),*]
                })
            }
            RirExpression::Index { array, index, .. } => {
                let array_expr = self.generate_expression(array)?;
                let index_expr = self.generate_expression(index)?;

                Ok(quote! {
                    #array_expr[#index_expr]
                })
            }

            RirExpression::If {
                condition,
                then_block,
                else_block,
                ..
            } => self.generate_if_expr(condition, then_block, else_block),

            RirExpression::Match {
                scrutinee, arms, ..
            } => self.generate_match_expr(scrutinee, arms),

            RirExpression::Block { block, result, .. } => self.generate_block_expr(block, result),

            RirExpression::While {
                condition,
                body,
                label,
                result_type,
                ..
            } => self.generate_while_expr(condition, body, label, *result_type),

            RirExpression::For {
                variable,
                start,
                end,
                inclusive,
                body,
                label,
                result_type,
                ..
            } => {
                let params = ForExprParams {
                    variable,
                    start,
                    end,
                    inclusive: *inclusive,
                    body,
                    label,
                    result_type: *result_type,
                };
                self.generate_for_expr(params)
            }

            RirExpression::Loop {
                body,
                label,
                result_type,
                ..
            } => self.generate_loop_expr(body, label, *result_type),
        }
    }

    /// Converts a TypeId and MemoryStrategy to a Rust type.
    ///
    /// This method uses the RIR's memory strategy annotations to generate
    /// the appropriate Rust types:
    /// - Copy: Direct value types (i64, f64, bool, String)
    /// - CoW: Copy-on-write types (Rc<String>, Rc<Vec<T>>)
    /// - Unique: Move-only types (not yet fully implemented)
    fn rust_type(&self, type_id: TypeId, strategy: MemoryStrategy) -> Result<TokenStream> {
        match type_id {
            TypeId::INT => Ok(quote! { i64 }),
            TypeId::FLOAT => Ok(quote! { f64 }),
            TypeId::BOOL => Ok(quote! { bool }),
            TypeId::UNIT => Ok(quote! { () }),
            TypeId::TEXT => match strategy {
                MemoryStrategy::Copy => Ok(quote! { String }),
                MemoryStrategy::CoW => Ok(quote! { std::rc::Rc<std::cell::RefCell<String>> }),
                MemoryStrategy::Unique => Ok(quote! { String }),
            },
            _ => {
                // For other types (arrays, custom types), use simple approach for now
                Ok(quote! { () })
            }
        }
    }

    /// Determines if a function should be inlined based on heuristics.
    ///
    /// Inline heuristics:
    /// - Small functions (≤ 5 statements)
    /// - Simple expressions (no complex control flow)
    /// - No recursive calls
    /// - Not the main function (main should not be inlined)
    fn should_inline_function(&self, function: &RirFunction) -> bool {
        // Don't inline main function
        if function.name == "main" {
            return false;
        }

        // Count statements in the function body
        let statement_count = self.count_statements(&function.body);

        // Don't inline if too many statements
        if statement_count > 5 {
            return false;
        }

        // Check for complex control flow (loops, nested blocks)
        if self.has_complex_control_flow(&function.body) {
            return false;
        }

        // Check for recursive calls (simplified check)
        if self.has_recursive_calls(function) {
            return false;
        }

        // Function is suitable for inlining
        true
    }

    /// Counts the number of statements in a block.
    #[allow(clippy::only_used_in_recursion)]
    fn count_statements(&self, block: &RirBlock) -> usize {
        let mut count = block.statements.len();

        // Count statements in nested blocks
        for stmt in &block.statements {
            match stmt {
                RirStatement::Block { block, .. } => {
                    count += self.count_statements(block);
                }
                RirStatement::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    count += self.count_statements(then_block);
                    if let Some(else_block) = else_block {
                        count += self.count_statements(else_block);
                    }
                }
                RirStatement::While { body, .. } => {
                    count += self.count_statements(body);
                }
                RirStatement::For { body, .. } => {
                    count += self.count_statements(body);
                }
                RirStatement::Loop { body, .. } => {
                    count += self.count_statements(body);
                }
                RirStatement::Match { arms, .. } => {
                    for (_, arm_body) in arms {
                        count += self.count_statements(arm_body);
                    }
                }
                _ => {} // Other statements don't contain nested blocks
            }
        }

        count
    }

    /// Checks if a block has complex control flow patterns.
    #[allow(clippy::only_used_in_recursion)]
    fn has_complex_control_flow(&self, block: &RirBlock) -> bool {
        for stmt in &block.statements {
            match stmt {
                RirStatement::Block { block, .. } => {
                    if self.has_complex_control_flow(block) {
                        return true;
                    }
                }
                RirStatement::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    if self.has_complex_control_flow(then_block) {
                        return true;
                    }
                    if let Some(else_block) = else_block
                        && self.has_complex_control_flow(else_block)
                    {
                        return true;
                    }
                }
                RirStatement::While { .. }
                | RirStatement::For { .. }
                | RirStatement::Loop { .. } => {
                    // Loops are always complex
                    return true;
                }
                RirStatement::Match { arms, .. } => {
                    // Match is complex
                    for (_, arm_body) in arms {
                        if self.has_complex_control_flow(arm_body) {
                            return true;
                        }
                    }
                    return true;
                }
                _ => {} // Other statements are simple
            }
        }
        false
    }

    /// Checks if a function contains recursive calls (simplified check).
    fn has_recursive_calls(&self, function: &RirFunction) -> bool {
        self.check_recursive_calls_in_block(&function.body, &function.name)
    }

    /// Recursively checks for recursive calls in a block.
    fn check_recursive_calls_in_block(&self, block: &RirBlock, function_name: &str) -> bool {
        for stmt in &block.statements {
            match stmt {
                RirStatement::Expression { expr, .. } => {
                    if self.check_recursive_calls_in_expr(expr, function_name) {
                        return true;
                    }
                }
                RirStatement::Let { value, .. } => {
                    if self.check_recursive_calls_in_expr(value, function_name) {
                        return true;
                    }
                }
                RirStatement::Assign { value, .. } => {
                    if self.check_recursive_calls_in_expr(value, function_name) {
                        return true;
                    }
                }
                RirStatement::Return { value, .. } => {
                    if let Some(value) = value
                        && self.check_recursive_calls_in_expr(value, function_name)
                    {
                        return true;
                    }
                }
                RirStatement::Block { block, .. } => {
                    if self.check_recursive_calls_in_block(block, function_name) {
                        return true;
                    }
                }
                RirStatement::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    if self.check_recursive_calls_in_block(then_block, function_name) {
                        return true;
                    }
                    if let Some(else_block) = else_block
                        && self.check_recursive_calls_in_block(else_block, function_name)
                    {
                        return true;
                    }
                }
                RirStatement::While { body, .. }
                | RirStatement::For { body, .. }
                | RirStatement::Loop { body, .. } => {
                    if self.check_recursive_calls_in_block(body, function_name) {
                        return true;
                    }
                }
                RirStatement::Match { arms, .. } => {
                    for (_, arm_body) in arms {
                        if self.check_recursive_calls_in_block(arm_body, function_name) {
                            return true;
                        }
                    }
                }
                _ => {} // Other statements don't contain expressions
            }
        }
        false
    }

    /// Checks for recursive calls in an expression.
    #[allow(clippy::only_used_in_recursion)]
    fn check_recursive_calls_in_expr(&self, expr: &RirExpression, function_name: &str) -> bool {
        match expr {
            RirExpression::Call { function, .. } => function == function_name,
            RirExpression::Binary { left, right, .. } => {
                self.check_recursive_calls_in_expr(left, function_name)
                    || self.check_recursive_calls_in_expr(right, function_name)
            }
            RirExpression::Unary { operand, .. } => {
                self.check_recursive_calls_in_expr(operand, function_name)
            }
            RirExpression::ArrayLiteral { elements, .. } => elements
                .iter()
                .any(|elem| self.check_recursive_calls_in_expr(elem, function_name)),
            _ => false, // Other expressions don't contain function calls
        }
    }
}

// Include control flow code generation implementation
include!("codegen_control_flow.rs");

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
