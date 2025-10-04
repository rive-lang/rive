//! Display implementations for RIR module and function.

use std::fmt;

use crate::{RirFunction, RirModule};

impl fmt::Display for RirModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "RIR Module")?;
        writeln!(f, "==========")?;
        for func in &self.functions {
            write!(f, "{func}")?;
        }
        Ok(())
    }
}

impl fmt::Display for RirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\nfn {}(", self.name)?;
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(
                f,
                "{}{}: {:?}",
                if param.is_mutable { "mut " } else { "" },
                param.name,
                param.type_id
            )?;
        }
        writeln!(f, ") -> {:?} {{", self.return_type)?;
        write!(f, "{}", self.body)?;
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RirBlock;
    use rive_core::{
        span::{Location, Span},
        type_system::{TypeId, TypeRegistry},
    };

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_function_display() {
        let span = dummy_span();
        let func = RirFunction::new(
            "test".to_string(),
            vec![],
            TypeId::UNIT,
            RirBlock::new(span),
            span,
        );

        let display = format!("{func}");
        assert!(display.contains("fn test"));
    }

    #[test]
    fn test_module_display() {
        let registry = TypeRegistry::new();
        let module = RirModule::new(registry);

        let display = format!("{module}");
        assert!(display.contains("RIR Module"));
    }
}
