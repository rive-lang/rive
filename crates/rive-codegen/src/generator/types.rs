//! Type conversion from Rive types to Rust types.

use proc_macro2::TokenStream;
use quote::quote;
use rive_core::{
    Result,
    type_system::{MemoryStrategy, TypeId},
};

/// Converts a TypeId and MemoryStrategy to a Rust type.
///
/// This method uses the RIR's memory strategy annotations to generate
/// the appropriate Rust types:
/// - Copy: Direct value types (i64, f64, bool, String)
/// - CoW: Copy-on-write types (Rc<String>, Rc<Vec<T>>)
/// - Unique: Move-only types (not yet fully implemented)
pub fn rust_type(type_id: TypeId, strategy: MemoryStrategy) -> Result<TokenStream> {
    match type_id {
        TypeId::INT => Ok(quote! { i64 }),
        TypeId::FLOAT => Ok(quote! { f64 }),
        TypeId::BOOL => Ok(quote! { bool }),
        TypeId::UNIT => Ok(quote! { () }),
        TypeId::TEXT => match strategy {
            MemoryStrategy::Copy => Ok(quote! { String }),
            MemoryStrategy::CoW => Ok(quote! { String }),
            MemoryStrategy::Unique => Ok(quote! { String }),
        },
        _ => Ok(quote! { () }),
    }
}

/// Generates return type annotation.
pub fn generate_return_type(type_id: TypeId) -> TokenStream {
    if type_id == TypeId::UNIT {
        quote! {}
    } else {
        // Return types typically use Copy or CoW strategy, not RcRefCell
        let rust_ty = rust_type(type_id, MemoryStrategy::Copy).unwrap();
        quote! {-> #rust_ty}
    }
}
