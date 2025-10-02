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

        Ok(quote! {
            fn #name(#(#params),*) #return_type {
                #body
            }
        })
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

    /// Generates code for a RIR block.
    fn generate_block(&mut self, block: &RirBlock) -> Result<TokenStream> {
        let mut statements = Vec::new();

        for stmt in &block.statements {
            statements.push(self.generate_statement(stmt)?);
        }

        // Handle final expression if present
        if let Some(final_expr) = &block.final_expr {
            let expr = self.generate_expression(final_expr)?;
            statements.push(quote! { #expr });
        }

        Ok(quote! {
            #(#statements)*
        })
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
                condition, body, ..
            } => {
                let cond = self.generate_expression(condition)?;
                let loop_body = self.generate_block(body)?;

                Ok(quote! {
                    while #cond {
                        #loop_body
                    }
                })
            }
            RirStatement::Block { block, .. } => {
                let body = self.generate_block(block)?;
                Ok(quote! {
                    {
                        #body
                    }
                })
            }
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
                op, left, right, ..
            } => {
                let left_expr = self.generate_expression(left)?;
                let right_expr = self.generate_expression(right)?;

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

                Ok(quote! { (#left_expr #operator #right_expr) })
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

                    // Create format string with {:?} for each argument (Debug formatting)
                    let format_str = vec!["{:?}"; arguments.len()].join(" ");

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
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
