//! Pattern matching and match expression code generation.

use super::core::CodeGenerator;
use super::labels;
use proc_macro2::TokenStream;
use quote::quote;
use rive_core::{Result, type_system::TypeId};
use rive_ir::{RirBlock, RirExpression, RirPattern};

impl CodeGenerator {
    /// Generates code for a match expression.
    pub(crate) fn generate_match_expr(
        &mut self,
        scrutinee: &RirExpression,
        arms: &[(RirPattern, Box<RirExpression>)],
    ) -> Result<TokenStream> {
        let val = self.generate_expression(scrutinee)?;
        let match_val = self.prepare_match_value(val, scrutinee.type_id());

        let match_arms: Result<Vec<_>> = arms
            .iter()
            .map(|(pattern, body)| {
                let pat = self.generate_pattern(pattern)?;
                let body_expr = self.generate_expression(body)?;
                Ok(quote! { #pat => #body_expr })
            })
            .collect();

        let match_arms = match_arms?;

        Ok(quote! {
            match #match_val {
                #(#match_arms),*
            }
        })
    }

    /// Generates code for a match statement.
    pub(crate) fn generate_match_stmt(
        &mut self,
        scrutinee: &RirExpression,
        arms: &[(RirPattern, RirBlock)],
    ) -> Result<TokenStream> {
        let val = self.generate_expression(scrutinee)?;
        let match_val = self.prepare_match_value(val, scrutinee.type_id());

        let match_arms: Result<Vec<_>> = arms
            .iter()
            .map(|(pattern, body)| {
                let pat = self.generate_pattern(pattern)?;
                let body_stmts = self.generate_block(body)?;
                Ok(quote! {
                    #pat => {
                        #body_stmts
                    }
                })
            })
            .collect();

        let match_arms = match_arms?;

        Ok(quote! {
            match #match_val {
                #(#match_arms),*
            }
        })
    }

    /// Prepares a value for matching (converts Text to &str).
    pub(crate) fn prepare_match_value(&self, val: TokenStream, type_id: TypeId) -> TokenStream {
        if type_id == TypeId::TEXT {
            quote! { (#val).as_str() }
        } else {
            val
        }
    }

    /// Generates code for a pattern.
    pub(crate) fn generate_pattern(&mut self, pattern: &RirPattern) -> Result<TokenStream> {
        Ok(match pattern {
            RirPattern::IntLiteral { value, .. } => {
                let lit = proc_macro2::Literal::i64_unsuffixed(*value);
                quote! { #lit }
            }
            RirPattern::FloatLiteral { value, .. } => {
                let lit = proc_macro2::Literal::f64_unsuffixed(*value);
                quote! { #lit }
            }
            RirPattern::StringLiteral { value, .. } => {
                let lit = proc_macro2::Literal::string(value);
                quote! { #lit }
            }
            RirPattern::BoolLiteral { value, .. } => {
                quote! { #value }
            }
            RirPattern::Wildcard { .. } => {
                quote! { _ }
            }
            RirPattern::RangePattern {
                start,
                end,
                inclusive,
                ..
            } => {
                let start_expr = self.generate_expression(start)?;
                let end_expr = self.generate_expression(end)?;
                labels::generate_range(&start_expr, &end_expr, *inclusive)
            }
            RirPattern::EnumVariant {
                enum_type_id,
                variant_name,
                bindings,
                ..
            } => {
                use quote::format_ident;

                // Get enum metadata
                let enum_metadata = self.type_registry.get(*enum_type_id).ok_or_else(|| {
                    rive_core::Error::Codegen(format!(
                        "Enum type {:?} not found in registry",
                        enum_type_id
                    ))
                })?;

                let enum_name = enum_metadata.kind.name();
                let enum_ident = format_ident!("{}", enum_name);
                let variant_ident = format_ident!("{}", variant_name);

                if let Some(bindings) = bindings {
                    // Variant with fields - generate destructuring pattern
                    let field_patterns: Vec<TokenStream> = bindings
                        .iter()
                        .map(|(field_name, binding_name)| {
                            let field_ident = format_ident!("{}", field_name);
                            let binding_ident = format_ident!("{}", binding_name);
                            quote! { #field_ident: #binding_ident }
                        })
                        .collect();

                    quote! {
                        #enum_ident::#variant_ident { #(#field_patterns),* }
                    }
                } else {
                    // Variant without fields
                    quote! {
                        #enum_ident::#variant_ident
                    }
                }
            }
        })
    }
}
