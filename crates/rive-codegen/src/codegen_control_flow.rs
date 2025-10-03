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

        // If scrutinee is String type, need to convert to &str for pattern matching
        let match_val = if scrutinee.type_id() == rive_core::type_system::TypeId::TEXT {
            quote! { (#val).as_str() }
        } else {
            val
        };

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
            match #match_val {
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

        // If scrutinee is String type, need to convert to &str for pattern matching
        let match_val = if scrutinee.type_id() == rive_core::type_system::TypeId::TEXT {
            quote! { (#val).as_str() }
        } else {
            val
        };

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
            match #match_val {
                #(#match_arms),*
            }
        })
    }

    /// Generates code for a pattern.
    fn generate_pattern(
        &mut self,
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
                // Create a string literal without escaping quotes
                // In Rust match, we need to match against a string reference
                let lit = proc_macro2::Literal::string(value);
                quote! { #lit }
            }
            rive_ir::RirPattern::BoolLiteral { value, .. } => {
                quote! { #value }
            }
            rive_ir::RirPattern::Wildcard { .. } => {
                quote! { _ }
            }
            rive_ir::RirPattern::RangePattern { start, end, inclusive, .. } => {
                let start_expr = self.generate_expression(start)?;
                let end_expr = self.generate_expression(end)?;
                
                if *inclusive {
                    quote! { #start_expr..=#end_expr }
                } else {
                    quote! { #start_expr..#end_expr }
                }
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

    /// Generates a default value for a given type.
    /// Returns None if the type is Unit (no value needed).
    fn generate_default_value(&self, type_id: rive_core::type_system::TypeId) -> Option<TokenStream> {
        use rive_core::type_system::TypeId;
        
        match type_id {
            TypeId::INT => Some(quote! { 0 }),
            TypeId::FLOAT => Some(quote! { 0.0 }),
            TypeId::TEXT => Some(quote! { String::new() }),
            TypeId::BOOL => Some(quote! { false }),
            TypeId::UNIT => None, // Unit type needs no value
            _ => None, // Default to no value for unknown types
        }
    }

    /// Generates code for a while loop expression.
    /// 
    /// Unlike while statements, while expressions can break with values.
    /// We convert while expressions to loop expressions to support this in Rust.
    pub(crate) fn generate_while_expr(
        &mut self,
        condition: &rive_ir::RirExpression,
        body: &rive_ir::RirBlock,
        label: &Option<String>,
        result_type: rive_core::type_system::TypeId,
    ) -> rive_core::Result<TokenStream> {
        let cond = self.generate_expression(condition)?;
        let body_stmts = self.generate_block(body)?;
        let default_value = self.generate_default_value(result_type);

        if let Some(lbl) = label {
            let label_lifetime = syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
            if let Some(value) = default_value {
                Ok(quote! {
                    #label_lifetime: loop {
                        if !(#cond) {
                            break #label_lifetime #value;
                        }
                        #body_stmts
                    }
                })
            } else {
                Ok(quote! {
                    #label_lifetime: loop {
                        if !(#cond) {
                            break #label_lifetime;
                        }
                        #body_stmts
                    }
                })
            }
        } else {
            if let Some(value) = default_value {
                Ok(quote! {
                    loop {
                        if !(#cond) {
                            break #value;
                        }
                        #body_stmts
                    }
                })
            } else {
                Ok(quote! {
                    loop {
                        if !(#cond) {
                            break;
                        }
                        #body_stmts
                    }
                })
            }
        }
    }

    /// Generates code for a for loop expression.
    /// 
    /// Unlike for statements, for expressions can break with values.
    /// We convert for expressions to loop expressions to support this in Rust.
    /// The range iterator is managed manually.
    pub(crate) fn generate_for_expr(
        &mut self,
        variable: &str,
        start: &rive_ir::RirExpression,
        end: &rive_ir::RirExpression,
        inclusive: bool,
        body: &rive_ir::RirBlock,
        label: &Option<String>,
        result_type: rive_core::type_system::TypeId,
    ) -> rive_core::Result<TokenStream> {
        let var = format_ident!("{}", variable);
        let start_expr = self.generate_expression(start)?;
        let end_expr = self.generate_expression(end)?;
        let body_stmts = self.generate_block(body)?;
        let default_value = self.generate_default_value(result_type);

        let range = if inclusive {
            quote! { (#start_expr..=#end_expr).into_iter() }
        } else {
            quote! { (#start_expr..#end_expr).into_iter() }
        };

        if let Some(lbl) = label {
            let label_lifetime = syn::Lifetime::new(&format!("'{}", lbl), proc_macro2::Span::call_site());
            if let Some(value) = default_value {
                Ok(quote! {
                    {
                        let mut iter = #range;
                        #label_lifetime: loop {
                            let #var = match iter.next() {
                                Some(val) => val,
                                None => break #label_lifetime #value,
                            };
                            #body_stmts
                        }
                    }
                })
            } else {
                Ok(quote! {
                    {
                        let mut iter = #range;
                        #label_lifetime: loop {
                            let #var = match iter.next() {
                                Some(val) => val,
                                None => break #label_lifetime,
                            };
                            #body_stmts
                        }
                    }
                })
            }
        } else {
            if let Some(value) = default_value {
                Ok(quote! {
                    {
                        let mut iter = #range;
                        loop {
                            let #var = match iter.next() {
                                Some(val) => val,
                                None => break #value,
                            };
                            #body_stmts
                        }
                    }
                })
            } else {
                Ok(quote! {
                    {
                        let mut iter = #range;
                        loop {
                            let #var = match iter.next() {
                                Some(val) => val,
                                None => break,
                            };
                            #body_stmts
                        }
                    }
                })
            }
        }
    }

    /// Generates code for an infinite loop expression.
    pub(crate) fn generate_loop_expr(
        &mut self,
        body: &rive_ir::RirBlock,
        label: &Option<String>,
        _result_type: rive_core::type_system::TypeId,
    ) -> rive_core::Result<TokenStream> {
        let body_stmts = self.generate_block(body)?;

        if let Some(lbl) = label {
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
}

