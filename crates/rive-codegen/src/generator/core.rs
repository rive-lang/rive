//! Core code generator implementation.

use super::{inline, types};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rive_core::{Error, Result, type_system::TypeId};
use rive_ir::{RirBlock, RirFunction, RirModule};

/// Loop context for tracking result variables in loop expressions.
#[derive(Debug, Clone)]
struct LoopContext {
    /// Name of the result variable for this loop (e.g., "__for_result")
    result_var: String,
}

/// Code generator for Rive programs.
pub struct CodeGenerator {
    /// Stack of loop contexts for rewriting break statements
    loop_stack: Vec<LoopContext>,
    /// Type registry for type lookups during codegen
    pub(crate) type_registry: rive_core::type_system::TypeRegistry,
}

impl CodeGenerator {
    /// Creates a new code generator.
    pub fn new() -> Self {
        Self {
            loop_stack: Vec::new(),
            type_registry: rive_core::type_system::TypeRegistry::new(),
        }
    }

    /// Enters a loop context with a result variable.
    pub(crate) fn enter_loop_context(&mut self, result_var: Option<String>) {
        if let Some(var) = result_var {
            self.loop_stack.push(LoopContext { result_var: var });
        }
    }

    /// Exits the current loop context.
    pub(crate) fn exit_loop_context(&mut self) {
        self.loop_stack.pop();
    }

    /// Gets the current loop's result variable name, if in a loop context.
    pub(crate) fn current_loop_result_var(&self) -> Option<&str> {
        self.loop_stack.last().map(|ctx| ctx.result_var.as_str())
    }

    /// Generates code for a constructor call.
    pub(crate) fn generate_constructor_call(
        &mut self,
        type_id: rive_core::type_system::TypeId,
        arguments: &[rive_ir::RirExpression],
    ) -> Result<TokenStream> {
        let type_meta = self
            .type_registry
            .get(type_id)
            .ok_or_else(|| Error::Codegen(format!("Type {:?} not found in registry", type_id)))?;

        // Extract the type name
        let type_name = type_meta.kind.name();
        if type_name.is_empty() {
            return Err(Error::Codegen(format!(
                "Cannot construct anonymous type {:?}",
                type_id
            )));
        }

        let type_ident = format_ident!("{}", type_name);

        // Generate field initialization from arguments
        use rive_core::type_system::TypeKind;
        let fields = match &type_meta.kind {
            TypeKind::Struct { fields, .. } => fields.clone(),
            TypeKind::Enum { .. } => {
                // For enums, we'll generate a simple placeholder for now
                // TODO: Implement proper enum codegen with Rust enum types
                Vec::new()
            }
            _ => {
                return Err(Error::Codegen(format!(
                    "Type '{}' is not constructible",
                    type_name
                )));
            }
        };

        // For enums, generate a placeholder unit value for now
        if matches!(type_meta.kind, TypeKind::Enum { .. }) {
            // TODO: Implement proper enum variant construction
            return Ok(quote! { () });
        }

        // Generate argument expressions first
        let arg_exprs: Vec<TokenStream> = arguments
            .iter()
            .map(|arg| self.generate_expression(arg))
            .collect::<Result<Vec<_>>>()?;

        // Build field initializations
        let field_inits: Vec<TokenStream> = fields
            .iter()
            .zip(arg_exprs.iter())
            .map(|((field_name, _), arg)| {
                let field = format_ident!("{}", field_name);
                quote! { #field: #arg }
            })
            .collect();

        Ok(quote! {
            #type_ident { #(#field_inits),* }
        })
    }

    /// Generates code for an enum variant construction.
    pub(crate) fn generate_enum_variant(
        &mut self,
        enum_type_id: rive_core::type_system::TypeId,
        variant_name: &str,
        arguments: &[rive_ir::RirExpression],
    ) -> Result<TokenStream> {
        use rive_core::type_system::TypeKind;

        let type_meta = self.type_registry.get(enum_type_id).ok_or_else(|| {
            Error::Codegen(format!(
                "Enum type {:?} not found in registry",
                enum_type_id
            ))
        })?;

        // Extract the enum name
        let enum_name = type_meta.kind.name();
        if enum_name.is_empty() {
            return Err(Error::Codegen(format!(
                "Cannot construct anonymous enum {:?}",
                enum_type_id
            )));
        }

        let enum_ident = format_ident!("{}", enum_name);
        let variant_ident = format_ident!("{}", variant_name);

        // Get the variant definition and clone the fields
        let variant_fields = match &type_meta.kind {
            TypeKind::Enum { variants, .. } => {
                let variant = variants
                    .iter()
                    .find(|v| v.name == variant_name)
                    .ok_or_else(|| {
                        Error::Codegen(format!(
                            "Variant '{}' not found in enum '{}'",
                            variant_name, enum_name
                        ))
                    })?;
                variant.fields.clone()
            }
            _ => {
                return Err(Error::Codegen(format!(
                    "Type '{}' is not an enum",
                    enum_name
                )));
            }
        };

        // Generate argument expressions
        let arg_exprs: Vec<TokenStream> = arguments
            .iter()
            .map(|arg| self.generate_expression(arg))
            .collect::<Result<Vec<_>>>()?;

        // Build the enum variant construction
        if let Some(fields) = &variant_fields {
            // Variant with named fields
            let field_inits: Vec<TokenStream> = fields
                .iter()
                .zip(arg_exprs.iter())
                .map(|((field_name, _), arg)| {
                    let field = format_ident!("{}", field_name);
                    quote! { #field: #arg }
                })
                .collect();

            Ok(quote! {
                #enum_ident::#variant_ident { #(#field_inits),* }
            })
        } else {
            // Variant without fields
            Ok(quote! {
                #enum_ident::#variant_ident
            })
        }
    }

    /// Generates a loop (for/while/loop) as a statement (no return value).
    pub(crate) fn generate_loop_stmt(
        &mut self,
        expr: &rive_ir::RirExpression,
    ) -> Result<TokenStream> {
        use crate::generator::control_flow::ForLoopParams;

        match expr {
            rive_ir::RirExpression::For {
                variable,
                start,
                end,
                inclusive,
                body,
                label,
                ..
            } => {
                let params = ForLoopParams {
                    variable,
                    start,
                    end,
                    inclusive: *inclusive,
                    body,
                    label,
                };
                self.generate_for(params)
            }
            rive_ir::RirExpression::While {
                condition,
                body,
                label,
                ..
            } => self.generate_while(condition, body, label),
            rive_ir::RirExpression::Loop { body, label, .. } => self.generate_loop(body, label),
            _ => unreachable!("generate_loop_stmt called on non-loop expression"),
        }
    }

    /// Generates Rust code from a RIR module.
    pub fn generate(&mut self, module: &RirModule) -> Result<String> {
        // Copy the type registry from the module
        self.type_registry = module.type_registry.clone();

        // Generate struct definitions for user-defined types
        let mut struct_defs = Vec::new();
        for (type_id, metadata) in &module.type_registry.types {
            if type_id.as_u64() >= rive_core::type_system::TypeId::USER_DEFINED_START
                && let Some(struct_def) = self.generate_struct_definition(metadata)?
            {
                struct_defs.push(struct_def);
            }
        }

        // Group methods by type for impl blocks
        let mut type_methods: std::collections::HashMap<String, Vec<&RirFunction>> =
            std::collections::HashMap::new();
        let mut standalone_functions = Vec::new();

        for function in &module.functions {
            // Check if this is a method (instance or static)
            let is_instance_method = function
                .parameters
                .first()
                .is_some_and(|p| p.name == "self");
            let is_static_method = function.name.contains("_") && !is_instance_method;

            if is_instance_method {
                // Extract type name from "TypeName_instance_methodName"
                if let Some(type_name) = function.name.split("_instance_").next() {
                    type_methods
                        .entry(type_name.to_string())
                        .or_default()
                        .push(function);
                } else {
                    standalone_functions.push(function);
                }
            } else if is_static_method && function.name.contains("_") {
                // Extract type name from "TypeName_methodName"
                if let Some(underscore_pos) = function.name.find('_') {
                    let type_name = &function.name[..underscore_pos];
                    // Verify this is actually a user-defined type
                    if self.type_registry.get_by_name(type_name).is_some() {
                        type_methods
                            .entry(type_name.to_string())
                            .or_default()
                            .push(function);
                    } else {
                        standalone_functions.push(function);
                    }
                } else {
                    standalone_functions.push(function);
                }
            } else {
                standalone_functions.push(function);
            }
        }

        // Generate impl blocks
        let mut impl_blocks = Vec::new();
        for (type_name, methods) in type_methods {
            let type_ident = format_ident!("{}", type_name);
            let method_defs: Result<Vec<_>> = methods
                .iter()
                .map(|func| self.generate_method_definition(func))
                .collect();
            let method_defs = method_defs?;

            impl_blocks.push(quote! {
                impl #type_ident {
                    #(#method_defs)*
                }
            });
        }

        // Generate standalone functions
        let standalone_items: Result<Vec<_>> = standalone_functions
            .iter()
            .map(|function| self.generate_standalone_function(function))
            .collect();
        let standalone_items = standalone_items?;

        let tokens = quote! {
            #(#struct_defs)*
            #(#impl_blocks)*
            #(#standalone_items)*
        };

        let syntax_tree = syn::parse2::<syn::File>(tokens)
            .map_err(|e| Error::Codegen(format!("Failed to parse generated code: {e}")))?;

        Ok(prettyplease::unparse(&syntax_tree))
    }

    /// Generates a return type with registry access for user-defined types.
    fn generate_return_type_with_registry(&self, type_id: TypeId) -> TokenStream {
        if type_id == TypeId::UNIT {
            quote! {}
        } else if type_id.as_u64() >= TypeId::USER_DEFINED_START {
            // User-defined type
            if let Some(metadata) = self.type_registry.get(type_id) {
                let type_name = metadata.kind.name();
                let type_ident = format_ident!("{}", type_name);
                quote! {-> #type_ident}
            } else {
                eprintln!(
                    "Warning: User-defined type {:?} not found in registry",
                    type_id
                );
                quote! {}
            }
        } else {
            types::generate_return_type(type_id)
        }
    }

    /// Generates a method definition (without the impl block wrapper).
    fn generate_method_definition(&mut self, function: &RirFunction) -> Result<TokenStream> {
        let is_instance_method = function
            .parameters
            .first()
            .is_some_and(|p| p.name == "self");

        if is_instance_method {
            // Instance method
            let other_params = if function.parameters.len() > 1 {
                self.generate_parameters(&function.parameters[1..])?
            } else {
                vec![]
            };

            let return_type = self.generate_return_type_with_registry(function.return_type);
            let body = self.generate_block(&function.body)?;

            // Extract method name from "TypeName_instance_methodName"
            let method_name = function
                .name
                .split("_instance_")
                .last()
                .unwrap_or(&function.name);
            let method_ident = format_ident!("{}", method_name);

            if inline::should_inline_function(function) {
                Ok(quote! {
                    #[inline]
                    fn #method_ident(&self, #(#other_params),*) #return_type {
                        #body
                    }
                })
            } else {
                Ok(quote! {
                    fn #method_ident(&self, #(#other_params),*) #return_type {
                        #body
                    }
                })
            }
        } else {
            // Static method
            let params = self.generate_parameters(&function.parameters)?;
            let return_type = self.generate_return_type_with_registry(function.return_type);
            let body = self.generate_block(&function.body)?;

            // Extract method name from "TypeName_methodName"
            let method_name = function
                .name
                .splitn(2, '_')
                .last()
                .unwrap_or(&function.name);
            let method_ident = format_ident!("{}", method_name);

            if inline::should_inline_function(function) {
                Ok(quote! {
                    #[inline]
                    fn #method_ident(#(#params),*) #return_type {
                        #body
                    }
                })
            } else {
                Ok(quote! {
                    fn #method_ident(#(#params),*) #return_type {
                        #body
                    }
                })
            }
        }
    }

    /// Generates a standalone function (not part of any type).
    fn generate_standalone_function(&mut self, function: &RirFunction) -> Result<TokenStream> {
        let name = format_ident!("{}", function.name);
        let params = self.generate_parameters(&function.parameters)?;
        let return_type = self.generate_return_type_with_registry(function.return_type);
        let body = self.generate_block(&function.body)?;

        if inline::should_inline_function(function) {
            Ok(quote! {
                #[inline]
                fn #name(#(#params),*) #return_type {
                    #body
                }
            })
        } else {
            Ok(quote! {
                fn #name(#(#params),*) #return_type {
                    #body
                }
            })
        }
    }

    /// Generates a struct definition from type metadata.
    fn generate_struct_definition(
        &self,
        metadata: &rive_core::type_system::TypeMetadata,
    ) -> Result<Option<TokenStream>> {
        use rive_core::type_system::TypeKind;

        match &metadata.kind {
            TypeKind::Struct { name, fields } => {
                let struct_name = format_ident!("{}", name);
                let field_defs: Vec<TokenStream> = fields
                    .iter()
                    .map(|(field_name, field_type)| {
                        let field_ident = format_ident!("{}", field_name);
                        let field_type_str =
                            types::rust_type(*field_type, metadata.memory_strategy)?;
                        Ok(quote! { #field_ident: #field_type_str })
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(Some(quote! {
                    #[derive(Debug, Clone, PartialEq)]
                    struct #struct_name {
                        #(#field_defs),*
                    }
                }))
            }
            TypeKind::Enum { name, variants } => {
                let enum_name = format_ident!("{}", name);
                let variant_defs: Vec<TokenStream> = variants
                    .iter()
                    .map(|variant| {
                        let variant_ident = format_ident!("{}", variant.name);
                        if let Some(fields) = &variant.fields {
                            // Variant with named fields
                            let field_defs: Vec<TokenStream> = fields
                                .iter()
                                .map(|(field_name, field_type)| {
                                    let field_ident = format_ident!("{}", field_name);
                                    let field_type_str =
                                        types::rust_type(*field_type, metadata.memory_strategy)?;
                                    Ok(quote! { #field_ident: #field_type_str })
                                })
                                .collect::<Result<Vec<_>>>()?;
                            Ok(quote! { #variant_ident { #(#field_defs),* } })
                        } else {
                            // Variant without fields
                            Ok(quote! { #variant_ident })
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(Some(quote! {
                    #[derive(Debug, Clone, PartialEq)]
                    enum #enum_name {
                        #(#variant_defs),*
                    }
                }))
            }
            _ => Ok(None),
        }
    }

    /// Generates function parameters.
    pub(crate) fn generate_parameters(
        &self,
        params: &[rive_ir::RirParameter],
    ) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|param| {
                let name = format_ident!("{}", param.name);
                // For 'self' parameter, use the actual type from registry
                let ty = if param.name == "self" {
                    let type_meta = self.type_registry.get(param.type_id).ok_or_else(|| {
                        Error::Codegen(format!("Type {:?} not found in registry", param.type_id))
                    })?;
                    let type_name = type_meta.kind.name();
                    let type_ident = format_ident!("{}", type_name);
                    quote! { #type_ident }
                } else {
                    types::rust_type(param.type_id, param.memory_strategy)?
                };
                Ok(quote! { #name: #ty })
            })
            .collect()
    }

    /// Generates code for a RIR block.
    pub(crate) fn generate_block(&mut self, block: &RirBlock) -> Result<TokenStream> {
        let statements: Result<Vec<_>> = block
            .statements
            .iter()
            .map(|stmt| self.generate_statement(stmt))
            .collect();

        let statements = statements?;

        if let Some(final_expr) = &block.final_expr {
            // Special case: if final_expr is a loop without break value, treat it as a statement
            // This prevents generating { let __result = None; for ... {} __result } wrapper
            if final_expr.is_loop() {
                let loop_stmt = self.generate_loop_stmt(final_expr)?;
                Ok(quote! {
                    #(#statements)*
                    #loop_stmt
                })
            } else {
                let expr = self.generate_expression(final_expr)?;
                Ok(quote! {
                    #(#statements)*
                    #expr
                })
            }
        } else {
            Ok(quote! {
                #(#statements)*
            })
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
