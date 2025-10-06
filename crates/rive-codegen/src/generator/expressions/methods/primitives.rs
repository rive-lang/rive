//! Primitive type method code generation.
//!
//! Generates code for methods on primitive types:
//! - Int: to_float()
//! - Float: to_int(), is_nan(), is_infinite(), is_finite(), round()
//! - Text: len(), is_empty(), contains(), to_upper(), to_lower(), trim(), replace(), split()

use proc_macro2::TokenStream;
use quote::quote;
use rive_core::{Error, Result, type_system::TypeId};

/// Generates code for a primitive type method call.
pub(super) fn generate(
    object_type: TypeId,
    object_expr: TokenStream,
    method: &str,
    arg_exprs: &[TokenStream],
) -> Result<TokenStream> {
    match (object_type, method) {
        // Int methods
        (TypeId::INT, "to_float") => Ok(quote! { (#object_expr as f64) }),

        // Float methods
        (TypeId::FLOAT, "to_int") => {
            // Returns Int? - truncate toward 0 if finite
            Ok(quote! {
                if #object_expr.is_finite() {
                    let truncated = #object_expr.trunc() as i64;
                    Some(truncated)
                } else {
                    None
                }
            })
        }
        (TypeId::FLOAT, "is_nan") => Ok(quote! { #object_expr.is_nan() }),
        (TypeId::FLOAT, "is_infinite") => Ok(quote! { #object_expr.is_infinite() }),
        (TypeId::FLOAT, "is_finite") => Ok(quote! { #object_expr.is_finite() }),
        (TypeId::FLOAT, "round") => Ok(quote! { #object_expr.round() }),

        // Text methods
        (TypeId::TEXT, "len") => Ok(quote! { (#object_expr.len() as i64) }),
        (TypeId::TEXT, "is_empty") => Ok(quote! { #object_expr.is_empty() }),
        (TypeId::TEXT, "contains") => {
            let pattern = &arg_exprs[0];
            Ok(quote! { #object_expr.contains(&#pattern) })
        }
        (TypeId::TEXT, "to_upper") => Ok(quote! { #object_expr.to_uppercase() }),
        (TypeId::TEXT, "to_lower") => Ok(quote! { #object_expr.to_lowercase() }),
        (TypeId::TEXT, "trim") => Ok(quote! { #object_expr.trim().to_string() }),
        (TypeId::TEXT, "replace") => {
            let from = &arg_exprs[0];
            let to = &arg_exprs[1];
            Ok(quote! { #object_expr.replace(&#from, &#to) })
        }
        (TypeId::TEXT, "split") => {
            let delimiter = &arg_exprs[0];
            Ok(quote! {
                std::rc::Rc::new(std::cell::RefCell::new(
                    #object_expr.split(&#delimiter).map(|s| s.to_string()).collect::<Vec<_>>()
                ))
            })
        }

        _ => Err(Error::Codegen(format!(
            "Method '{}' not implemented for type",
            method
        ))),
    }
}
