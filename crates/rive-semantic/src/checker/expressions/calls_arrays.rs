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
}

