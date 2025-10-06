//! Function call code generation.
//!
//! This module handles:
//! - General function calls
//! - Special `print()` function with custom formatting

use super::super::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Error, Result};
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for a function call.
    ///
    /// Special handling for `print()` function with custom Rive-style formatting.
    pub(crate) fn generate_call(
        &mut self,
        function: &str,
        arguments: &[RirExpression],
    ) -> Result<TokenStream> {
        // Special handling for print function
        if function == "print" {
            return self.generate_print_call(arguments);
        }

        // General function call
        let func_name = format_ident!("{}", function);
        let args = arguments
            .iter()
            .map(|arg| self.generate_expression(arg))
            .collect::<Result<Vec<_>>>()?;

        Ok(quote! { #func_name(#(#args),*) })
    }

    /// Generates code for the special `print()` function.
    ///
    /// This function applies custom Rive formatting:
    /// - Optional types: prints value or "null"
    /// - Lists: prints as `[x, x, x]`
    /// - Maps: prints as `{k: v, k: v}`
    /// - Tuples: prints as `(x, x, x)`
    /// - Strings: prints without quotes
    fn generate_print_call(&mut self, arguments: &[RirExpression]) -> Result<TokenStream> {
        if arguments.is_empty() {
            return Err(Error::Codegen(
                "print() requires at least one argument".to_string(),
            ));
        }

        // Generate println! with custom formatting for special types
        if arguments.len() == 1 {
            let (format_str, exprs) = self.generate_print_format(&arguments[0])?;
            return Ok(quote! {
                println!(#format_str, #(#exprs),*);
            });
        }

        // Multiple arguments - combine format strings
        let mut format_parts = Vec::new();
        let mut all_exprs = Vec::new();

        for arg in arguments {
            let (format_str, exprs) = self.generate_print_format(arg)?;
            format_parts.push(format_str);
            all_exprs.extend(exprs);
        }

        let combined_format = format_parts.join("");

        Ok(quote! {
            println!(#combined_format, #(#all_exprs),*);
        })
    }

    /// Generates custom print format for special types.
    ///
    /// Returns `(format_string, expression_tokens)`.
    ///
    /// # Examples
    /// - Optional: `("null" or format!("{}", value))`
    /// - List: `("[{}, {}]", list[0], list[1])`
    /// - Tuple: `("({}, {})", t.0, t.1)`
    /// - String: `("{}", str)`
    fn generate_print_format(&mut self, arg: &RirExpression) -> Result<(String, Vec<TokenStream>)> {
        use rive_core::type_system::TypeKind;

        let type_id = arg.type_id();
        let expr = self.generate_expression(arg)?;

        // Get type info to determine formatting
        if let Some(type_info) = self.type_registry.get(type_id) {
            match &type_info.kind {
                TypeKind::Optional { .. } => {
                    // Format Optional: match expr { Some(v) => format!("{}", v), None => "null" }
                    let format_expr = quote! {
                        match &#expr {
                            Some(v) => format!("{}", v),
                            None => "null".to_string(),
                        }
                    };
                    return Ok(("{}".to_string(), vec![format_expr]));
                }
                TypeKind::List { .. } => {
                    // Format List: use Debug formatting
                    return Ok(("{:?}".to_string(), vec![quote! { &#expr.borrow() }]));
                }
                TypeKind::Map { .. } => {
                    // Format Map: custom format for readability
                    let format_expr = quote! {
                        {
                            let dict = #expr.borrow();
                            let items: Vec<String> = dict.iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect();
                            format!("{{{}}}", items.join(", "))
                        }
                    };
                    return Ok(("{}".to_string(), vec![format_expr]));
                }
                TypeKind::Tuple { elements } => {
                    // Format Tuple: println!("({}, {})", t.0, t.1)
                    let count = elements.len();
                    let format_str = format!("({})", vec!["{}"; count].join(", "));
                    let indices: Vec<syn::Index> = (0..count).map(syn::Index::from).collect();
                    let field_exprs: Vec<TokenStream> =
                        indices.iter().map(|idx| quote! { #expr.#idx }).collect();
                    return Ok((format_str, field_exprs));
                }
                TypeKind::Text => {
                    // Text: print without quotes
                    return Ok(("{}".to_string(), vec![expr]));
                }
                _ => {}
            }
        }

        // Default: use {} formatting
        Ok(("{}".to_string(), vec![expr]))
    }
}
