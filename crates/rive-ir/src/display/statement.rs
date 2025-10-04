//! Display implementation for RIR statements.

use std::fmt;

use crate::RirStatement;

impl fmt::Display for RirStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Let {
                name,
                type_id,
                is_mutable,
                value,
                memory_strategy,
                ..
            } => {
                write!(
                    f,
                    "let {}{}: {:?} = {} // {:?}",
                    if *is_mutable { "mut " } else { "" },
                    name,
                    type_id,
                    value,
                    memory_strategy
                )
            }
            Self::Assign { name, value, .. } => {
                write!(f, "{name} = {value}")
            }
            Self::AssignIndex {
                array,
                index,
                value,
                ..
            } => {
                write!(f, "{array}[{index}] = {value}")
            }
            Self::Return { value, .. } => {
                if let Some(v) = value {
                    write!(f, "return {v}")
                } else {
                    write!(f, "return")
                }
            }
            Self::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                write!(f, "if {condition} {{\n{then_block}")?;
                if let Some(else_b) = else_block {
                    write!(f, "}} else {{\n{else_b}")?;
                }
                write!(f, "}}")
            }
            Self::While {
                condition, body, ..
            } => {
                write!(f, "while {condition} {{\n{body}}}")
            }
            Self::Expression { expr, .. } => {
                write!(f, "{expr}")
            }
            Self::Block { block, .. } => {
                write!(f, "{{\n{block}}}")
            }
            Self::For {
                variable, label, ..
            } => {
                if let Some(lbl) = label {
                    write!(f, "{lbl}: for {variable} in <range> {{ ... }}")
                } else {
                    write!(f, "for {variable} in <range> {{ ... }}")
                }
            }
            Self::Loop { label, .. } => {
                if let Some(lbl) = label {
                    write!(f, "{lbl}: loop {{ ... }}")
                } else {
                    write!(f, "loop {{ ... }}")
                }
            }
            Self::Break { label, value, .. } => {
                if let Some(lbl) = label {
                    if let Some(val) = value {
                        write!(f, "break {lbl} with {val}")
                    } else {
                        write!(f, "break {lbl}")
                    }
                } else if let Some(val) = value {
                    write!(f, "break with {val}")
                } else {
                    write!(f, "break")
                }
            }
            Self::Continue { label, .. } => {
                if let Some(lbl) = label {
                    write!(f, "continue {lbl}")
                } else {
                    write!(f, "continue")
                }
            }
            Self::Match { .. } => {
                write!(f, "match {{ ... }}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExprBuilder;
    use rive_core::{
        span::{Location, Span},
        type_system::{MemoryStrategy, TypeId},
    };

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_statement_display() {
        let span = dummy_span();

        let stmt = RirStatement::Let {
            name: "x".to_string(),
            type_id: TypeId::INT,
            is_mutable: false,
            value: Box::new(ExprBuilder::int(42, span)),
            memory_strategy: MemoryStrategy::Copy,
            span,
        };

        let display = format!("{stmt}");
        assert!(display.contains("let x"));
        assert!(display.contains("42"));
    }
}
