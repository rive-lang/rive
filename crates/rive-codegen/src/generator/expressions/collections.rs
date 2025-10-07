//! Collection literal code generation.
//!
//! This module handles generation of collection literals:
//! - Tuple literals: `(a, b, c)`
//! - List literals: `List(1, 2, 3)` → `Rc<RefCell<Vec<T>>>`
//! - Dict literals: `{"key": value}` → `Rc<RefCell<HashMap<String, T>>>`

use super::super::core::CodeGenerator;
use proc_macro2::TokenStream;
use quote::quote;
use rive_core::Result;
use rive_ir::RirExpression;

impl CodeGenerator {
    /// Generates code for a tuple literal.
    ///
    /// # Example
    /// `(1, 2, 3)` → `(1, 2, 3)`
    pub(crate) fn generate_tuple_literal(
        &mut self,
        elements: &[RirExpression],
    ) -> Result<TokenStream> {
        let element_exprs: Result<Vec<_>> = elements
            .iter()
            .map(|e| self.generate_expression(e))
            .collect();
        let element_exprs = element_exprs?;
        Ok(quote! { (#(#element_exprs),*) })
    }

    /// Generates code for a list literal.
    ///
    /// # Example
    /// `List(1, 2, 3)` → `Rc::new(RefCell::new(vec![1, 2, 3]))`
    ///
    /// Lists use `Rc<RefCell<Vec<T>>>` for shared ownership and interior mutability.
    pub(crate) fn generate_list_literal(
        &mut self,
        elements: &[RirExpression],
    ) -> Result<TokenStream> {
        let element_exprs: Result<Vec<_>> = elements
            .iter()
            .map(|e| self.generate_expression(e))
            .collect();
        let element_exprs = element_exprs?;

        // Generate: std::rc::Rc::new(std::cell::RefCell::new(vec![elements]))
        Ok(quote! {
            std::rc::Rc::new(std::cell::RefCell::new(vec![#(#element_exprs),*]))
        })
    }

    /// Generates code for a dictionary literal.
    ///
    /// # Example
    /// `{"name": "Alice", "age": 30}` → `Rc::new(RefCell::new(HashMap::from([...])))`
    ///
    /// Dictionaries use `Rc<RefCell<HashMap<String, T>>>` for shared ownership.
    pub(crate) fn generate_dict_literal(
        &mut self,
        entries: &[(String, RirExpression)],
    ) -> Result<TokenStream> {
        let entry_exprs: Result<Vec<_>> = entries
            .iter()
            .map(|(key, value)| {
                let value_expr = self.generate_expression(value)?;
                let key_lit = proc_macro2::Literal::string(key);
                Ok(quote! { (#key_lit.to_string(), #value_expr) })
            })
            .collect();
        let entry_exprs = entry_exprs?;

        // Generate: std::rc::Rc::new(std::cell::RefCell::new(HashMap::from([entries])))
        Ok(quote! {
            std::rc::Rc::new(std::cell::RefCell::new(
                std::collections::HashMap::from([#(#entry_exprs),*])
            ))
        })
    }
}
