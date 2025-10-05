//! Label and lifetime generation helpers.

use proc_macro2::TokenStream;
use quote::quote;

/// Generates a Rust label (as lifetime) from an optional label.
pub fn generate_label_lifetime(label: &Option<String>) -> Option<syn::Lifetime> {
    label
        .as_ref()
        .map(|lbl| syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site()))
}

/// Generates a labeled loop statement.
pub fn with_label<F>(label: &Option<String>, generate_body: F) -> TokenStream
where
    F: FnOnce() -> TokenStream,
{
    let body = generate_body();

    if let Some(label_lt) = generate_label_lifetime(label) {
        quote! {
            #label_lt: #body
        }
    } else {
        body
    }
}

/// Generates a break statement with optional label and value.
pub fn generate_break_stmt(label: &Option<String>, value: &Option<TokenStream>) -> TokenStream {
    let label_lt = generate_label_lifetime(label);

    match (label_lt, value) {
        (Some(lbl), Some(val)) => quote! { break #lbl #val },
        (Some(lbl), None) => quote! { break #lbl },
        (None, Some(val)) => quote! { break #val },
        (None, None) => quote! { break },
    }
}

/// Generates a continue statement with optional label.
pub fn generate_continue_stmt(label: &Option<String>) -> TokenStream {
    if let Some(label_lt) = generate_label_lifetime(label) {
        quote! { continue #label_lt }
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

/// Generates a loop (while/for) with optional label using lifetime syntax.
/// This is specifically for while/for loops which need different label handling than loop.
pub fn with_loop_label(label: &Option<String>, loop_code: TokenStream) -> TokenStream {
    if let Some(lbl) = label {
        let label_lifetime =
            syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
        quote! { #label_lifetime: #loop_code }
    } else {
        loop_code
    }
}
