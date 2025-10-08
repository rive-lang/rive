//! Expression lowering.

use crate::RirExpression;
use crate::lowering::core::AstLowering;
use rive_core::{Error, Result};
use rive_parser::Expression as AstExpression;
use rive_parser::ast::Argument;

impl AstLowering {
    /// Lowers an expression.
    pub(crate) fn lower_expression(&mut self, expr: &AstExpression) -> Result<RirExpression> {
        match expr {
            AstExpression::Integer { value, span } => Ok(RirExpression::IntLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::Float { value, span } => Ok(RirExpression::FloatLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::String { value, span } => Ok(RirExpression::StringLiteral {
                value: value.clone(),
                span: *span,
            }),

            AstExpression::StringInterpolation { parts, span } => {
                // Convert string interpolation to a series of string concatenations
                use rive_parser::ast::StringPart;

                if parts.is_empty() {
                    return Ok(RirExpression::StringLiteral {
                        value: String::new(),
                        span: *span,
                    });
                }

                // Build concatenation expression
                let mut result: Option<RirExpression> = None;

                for part in parts {
                    let part_expr = match part {
                        StringPart::Literal(s) => RirExpression::StringLiteral {
                            value: s.clone(),
                            span: *span,
                        },
                        StringPart::Interpolation(expr) => {
                            // Lower the interpolated expression and convert to string
                            let lowered = self.lower_expression(expr)?;
                            // TODO: Add proper to_string conversion based on type
                            lowered
                        }
                    };

                    result = Some(if let Some(prev) = result {
                        // Concatenate with previous result using + operator
                        RirExpression::Binary {
                            op: crate::BinaryOp::Add,
                            left: Box::new(prev),
                            right: Box::new(part_expr),
                            result_type: rive_core::type_system::TypeId::TEXT,
                            span: *span,
                        }
                    } else {
                        part_expr
                    });
                }

                Ok(result.unwrap())
            }

            AstExpression::Boolean { value, span } => Ok(RirExpression::BoolLiteral {
                value: *value,
                span: *span,
            }),

            AstExpression::Null { span } => Ok(RirExpression::NullLiteral {
                type_id: rive_core::type_system::TypeId::NULL,
                span: *span,
            }),

            AstExpression::Variable { name, span } => {
                // Look up variable type from symbol table
                if let Some(var_info) = self.lookup_variable(name) {
                    Ok(RirExpression::Variable {
                        name: name.clone(),
                        type_id: var_info.type_id,
                        span: *span,
                    })
                } else if let Some(self_info) = self.lookup_variable("self") {
                    // Check if this is a field access on self
                    let self_type = self_info.type_id;
                    let metadata = self.type_registry.get(self_type).cloned();

                    if let Some(meta) = metadata {
                        use rive_core::type_system::TypeKind;
                        if let TypeKind::Struct { fields, .. } = &meta.kind
                            && let Some((_, field_type)) =
                                fields.iter().find(|(field_name, _)| field_name == name)
                        {
                            // This is a field access - convert to self.field
                            let self_expr = RirExpression::Variable {
                                name: "self".to_string(),
                                type_id: self_type,
                                span: *span,
                            };
                            return Ok(RirExpression::FieldAccess {
                                object: Box::new(self_expr),
                                field: name.clone(),
                                result_type: *field_type,
                                span: *span,
                            });
                        }
                    }
                    Err(Error::Semantic(format!("Undefined variable '{name}'")))
                } else {
                    Err(Error::Semantic(format!("Undefined variable '{name}'")))
                }
            }

            AstExpression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_expr = self.lower_expression(left)?;
                let right_expr = self.lower_expression(right)?;
                let op = self.lower_binary_op(operator);
                let result_type = self.infer_binary_result_type(&left_expr, &right_expr, op);

                Ok(RirExpression::Binary {
                    op,
                    left: Box::new(left_expr),
                    right: Box::new(right_expr),
                    result_type,
                    span: *span,
                })
            }

            AstExpression::Unary {
                operator,
                operand,
                span,
            } => {
                let operand_expr = self.lower_expression(operand)?;
                let op = self.lower_unary_op(operator);
                let result_type = operand_expr.type_id();

                Ok(RirExpression::Unary {
                    op,
                    operand: Box::new(operand_expr),
                    result_type,
                    span: *span,
                })
            }

            AstExpression::Call {
                callee,
                arguments,
                span,
            } => {
                let args = self.lower_arguments(arguments)?;

                // Look up function return type from function signatures
                // Special case for built-in print function
                let return_type = if callee == "print" {
                    rive_core::type_system::TypeId::UNIT
                } else {
                    self.lookup_function(callee)
                        .map(|(_, return_type)| *return_type)
                        .ok_or_else(|| Error::Semantic(format!("Undefined function '{callee}'")))?
                };

                Ok(RirExpression::Call {
                    function: callee.clone(),
                    arguments: args,
                    return_type,
                    span: *span,
                })
            }

            AstExpression::Array { elements, span } => {
                let rir_elements = elements
                    .iter()
                    .map(|e| self.lower_expression(e))
                    .collect::<Result<Vec<_>>>()?;

                let element_type = if let Some(first) = rir_elements.first() {
                    first.type_id()
                } else {
                    rive_core::type_system::TypeId::INT // Default for empty arrays
                };

                Ok(RirExpression::ArrayLiteral {
                    elements: rir_elements,
                    element_type,
                    span: *span,
                })
            }

            AstExpression::If(if_expr) => self.lower_if_expr(if_expr),
            AstExpression::While(while_loop) => self.lower_while_expr(while_loop),
            AstExpression::For(for_loop) => self.lower_for_expr(for_loop),
            AstExpression::Loop(loop_expr) => self.lower_loop_expr(loop_expr),
            AstExpression::Match(match_expr) => self.lower_match_expr(match_expr),
            AstExpression::Range(_) => Err(Error::Semantic(
                "Range expressions can only be used in for loops".to_string(),
            )),
            AstExpression::Block(block) => self.lower_block_expr(block),

            // Null safety operators
            AstExpression::Elvis {
                value,
                fallback,
                span,
            } => {
                let value_expr = self.lower_expression(value)?;
                let fallback_expr = self.lower_expression(fallback)?;

                // Determine result type:
                // If value is T?, and fallback is T, result is T
                // If value is T?, and fallback is T?, result is T?
                // If value is T (non-nullable), result is T (redundant but valid)
                let result_type = if let Some(inner) = self.get_nullable_inner(value_expr.type_id())
                {
                    // value is T?
                    if self.get_nullable_inner(fallback_expr.type_id()).is_some() {
                        // fallback is also nullable, result is T?
                        fallback_expr.type_id()
                    } else {
                        // fallback is T, result is T
                        inner
                    }
                } else {
                    // value is non-nullable, result is value's type
                    value_expr.type_id()
                };

                Ok(RirExpression::Elvis {
                    value: Box::new(value_expr),
                    fallback: Box::new(fallback_expr),
                    result_type,
                    span: *span,
                })
            }

            AstExpression::SafeCall { object, call, span } => {
                let object_expr = self.lower_expression(object)?;
                let call_expr = self.lower_expression(call)?;

                // Safe call always returns a nullable type
                // If call returns T, safe call returns T?
                let call_type = call_expr.type_id();
                let result_type = if self.get_nullable_inner(call_type).is_some() {
                    // call already returns T?, keep it
                    call_type
                } else {
                    // call returns T, wrap in T?
                    self.type_registry.create_optional(call_type)
                };

                Ok(RirExpression::SafeCall {
                    object: Box::new(object_expr),
                    call: Box::new(call_expr),
                    result_type,
                    span: *span,
                })
            }

            // New collection literals
            AstExpression::Tuple { elements, span } => {
                let rir_elements = elements
                    .iter()
                    .map(|e| self.lower_expression(e))
                    .collect::<Result<Vec<_>>>()?;

                let element_types: Vec<_> = rir_elements.iter().map(|e| e.type_id()).collect();
                let result_type = self.type_registry.create_tuple(element_types);

                Ok(RirExpression::TupleLiteral {
                    elements: rir_elements,
                    result_type,
                    span: *span,
                })
            }

            AstExpression::List { elements, span } => {
                let rir_elements = elements
                    .iter()
                    .map(|e| self.lower_expression(e))
                    .collect::<Result<Vec<_>>>()?;

                let element_type = if let Some(first) = rir_elements.first() {
                    first.type_id()
                } else {
                    rive_core::type_system::TypeId::UNIT
                };

                let result_type = self.type_registry.create_list(element_type);

                Ok(RirExpression::ListLiteral {
                    elements: rir_elements,
                    result_type,
                    span: *span,
                })
            }

            AstExpression::Dict { entries, span } => {
                let rir_entries = entries
                    .iter()
                    .map(|(key, value)| {
                        let value_expr = self.lower_expression(value)?;
                        Ok((key.clone(), value_expr))
                    })
                    .collect::<Result<Vec<_>>>()?;

                let value_type = if let Some((_, first_value)) = rir_entries.first() {
                    first_value.type_id()
                } else {
                    rive_core::type_system::TypeId::UNIT
                };

                let result_type = self
                    .type_registry
                    .create_map(rive_core::type_system::TypeId::TEXT, value_type);

                Ok(RirExpression::DictLiteral {
                    entries: rir_entries,
                    result_type,
                    span: *span,
                })
            }

            // Method calls and field access
            AstExpression::MethodCall {
                object,
                method,
                arguments,
                span,
            } => {
                let object_expr = self.lower_expression(object)?;
                let object_type = object_expr.type_id();
                let args = self.lower_arguments(arguments)?;

                // Look up method signature
                let method_sig = self.type_registry.get_method(object_type, method);

                if let Some(sig) = method_sig {
                    // Built-in method found
                    let base_return_type = sig.return_type;

                    // For 'get' methods on List and Map, wrap return type in Optional
                    let return_type = if method == "get" {
                        self.type_registry.create_optional(base_return_type)
                    } else {
                        base_return_type
                    };

                    Ok(RirExpression::MethodCall {
                        object: Box::new(object_expr),
                        method: method.clone(),
                        arguments: args,
                        return_type,
                        span: *span,
                    })
                } else {
                    // User-defined method - convert to function call
                    let metadata = self.type_registry.get_type_metadata(object_type);
                    let type_name = metadata.kind.name();

                    if !type_name.is_empty()
                        && object_type.as_u64()
                            >= rive_core::type_system::TypeId::USER_DEFINED_START
                    {
                        // For user-defined types, generate a function call to Type_instance_method
                        let func_name = format!("{}_instance_{}", type_name, method);

                        // Build arguments with object as first argument (self)
                        let mut call_args = vec![object_expr];
                        call_args.extend(args);

                        Ok(RirExpression::Call {
                            function: func_name,
                            arguments: call_args,
                            return_type: rive_core::type_system::TypeId::UNIT, // TODO: get actual return type
                            span: *span,
                        })
                    } else {
                        Err(Error::Semantic(format!(
                            "Method '{}' not found on type",
                            method
                        )))
                    }
                }
            }

            AstExpression::FieldAccess {
                object,
                field,
                span,
            } => {
                let object_expr = self.lower_expression(object)?;
                let object_type = object_expr.type_id();

                // Get field type from tuple or struct
                let result_type = {
                    use rive_core::type_system::TypeKind;
                    let metadata = self.type_registry.get_type_metadata(object_type);
                    if let TypeKind::Tuple { elements } = &metadata.kind {
                        let index: usize = field.parse().map_err(|_| {
                            Error::Semantic(format!("Invalid tuple index '{}'", field))
                        })?;
                        elements.get(index).copied().ok_or_else(|| {
                            Error::Semantic(format!("Tuple index {} out of bounds", index))
                        })?
                    } else if let TypeKind::Struct { fields, .. } = &metadata.kind {
                        // Look up field in struct
                        fields
                            .iter()
                            .find(|(name, _)| name == field)
                            .map(|(_, ty)| *ty)
                            .ok_or_else(|| {
                                Error::Semantic(format!("Field '{}' not found in struct", field))
                            })?
                    } else {
                        return Err(Error::Semantic(
                            "Field access is only supported on tuples and structs".to_string(),
                        ));
                    }
                };

                Ok(RirExpression::FieldAccess {
                    object: Box::new(object_expr),
                    field: field.clone(),
                    result_type,
                    span: *span,
                })
            }

            rive_parser::Expression::ConstructorCall {
                type_name,
                arguments,
                span,
            } => {
                // Lower constructor call - find the type and create initialization
                let type_id = self
                    .type_registry
                    .get_by_name(type_name)
                    .ok_or_else(|| Error::Semantic(format!("Unknown type '{type_name}'")))?;

                let lowered_args = self.lower_arguments(arguments)?;

                Ok(RirExpression::ConstructorCall {
                    type_id,
                    arguments: lowered_args,
                    span: *span,
                })
            }

            rive_parser::Expression::EnumVariant {
                enum_name,
                variant_name,
                arguments,
                span,
            } => {
                // Lower enum variant construction
                let enum_type_id = self
                    .type_registry
                    .get_by_name(enum_name)
                    .ok_or_else(|| Error::Semantic(format!("Unknown enum '{enum_name}'")))?;

                // Verify the variant exists in the enum
                let type_metadata = self.type_registry.get_type_metadata(enum_type_id);
                if let rive_core::type_system::TypeKind::Enum { variants, .. } = &type_metadata.kind
                {
                    if !variants.iter().any(|v| &v.name == variant_name) {
                        return Err(Error::Semantic(format!(
                            "Enum '{enum_name}' has no variant '{variant_name}'"
                        )));
                    }
                } else {
                    return Err(Error::Semantic(format!("'{enum_name}' is not an enum")));
                }

                let lowered_args = self.lower_arguments(arguments)?;

                // Use proper enum variant representation in IR
                Ok(RirExpression::EnumVariant {
                    enum_type_id,
                    variant_name: variant_name.clone(),
                    arguments: lowered_args,
                    span: *span,
                })
            }
        }
    }

    /// Lowers a block expression to RIR.
    pub(crate) fn lower_block_expr(&mut self, block: &rive_parser::Block) -> Result<RirExpression> {
        let rir_block = self.lower_block(block)?;

        // Check if the block has a final expression
        let (result, result_type) = if let Some(ref final_expr) = rir_block.final_expr {
            (rir_block.final_expr.clone(), final_expr.type_id())
        } else {
            (None, rive_core::type_system::TypeId::UNIT)
        };

        Ok(RirExpression::Block {
            block: rir_block,
            result,
            result_type,
            span: block.span,
        })
    }

    /// Helper function to lower arguments (handles both positional and named arguments).
    /// For now, named arguments are converted to positional order based on parameter names.
    pub(crate) fn lower_arguments(&mut self, arguments: &[Argument]) -> Result<Vec<RirExpression>> {
        // Simple lowering - just convert each argument to an expression
        // Reordering is handled by semantic analysis validation
        arguments
            .iter()
            .map(|arg| match arg {
                Argument::Positional(expr) => self.lower_expression(expr),
                Argument::Named { value, .. } => self.lower_expression(value),
            })
            .collect()
    }
}
