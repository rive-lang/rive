//! Core code generation logic.
//!
//! Implements the Automatic Value Semantics (AVS) memory model:
//! - Copy types: Stack-allocated, bitwise copyable (i64, f64, bool)
//! - CoW types: Reference-counted with copy-on-write (String, arrays of non-Copy types)
//! - Unique types: Move-only semantics (future: marked with @unique)

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Error, Result, types::Type};
use rive_parser::{BinaryOperator, Expression, Item, Program, Statement, UnaryOperator};

/// Code generator for Rive programs.
pub struct CodeGenerator {
    /// Counter for generating unique temporary variable names
    temp_counter: usize,
}

impl CodeGenerator {
    /// Creates a new code generator.
    pub fn new() -> Self {
        Self { temp_counter: 0 }
    }

    /// Generates Rust code from a Rive program.
    pub fn generate(&mut self, program: &Program) -> Result<String> {
        let mut items = Vec::new();

        for item in &program.items {
            items.push(self.generate_item(item)?);
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

    /// Generates code for a top-level item.
    fn generate_item(&mut self, item: &Item) -> Result<TokenStream> {
        match item {
            Item::Function(func) => {
                let name = format_ident!("{}", func.name);
                let params = self.generate_parameters(&func.params)?;
                let return_type = self.generate_return_type(&func.return_type);
                let body = self.generate_block(&func.body)?;

                Ok(quote! {
                    fn #name(#(#params),*) #return_type {
                        #body
                    }
                })
            }
        }
    }

    /// Generates function parameters.
    fn generate_parameters(&self, params: &[rive_parser::Parameter]) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|param| {
                let name = format_ident!("{}", param.name);
                let ty = self.rust_type(&param.param_type)?;
                Ok(quote! { #name: #ty })
            })
            .collect()
    }

    /// Generates return type annotation.
    fn generate_return_type(&self, ty: &Type) -> TokenStream {
        if ty.is_unit() {
            quote! {}
        } else {
            let rust_ty = self.rust_type(ty).unwrap();
            // Use explicit token construction to avoid extra spaces
            quote! {-> #rust_ty}
        }
    }

    /// Generates code for a block.
    fn generate_block(&mut self, block: &rive_parser::Block) -> Result<TokenStream> {
        let mut statements = Vec::new();

        for stmt in &block.statements {
            statements.push(self.generate_statement(stmt)?);
        }

        Ok(quote! {
            #(#statements)*
        })
    }

    /// Generates code for a statement.
    fn generate_statement(&mut self, stmt: &Statement) -> Result<TokenStream> {
        match stmt {
            Statement::Let {
                name,
                mutable,
                initializer,
                ..
            } => {
                let var_name = format_ident!("{}", name);
                let expr = self.generate_expression(initializer)?;

                if *mutable {
                    Ok(quote! {
                        let mut #var_name = #expr;
                    })
                } else {
                    Ok(quote! {
                        let #var_name = #expr;
                    })
                }
            }
            Statement::Assignment { name, value, .. } => {
                let var_name = format_ident!("{}", name);
                let expr = self.generate_expression(value)?;

                Ok(quote! {
                    #var_name = #expr;
                })
            }
            Statement::Expression { expression, .. } => {
                let expr = self.generate_expression(expression)?;
                Ok(quote! {
                    #expr;
                })
            }
            Statement::Return { value, .. } => {
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
        }
    }

    /// Generates code for an expression.
    #[allow(clippy::only_used_in_recursion)]
    fn generate_expression(&mut self, expr: &Expression) -> Result<TokenStream> {
        match expr {
            Expression::Integer { value, .. } => {
                let lit = proc_macro2::Literal::i64_unsuffixed(*value);
                Ok(quote! { #lit })
            }
            Expression::Float { value, .. } => {
                let lit = proc_macro2::Literal::f64_unsuffixed(*value);
                Ok(quote! { #lit })
            }
            Expression::String { value, .. } => {
                let lit = proc_macro2::Literal::string(value);
                // TODO: Wrap in Rc<RefCell<>> for CoW when escape analysis is implemented
                Ok(quote! { #lit.to_string() })
            }
            Expression::Boolean { value, .. } => {
                if *value {
                    Ok(quote! { true })
                } else {
                    Ok(quote! { false })
                }
            }
            Expression::Null { .. } => Ok(quote! { None }),
            Expression::Variable { name, .. } => {
                let var_name = format_ident!("{}", name);
                Ok(quote! { #var_name })
            }
            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let left_expr = self.generate_expression(left)?;
                let right_expr = self.generate_expression(right)?;

                let op = match operator {
                    BinaryOperator::Add => quote! { + },
                    BinaryOperator::Subtract => quote! { - },
                    BinaryOperator::Multiply => quote! { * },
                    BinaryOperator::Divide => quote! { / },
                    BinaryOperator::Modulo => quote! { % },
                    BinaryOperator::Equal => quote! { == },
                    BinaryOperator::NotEqual => quote! { != },
                    BinaryOperator::Less => quote! { < },
                    BinaryOperator::LessEqual => quote! { <= },
                    BinaryOperator::Greater => quote! { > },
                    BinaryOperator::GreaterEqual => quote! { >= },
                    BinaryOperator::And => quote! { && },
                    BinaryOperator::Or => quote! { || },
                };

                Ok(quote! { (#left_expr #op #right_expr) })
            }
            Expression::Unary {
                operator, operand, ..
            } => {
                let operand_expr = self.generate_expression(operand)?;

                let op = match operator {
                    UnaryOperator::Negate => quote! { - },
                    UnaryOperator::Not => quote! { ! },
                };

                Ok(quote! { (#op #operand_expr) })
            }
            Expression::Call {
                callee, arguments, ..
            } => {
                // Special handling for print function
                if callee == "print" {
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
                    // This works for all types including Rc<RefCell<T>>
                    let format_str = vec!["{:?}"; arguments.len()].join(" ");

                    return Ok(quote! {
                        println!(#format_str, #(#args),*)
                    });
                }

                let func_name = format_ident!("{}", callee);
                let args = arguments
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect::<Result<Vec<_>>>()?;

                Ok(quote! {
                    #func_name(#(#args),*)
                })
            }
            Expression::Array { elements, .. } => {
                let elems = elements
                    .iter()
                    .map(|elem| self.generate_expression(elem))
                    .collect::<Result<Vec<_>>>()?;

                // Check if all elements are Copy types (literals)
                let all_copy = elements.iter().all(|elem| {
                    matches!(
                        elem,
                        Expression::Integer { .. }
                            | Expression::Float { .. }
                            | Expression::Boolean { .. }
                    )
                });

                if all_copy {
                    // For Copy types, use fixed-size arrays [T; N]
                    // Arrays of Copy types (up to size 32) automatically implement Copy
                    Ok(quote! {
                        [#(#elems),*]
                    })
                } else {
                    // For non-Copy types, use Rc<RefCell<>> for CoW semantics
                    Ok(quote! {
                        std::rc::Rc::new(std::cell::RefCell::new(vec![#(#elems),*]))
                    })
                }
            }
        }
    }

    /// Converts a Rive type to Rust type with AVS memory model.
    ///
    /// This method implements the Automatic Value Semantics (AVS) model:
    /// - Copy types (Int, Float, Bool) => direct Rust types (i64, f64, bool)
    /// - CoW types (Text) => Rc<RefCell<T>> for shared mutable state
    /// - Arrays inherit strategy from their element type
    #[allow(clippy::only_used_in_recursion)]
    fn rust_type(&self, ty: &Type) -> Result<TokenStream> {
        match ty {
            Type::Int => Ok(quote! { i64 }),
            Type::Float => Ok(quote! { f64 }),
            Type::Bool => Ok(quote! { bool }),
            Type::Unit => Ok(quote! { () }),
            Type::Text => {
                // TODO: Implement full CoW strategy with Rc<RefCell<String>>
                // For now, use plain String to simplify code generation
                Ok(quote! { String })
            }
            Type::Optional(inner) => {
                let inner_type = self.rust_type(inner)?;
                Ok(quote! { Option<#inner_type> })
            }
            Type::Array(elem_type, size) => {
                let elem = self.rust_type(elem_type)?;
                let size_lit = proc_macro2::Literal::usize_unsuffixed(*size);

                // Check if element type is Copy
                let elem_is_copy =
                    matches!(elem_type.as_ref(), Type::Int | Type::Float | Type::Bool);

                if elem_is_copy {
                    // For Copy types, use fixed-size arrays [T; N]
                    Ok(quote! { [#elem; #size_lit] })
                } else {
                    // For non-Copy types, use Rc<RefCell<>> for CoW semantics
                    Ok(quote! { std::rc::Rc<std::cell::RefCell<Vec<#elem>>> })
                }
            }
            Type::Function {
                parameters,
                return_type,
            } => {
                let param_types: Vec<_> = parameters
                    .iter()
                    .map(|p| self.rust_type(p))
                    .collect::<Result<Vec<_>>>()?;
                let ret = self.rust_type(return_type)?;
                Ok(quote! { fn(#(#param_types),*) -> #ret })
            }
        }
    }

    /// Generates a unique temporary variable name.
    #[allow(dead_code)]
    fn temp_var(&mut self) -> String {
        let name = format!("_temp_{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
