//! Display implementations for RIR structures for debugging.

use std::fmt;

use crate::{BinaryOp, RirBlock, RirExpression, RirFunction, RirModule, RirStatement, UnaryOp};

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

impl fmt::Display for RirBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            writeln!(f, "  {stmt}")?;
        }
        if let Some(expr) = &self.final_expr {
            writeln!(f, "  {expr}")?;
        }
        Ok(())
    }
}

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

            // TODO: Phase 6 - Implement display for new control flow statements
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

impl fmt::Display for RirExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntLiteral { value, .. } => write!(f, "{value}"),
            Self::FloatLiteral { value, .. } => write!(f, "{value}"),
            Self::StringLiteral { value, .. } => write!(f, "\"{value}\""),
            Self::BoolLiteral { value, .. } => write!(f, "{value}"),
            Self::Unit { .. } => write!(f, "()"),
            Self::Variable { name, .. } => write!(f, "{name}"),
            Self::Binary {
                op, left, right, ..
            } => {
                write!(f, "({left} {op} {right})")
            }
            Self::Unary { op, operand, .. } => {
                write!(f, "({op}{operand})")
            }
            Self::Call {
                function,
                arguments,
                ..
            } => {
                write!(f, "{function}(")?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Self::ArrayLiteral { elements, .. } => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{elem}")?;
                }
                write!(f, "]")
            }
            Self::Index { array, index, .. } => {
                write!(f, "{array}[{index}]")
            }

            // TODO: Phase 6 - Implement display for control flow expressions
            Self::If { .. } => write!(f, "if {{ ... }} else {{ ... }}"),
            Self::Match { .. } => write!(f, "match {{ ... }}"),
            Self::Block { .. } => write!(f, "{{ ... }}"),
            Self::While { .. } => write!(f, "while {{ ... }}"),
            Self::For { .. } => write!(f, "for {{ ... }}"),
            Self::Loop { .. } => write!(f, "loop {{ ... }}"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterEqual => ">=",
            Self::And => "&&",
            Self::Or => "||",
        };
        write!(f, "{symbol}")
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Self::Negate => "-",
            Self::Not => "!",
        };
        write!(f, "{symbol}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExprBuilder;
    use rive_core::{
        span::{Location, Span},
        type_system::{MemoryStrategy, TypeId, TypeRegistry},
    };

    fn dummy_span() -> Span {
        Span::new(Location::new(1, 1), Location::new(1, 10))
    }

    #[test]
    fn test_expression_display() {
        let span = dummy_span();

        let int_expr = ExprBuilder::int(42, span);
        assert_eq!(format!("{int_expr}"), "42");

        let str_expr = ExprBuilder::string("hello".to_string(), span);
        assert_eq!(format!("{str_expr}"), "\"hello\"");

        let var_expr = ExprBuilder::var("x".to_string(), TypeId::INT, span);
        assert_eq!(format!("{var_expr}"), "x");
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(format!("{}", BinaryOp::Add), "+");
        assert_eq!(format!("{}", BinaryOp::Equal), "==");
        assert_eq!(format!("{}", BinaryOp::And), "&&");
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
