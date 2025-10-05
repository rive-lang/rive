//! Utility functions for code generation.

use proc_macro2::TokenStream;
use quote::quote;
use rive_ir::BinaryOp;

/// Returns the precedence of an operator (higher number = higher precedence).
pub const fn operator_precedence(op: &BinaryOp) -> u8 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::And => 2,
        BinaryOp::Equal | BinaryOp::NotEqual => 3,
        BinaryOp::LessThan
        | BinaryOp::LessEqual
        | BinaryOp::GreaterThan
        | BinaryOp::GreaterEqual => 4,
        BinaryOp::Add | BinaryOp::Subtract => 5,
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 6,
    }
}

/// Returns true if the operator is right-associative.
/// All binary operators in Rive are left-associative.
pub const fn is_right_associative(_op: &BinaryOp) -> bool {
    false
}

/// Converts a binary operator to its Rust token representation.
pub fn binary_op_token(op: &BinaryOp) -> TokenStream {
    match op {
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
    }
}
