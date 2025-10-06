//! List method code generation.
//!
//! Generates code for List<T> methods:
//! - len() → i64
//! - is_empty() → bool
//! - get(index: Int) → T?
//! - append(value: T) → Unit
//! - insert(index: Int, value: T) → Unit
//! - remove(index: Int) → Unit
//! - clear() → Unit
//! - reverse() → Unit
//! - contains(value: T) → bool
//! - sort() → Unit

use proc_macro2::TokenStream;
use quote::quote;
use rive_core::{Error, Result};

/// Generates code for a list method call.
///
/// Lists are represented as `Rc<RefCell<Vec<T>>>`.
pub(super) fn generate(
    object_expr: TokenStream,
    method: &str,
    arg_exprs: &[TokenStream],
) -> Result<TokenStream> {
    match method {
        "len" => Ok(quote! { (#object_expr.borrow().len() as i64) }),

        "is_empty" => Ok(quote! { #object_expr.borrow().is_empty() }),

        "get" => {
            let index = &arg_exprs[0];
            Ok(quote! {
                {
                    let idx = #index as usize;
                    #object_expr.borrow().get(idx).cloned()
                }
            })
        }

        "append" => {
            let value = &arg_exprs[0];
            Ok(quote! { #object_expr.borrow_mut().push(#value) })
        }

        "insert" => {
            let index = &arg_exprs[0];
            let value = &arg_exprs[1];
            Ok(quote! {
                {
                    let idx = #index as usize;
                    if idx <= #object_expr.borrow().len() {
                        #object_expr.borrow_mut().insert(idx, #value);
                    }
                }
            })
        }

        "remove" => {
            let index = &arg_exprs[0];
            Ok(quote! {
                {
                    let idx = #index as usize;
                    if idx < #object_expr.borrow().len() {
                        #object_expr.borrow_mut().remove(idx);
                    }
                }
            })
        }

        "clear" => Ok(quote! { #object_expr.borrow_mut().clear() }),

        "reverse" => Ok(quote! { #object_expr.borrow_mut().reverse() }),

        "contains" => {
            let value = &arg_exprs[0];
            Ok(quote! { #object_expr.borrow().contains(&#value) })
        }

        "sort" => Ok(quote! { #object_expr.borrow_mut().sort() }),

        _ => Err(Error::Codegen(format!("Unknown list method: {}", method))),
    }
}
