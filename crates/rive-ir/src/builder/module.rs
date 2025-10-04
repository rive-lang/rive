//! Builder for constructing RIR modules.

use crate::{RirFunction, RirModule};

/// Builder for constructing RIR modules
pub struct RirBuilder {
    module: RirModule,
}

impl RirBuilder {
    /// Creates a new RIR builder
    #[must_use]
    pub fn new(module: RirModule) -> Self {
        Self { module }
    }

    /// Adds a function to the module
    pub fn add_function(&mut self, function: RirFunction) -> &mut Self {
        self.module.add_function(function);
        self
    }

    /// Builds and returns the module
    #[must_use]
    pub fn build(self) -> RirModule {
        self.module
    }

    /// Returns a mutable reference to the module
    pub fn module_mut(&mut self) -> &mut RirModule {
        &mut self.module
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RirBlock;
    use rive_core::{
        span::Location,
        span::Span,
        type_system::{TypeId, TypeRegistry},
    };

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_module_builder() {
        let span = dummy_span();
        let registry = TypeRegistry::new();
        let module = RirModule::new(registry);
        let mut builder = RirBuilder::new(module);

        let func = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock::new(span),
            span,
        );

        builder.add_function(func);
        let module = builder.build();

        assert_eq!(module.functions.len(), 1);
        assert!(module.get_function("test").is_some());
    }
}
