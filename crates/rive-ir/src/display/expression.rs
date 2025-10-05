//! Display implementation for RIR expressions.

use std::fmt;

use crate::RirExpression;

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
            Self::If { .. } => write!(f, "if {{ ... }} else {{ ... }}"),
            Self::Match { .. } => write!(f, "match {{ ... }}"),
            Self::Block { .. } => write!(f, "{{ ... }}"),
            Self::While { .. } => write!(f, "while {{ ... }}"),
            Self::For { .. } => write!(f, "for {{ ... }}"),
            Self::Loop { .. } => write!(f, "loop {{ ... }}"),
            Self::NullLiteral { .. } => write!(f, "null"),
            Self::Elvis {
                value, fallback, ..
            } => write!(f, "({value} ?: {fallback})"),
            Self::SafeCall { object, call, .. } => write!(f, "({object}?.{call})"),
            Self::WrapOptional { value, .. } => write!(f, "Some({value})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ExprBuilder;
    use rive_core::{span::Location, span::Span, type_system::TypeId};

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
}
