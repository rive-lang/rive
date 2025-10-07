//! Function call and array literal type checking.

use crate::checker::core::TypeChecker;
use rive_core::type_system::TypeId;
use rive_core::{Error, Result, Span};
use rive_parser::ast::Expression;

impl TypeChecker {
    /// Checks a function call.
    pub(super) fn check_call(
        &mut self,
        callee: &str,
        arguments: &[Expression],
        span: Span,
    ) -> Result<TypeId> {
        // Special handling for built-in print function
        if callee == "print" {
            if arguments.is_empty() {
                return Err(Error::SemanticWithSpan(
                    "print requires at least one argument".to_string(),
                    span,
                ));
            }
            // Check all arguments
            for arg in arguments {
                self.check_expression(arg)?;
            }
            return Ok(TypeId::UNIT);
        }

        // Look up function symbol
        let func_type_id = self
            .symbols
            .lookup(callee)
            .ok_or_else(|| Error::SemanticWithSpan(format!("Undefined function '{callee}'"), span))?
            .symbol_type;

        // Extract function type
        let (param_types, return_type) = {
            let registry = self.symbols.type_registry();
            let metadata = registry.get_type_metadata(func_type_id);

            use rive_core::type_system::TypeKind;
            if let TypeKind::Function {
                parameters,
                return_type,
            } = &metadata.kind
            {
                (parameters.clone(), *return_type)
            } else {
                return Err(Error::SemanticWithSpan(
                    format!("'{callee}' is not a function"),
                    span,
                ));
            }
        };

        // Check argument count
        if arguments.len() != param_types.len() {
            return Err(Error::SemanticWithSpan(
                format!(
                    "Function '{callee}' expects {} arguments, but {} were provided",
                    param_types.len(),
                    arguments.len()
                ),
                span,
            ));
        }

        // Clone arguments to avoid borrow issues during checking
        let args_clone = arguments.to_vec();

        // Check argument types
        for (i, (expected_type, arg)) in param_types.iter().zip(args_clone.iter()).enumerate() {
            let arg_type = self.check_expression(arg)?;
            // Check if arg_type can be assigned to expected_type
            if !self.types_compatible(*expected_type, arg_type) {
                return Err(self.type_mismatch_error(
                    &format!("Argument {} type mismatch", i + 1),
                    *expected_type,
                    arg_type,
                    span,
                ));
            }
        }

        Ok(return_type)
    }

    /// Checks a constructor call.
    pub(super) fn check_constructor_call(
        &mut self,
        type_name: &str,
        arguments: &[Expression],
        span: Span,
    ) -> Result<TypeId> {
        // Look up the type
        let type_id = self
            .symbols
            .type_registry()
            .get_by_name(type_name)
            .ok_or_else(|| {
                Error::SemanticWithSpan(format!("Unknown type '{type_name}'"), span)
            })?;

        // Get type metadata
        let metadata = self.symbols.type_registry().get_type_metadata(type_id).clone();

        // Extract constructor parameter types
        use rive_core::type_system::TypeKind;
        let param_types = if let TypeKind::Struct { fields, .. } = &metadata.kind {
            fields.iter().map(|(_, t)| *t).collect::<Vec<_>>()
        } else {
            return Err(Error::SemanticWithSpan(
                format!("Type '{type_name}' is not constructible"),
                span,
            ));
        };

        // Check argument count
        if arguments.len() != param_types.len() {
            return Err(Error::SemanticWithSpan(
                format!(
                    "Constructor for '{type_name}' expects {} arguments, but {} were provided",
                    param_types.len(),
                    arguments.len()
                ),
                span,
            ));
        }

        // Check argument types
        for (i, (expected_type, arg)) in param_types.iter().zip(arguments.iter()).enumerate() {
            let arg_type = self.check_expression(arg)?;
            if !self.types_compatible(*expected_type, arg_type) {
                return Err(self.type_mismatch_error(
                    &format!("Constructor argument {} type mismatch", i + 1),
                    *expected_type,
                    arg_type,
                    span,
                ));
            }
        }

        Ok(type_id)
    }

    /// Checks an array literal.
    pub(super) fn check_array(&mut self, elements: &[Expression], span: Span) -> Result<TypeId> {
        if elements.is_empty() {
            return Err(Error::SemanticWithSpan(
                "Empty arrays are not supported".to_string(),
                span,
            ));
        }

        // Check all elements have the same type
        let first_type = self.check_expression(&elements[0])?;
        for (i, elem) in elements.iter().enumerate().skip(1) {
            let elem_type = self.check_expression(elem)?;
            if !self.types_compatible(elem_type, first_type) {
                return Err(self.type_mismatch_error(
                    &format!("Array element at index {i} type mismatch"),
                    first_type,
                    elem_type,
                    span,
                ));
            }
        }

        // Create array type
        let array_type = self
            .symbols
            .type_registry_mut()
            .create_array(first_type, elements.len());
        Ok(array_type)
    }

    /// Checks a tuple literal.
    pub(super) fn check_tuple(&mut self, elements: &[Expression], _span: Span) -> Result<TypeId> {
        // Empty tuple is Unit type
        if elements.is_empty() {
            return Ok(TypeId::UNIT);
        }

        // Check all element types
        let mut element_types = Vec::new();
        for elem in elements {
            let elem_type = self.check_expression(elem)?;
            element_types.push(elem_type);
        }

        // Create tuple type
        let tuple_type = self.symbols.type_registry_mut().create_tuple(element_types);
        Ok(tuple_type)
    }

    /// Checks a list constructor.
    pub(super) fn check_list(&mut self, elements: &[Expression], span: Span) -> Result<TypeId> {
        // Empty list defaults to List<Unit>
        if elements.is_empty() {
            let list_type = self.symbols.type_registry_mut().create_list(TypeId::UNIT);
            return Ok(list_type);
        }

        // Check all elements have compatible types
        let first_type = self.check_expression(&elements[0])?;
        for (i, elem) in elements.iter().enumerate().skip(1) {
            let elem_type = self.check_expression(elem)?;
            if !self.types_compatible(first_type, elem_type) {
                return Err(self.type_mismatch_error(
                    &format!("List element at index {i} type mismatch"),
                    first_type,
                    elem_type,
                    span,
                ));
            }
        }

        // Create list type
        let list_type = self.symbols.type_registry_mut().create_list(first_type);
        Ok(list_type)
    }

    /// Checks a dictionary literal.
    pub(super) fn check_dict(
        &mut self,
        entries: &[(String, Expression)],
        span: Span,
    ) -> Result<TypeId> {
        // Empty dict defaults to Map<Text, Unit>
        if entries.is_empty() {
            let map_type = self
                .symbols
                .type_registry_mut()
                .create_map(TypeId::TEXT, TypeId::UNIT);
            return Ok(map_type);
        }

        // Keys are always Text (string literals)
        // Check all values have compatible types
        let first_value_type = self.check_expression(&entries[0].1)?;
        for (i, (_key, value)) in entries.iter().enumerate().skip(1) {
            let value_type = self.check_expression(value)?;
            if !self.types_compatible(first_value_type, value_type) {
                return Err(self.type_mismatch_error(
                    &format!("Dictionary value at index {i} type mismatch"),
                    first_value_type,
                    value_type,
                    span,
                ));
            }
        }

        // Create map type
        let map_type = self
            .symbols
            .type_registry_mut()
            .create_map(TypeId::TEXT, first_value_type);
        Ok(map_type)
    }

    /// Checks a method call.
    pub(super) fn check_method_call(
        &mut self,
        object: &Expression,
        method: &str,
        arguments: &[Expression],
        span: Span,
    ) -> Result<TypeId> {
        // Check object type
        let object_type = self.check_expression(object)?;

        // Look up method in type registry
        let method_sig = {
            let registry = self.symbols.type_registry();
            if let Some(sig) = registry.get_method(object_type, method) {
                sig.clone()
            } else {
                // Check if it's a user-defined type
                let metadata = registry.get_type_metadata(object_type);
                let type_name = metadata.kind.name();
                if !type_name.is_empty() && object_type.as_u64() >= rive_core::type_system::TypeId::USER_DEFINED_START {
                    // For user-defined types, assume method exists and return Unit
                    // TODO: Store method signatures in type metadata
                    return Ok(TypeId::UNIT);
                }
                
                return Err(Error::SemanticWithSpan(
                    format!("Type '{type_name}' has no method '{method}'"),
                    span,
                ));
            }
        };

        // Check argument count
        if arguments.len() != method_sig.parameters.len() {
            return Err(Error::SemanticWithSpan(
                format!(
                    "Method '{method}' expects {} arguments, but {} were provided",
                    method_sig.parameters.len(),
                    arguments.len()
                ),
                span,
            ));
        }

        // Check argument types
        for (i, (expected_type, arg)) in method_sig
            .parameters
            .iter()
            .zip(arguments.iter())
            .enumerate()
        {
            let arg_type = self.check_expression(arg)?;
            if !self.types_compatible(*expected_type, arg_type) {
                return Err(self.type_mismatch_error(
                    &format!("Method argument {} type mismatch", i + 1),
                    *expected_type,
                    arg_type,
                    span,
                ));
            }
        }

        // Special handling for methods that return Optional or List types
        let return_type = match method {
            "get" => {
                // get() returns Optional<T>
                self.symbols
                    .type_registry_mut()
                    .create_optional(method_sig.return_type)
            }
            "keys" | "values" => {
                // keys()/values() return List<T>
                self.symbols
                    .type_registry_mut()
                    .create_list(method_sig.return_type)
            }
            _ => method_sig.return_type,
        };

        Ok(return_type)
    }

    /// Checks field access (for tuple indexing).
    pub(super) fn check_field_access(
        &mut self,
        object: &Expression,
        field: &str,
        span: Span,
    ) -> Result<TypeId> {
        // Check object type
        let object_type = self.check_expression(object)?;

        // Get type metadata
        let registry = self.symbols.type_registry();
        let metadata = registry.get_type_metadata(object_type);

        use rive_core::type_system::TypeKind;
        match &metadata.kind {
            TypeKind::Tuple { elements } => {
                // Parse field as integer index
                let index: usize = field.parse().map_err(|_| {
                    Error::SemanticWithSpan(format!("Invalid tuple index '{field}'"), span)
                })?;

                // Check bounds
                if index >= elements.len() {
                    return Err(Error::SemanticWithSpan(
                        format!(
                            "Tuple index {} out of bounds (tuple has {} elements)",
                            index,
                            elements.len()
                        ),
                        span,
                    ));
                }

                Ok(elements[index])
            }
            TypeKind::Struct { fields, .. } => {
                // Look up field in struct
                for (field_name, field_type) in fields {
                    if field_name == field {
                        return Ok(*field_type);
                    }
                }
                let type_name = registry.get_type_name(object_type);
                Err(Error::SemanticWithSpan(
                    format!("Type '{type_name}' has no field '{field}'"),
                    span,
                ))
            }
            _ => {
                let type_name = registry.get_type_name(object_type);
                Err(Error::SemanticWithSpan(
                    format!("Type '{type_name}' does not support field access"),
                    span,
                ))
            }
        }
    }
}
