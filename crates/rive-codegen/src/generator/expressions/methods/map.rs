//! Map method code generation.
//!
//! Generates code for Map<K, V> methods:
//! - len() → i64
//! - is_empty() → bool
//! - get(key: Text) → V?
//! - insert(key: Text, value: V) → Unit
//! - remove(key: Text) → Unit
//! - contains_key(key: Text) → bool
//! - keys() → List<Text>
//! - values() → List<V>

use proc_macro2::TokenStream;
use quote::quote;
use rive_core::{Error, Result};

/// Generates code for a map method call.
///
/// Maps are represented as `Rc<RefCell<HashMap<String, V>>>`.
pub(super) fn generate(
    object_expr: TokenStream,
    method: &str,
    arg_exprs: &[TokenStream],
) -> Result<TokenStream> {
    match method {
        "len" => Ok(quote! { (#object_expr.borrow().len() as i64) }),

        "is_empty" => Ok(quote! { #object_expr.borrow().is_empty() }),

        "get" => {
            let key = &arg_exprs[0];
            Ok(quote! { #object_expr.borrow().get(#key).cloned() })
        }

        "insert" => {
            let key = &arg_exprs[0];
            let value = &arg_exprs[1];
            Ok(quote! { #object_expr.borrow_mut().insert(#key, #value) })
        }

        "remove" => {
            let key = &arg_exprs[0];
            Ok(quote! { #object_expr.borrow_mut().remove(#key) })
        }

        "contains_key" => {
            let key = &arg_exprs[0];
            Ok(quote! { #object_expr.borrow().contains_key(#key) })
        }

        "keys" => Ok(quote! {
            std::rc::Rc::new(std::cell::RefCell::new(
                #object_expr.borrow().keys().cloned().collect::<Vec<_>>()
            ))
        }),

        "values" => Ok(quote! {
            std::rc::Rc::new(std::cell::RefCell::new(
                #object_expr.borrow().values().cloned().collect::<Vec<_>>()
            ))
        }),

        _ => Err(Error::Codegen(format!("Unknown map method: {}", method))),
    }
}
