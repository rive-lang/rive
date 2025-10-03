// Control flow code generation for Rive.
//
// This file is included in codegen.rs and implements code generation for control flow constructs:
// - If expressions/statements
// - Loops (while, for, loop)
// - Break and continue statements
// - Match expressions
// - Block expressions

impl CodeGenerator {
    /// Generates code for an if statement.
    #[allow(dead_code)] // Currently RirStatement::If is generated inline
    pub(crate) fn generate_if_stmt(
        &mut self,
        condition: &rive_ir::RirExpression,
        then_block: &rive_ir::RirBlock,
        else_block: &Option<rive_ir::RirBlock>,
    ) -> rive_core::Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let then_body = self.generate_block(then_block)?;

        if let Some(else_blk) = else_block {
            let else_body = self.generate_block(else_blk)?;
            Ok(quote! {
                if #cond {
                    #then_body
                } else {
                    #else_body
                }
            })
        } else {
            Ok(quote! {
                if #cond {
                    #then_body
                }
            })
        }
    }

    /// Generates code for an if expression (must have else).
    pub(crate) fn generate_if_expr(
        &mut self,
        condition: &rive_ir::RirExpression,
        then_block: &rive_ir::RirBlock,
        else_block: &rive_ir::RirBlock,
    ) -> rive_core::Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let then_body = self.generate_block(then_block)?;
        let else_body = self.generate_block(else_block)?;

        Ok(quote! {
            if #cond {
                #then_body
            } else {
                #else_body
            }
        })
    }

    /// Generates code for a while loop.
    pub(crate) fn generate_while(
        &mut self,
        condition: &rive_ir::RirExpression,
        body: &rive_ir::RirBlock,
        label: &Option<String>,
    ) -> rive_core::Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let body_stmts = self.generate_block(body)?;

        if let Some(lbl) = label {
            // Create a lifetime for the label (e.g., 'loop_1)
            let label_lifetime = syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
            Ok(quote! {
                #label_lifetime: while #cond {
                    #body_stmts
                }
            })
        } else {
            Ok(quote! {
                while #cond {
                    #body_stmts
                }
            })
        }
    }

    /// Generates code for a for loop.
    pub(crate) fn generate_for(
        &mut self,
        variable: &str,
        start: &rive_ir::RirExpression,
        end: &rive_ir::RirExpression,
        inclusive: bool,
        body: &rive_ir::RirBlock,
        label: &Option<String>,
    ) -> rive_core::Result<TokenStream> {
        let var = format_ident!("{}", variable);
        let start_expr = self.generate_expression(start)?;
        let end_expr = self.generate_expression(end)?;
        let body_stmts = self.generate_block(body)?;

        let range = if inclusive {
            quote! { #start_expr..=#end_expr }
        } else {
            quote! { #start_expr..#end_expr }
        };

        if let Some(lbl) = label {
            // Create a lifetime for the label (e.g., 'loop_1)
            let label_lifetime = syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
            Ok(quote! {
                #label_lifetime: for #var in #range {
                    #body_stmts
                }
            })
        } else {
            Ok(quote! {
                for #var in #range {
                    #body_stmts
                }
            })
        }
    }

    /// Generates code for an infinite loop.
    pub(crate) fn generate_loop(
        &mut self,
        body: &rive_ir::RirBlock,
        label: &Option<String>,
    ) -> rive_core::Result<TokenStream> {
        let body_stmts = self.generate_block(body)?;

        if let Some(lbl) = label {
            // Create a lifetime for the label (e.g., 'loop_1)
            let label_lifetime = syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
            Ok(quote! {
                #label_lifetime: loop {
                    #body_stmts
                }
            })
        } else {
            Ok(quote! {
                loop {
                    #body_stmts
                }
            })
        }
    }

    /// Generates code for a break statement.
    pub(crate) fn generate_break(
        &mut self,
        label: &Option<String>,
        value: &Option<Box<rive_ir::RirExpression>>,
    ) -> rive_core::Result<TokenStream> {
        if let Some(lbl) = label {
            // Create a lifetime for the label (e.g., 'loop_1)
            let label_lifetime = syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
            if let Some(val_expr) = value {
                let val = self.generate_expression(val_expr)?;
                Ok(quote! { break #label_lifetime #val })
            } else {
                Ok(quote! { break #label_lifetime })
            }
        } else if let Some(val_expr) = value {
            let val = self.generate_expression(val_expr)?;
            Ok(quote! { break #val })
        } else {
            Ok(quote! { break })
        }
    }

    /// Generates code for a continue statement.
    pub(crate) fn generate_continue(
        &mut self,
        label: &Option<String>,
    ) -> rive_core::Result<TokenStream> {
        if let Some(lbl) = label {
            // Create a lifetime for the label (e.g., 'loop_1)
            let label_lifetime = syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
            Ok(quote! { continue #label_lifetime })
        } else {
            Ok(quote! { continue })
        }
    }

    /// Generates code for a match expression.
    pub(crate) fn generate_match_expr(
        &mut self,
        scrutinee: &rive_ir::RirExpression,
        arms: &[(rive_ir::RirPattern, Box<rive_ir::RirExpression>)],
    ) -> rive_core::Result<TokenStream> {
        let val = self.generate_expression(scrutinee)?;

        let match_arms: rive_core::Result<Vec<_>> = arms
            .iter()
            .map(|(pattern, body)| {
                let pat = self.generate_pattern(pattern)?;
                let body_expr = self.generate_expression(body)?;
                // Note: Rive uses -> but Rust uses =>, so we convert here
                Ok(quote! { #pat => #body_expr })
            })
            .collect();

        let match_arms = match_arms?;

        Ok(quote! {
            match #val {
                #(#match_arms),*
            }
        })
    }

    /// Generates code for a match statement.
    pub(crate) fn generate_match_stmt(
        &mut self,
        scrutinee: &rive_ir::RirExpression,
        arms: &[(rive_ir::RirPattern, rive_ir::RirBlock)],
    ) -> rive_core::Result<TokenStream> {
        let val = self.generate_expression(scrutinee)?;

        let match_arms: rive_core::Result<Vec<_>> = arms
            .iter()
            .map(|(pattern, body)| {
                let pat = self.generate_pattern(pattern)?;
                let body_stmts = self.generate_block(body)?;
                // Note: Rive uses -> but Rust uses =>, so we convert here
                Ok(quote! {
                    #pat => {
                        #body_stmts
                    }
                })
            })
            .collect();

        let match_arms = match_arms?;

        Ok(quote! {
            match #val {
                #(#match_arms),*
            }
        })
    }

    /// Generates code for a pattern.
    fn generate_pattern(
        &self,
        pattern: &rive_ir::RirPattern,
    ) -> rive_core::Result<TokenStream> {
        Ok(match pattern {
            rive_ir::RirPattern::IntLiteral { value, .. } => {
                let lit = proc_macro2::Literal::i64_unsuffixed(*value);
                quote! { #lit }
            }
            rive_ir::RirPattern::FloatLiteral { value, .. } => {
                // For float patterns, we need to be careful
                // Rust doesn't allow direct float patterns, so this would need special handling
                // For Phase 1, we'll generate a simple literal and rely on semantic checks
                let lit = proc_macro2::Literal::f64_unsuffixed(*value);
                quote! { #lit }
            }
            rive_ir::RirPattern::StringLiteral { value, .. } => {
                quote! { #value }
            }
            rive_ir::RirPattern::BoolLiteral { value, .. } => {
                quote! { #value }
            }
            rive_ir::RirPattern::Wildcard { .. } => {
                quote! { _ }
            }
        })
    }

    /// Generates code for a block expression.
    pub(crate) fn generate_block_expr(
        &mut self,
        block: &rive_ir::RirBlock,
        result: &Option<Box<rive_ir::RirExpression>>,
    ) -> rive_core::Result<TokenStream> {
        let stmts: rive_core::Result<Vec<_>> = block
            .statements
            .iter()
            .map(|stmt| self.generate_statement(stmt))
            .collect();

        let stmts = stmts?;

        if let Some(result_expr) = result {
            let result_code = self.generate_expression(result_expr)?;
            Ok(quote! {
                {
                    #(#stmts)*
                    #result_code
                }
            })
        } else {
            Ok(quote! {
                {
                    #(#stmts)*
                }
            })
        }
    }
}

