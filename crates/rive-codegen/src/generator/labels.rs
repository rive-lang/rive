//! Label and lifetime generation helpers.

use proc_macro2::TokenStream;
use quote::quote;

/// Generates a Rust lifetime from an optional label.
pub fn generate_label_lifetime(label: &Option<String>) -> Option<syn::Lifetime> {
    label
        .as_ref()
        .map(|lbl| syn::Lifetime::new(&format!("'{lbl}"), proc_macro2::Span::call_site()))
}

/// Generates a labeled loop statement.
pub fn with_label<F>(label: &Option<String>, generate_body: F) -> TokenStream
where
    F: FnOnce() -> TokenStream,
{
    let body = generate_body();

    if let Some(label_lifetime) = generate_label_lifetime(label) {
        quote! {
            #label_lifetime: #body
        }
    } else {
        body
    }
}

/// Generates a break statement with optional label and value.
pub fn generate_break_stmt(label: &Option<String>, value: &Option<TokenStream>) -> TokenStream {
    let label_lifetime = generate_label_lifetime(label);

    match (label_lifetime, value) {
        (Some(lbl), Some(val)) => quote! { break #lbl #val },
        (Some(lbl), None) => quote! { break #lbl },
        (None, Some(val)) => quote! { break #val },
        (None, None) => quote! { break },
    }
}

/// Generates a continue statement with optional label.
pub fn generate_continue_stmt(label: &Option<String>) -> TokenStream {
    if let Some(label_lifetime) = generate_label_lifetime(label) {
        quote! { continue #label_lifetime }
    } else {
        quote! { continue }
    }
}

/// Generates a range expression.
pub fn generate_range(start: &TokenStream, end: &TokenStream, inclusive: bool) -> TokenStream {
    if inclusive {
        quote! { #start..=#end }
    } else {
        quote! { #start..#end }
    }
}

/// Generates an iterator from a range.
pub fn generate_range_iterator(
    start: &TokenStream,
    end: &TokenStream,
    inclusive: bool,
) -> TokenStream {
    if inclusive {
        quote! { (#start..=#end).into_iter() }
    } else {
        quote! { (#start..#end).into_iter() }
    }
}
