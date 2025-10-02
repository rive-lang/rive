/// Legacy type representation (deprecated - use type_system module instead)
///
/// This module is kept for backward compatibility during migration.
/// New code should use the `type_system` module with TypeRegistry.
use std::fmt;

use crate::type_system::{MemoryStrategy, TypeId, TypeKind, TypeRegistry};

/// Type representation for expressions and variables
///
/// **DEPRECATED**: This enum will be phased out in favor of TypeId + TypeRegistry.
/// It's currently kept to maintain compatibility with existing parser and semantic analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Integer type (i64)
    Int,
    /// Floating-point type (f64)
    Float,
    /// String type
    Text,
    /// Boolean type
    Bool,
    /// Unit/void type
    Unit,
    /// Array type with element type and size
    Array(Box<Type>, usize),
    /// Optional/nullable type
    Optional(Box<Type>),
    /// Function type with parameter types and return type
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
}

impl Type {
    /// Converts the legacy Type to TypeId using the type registry
    ///
    /// This is the bridge function between old and new type systems.
    /// It recursively converts nested types and registers them in the registry.
    pub fn to_type_id(&self, registry: &mut TypeRegistry) -> TypeId {
        match self {
            Type::Int => TypeId::INT,
            Type::Float => TypeId::FLOAT,
            Type::Text => TypeId::TEXT,
            Type::Bool => TypeId::BOOL,
            Type::Unit => TypeId::UNIT,
            Type::Array(element, size) => {
                let element_id = element.to_type_id(registry);
                registry.create_array(element_id, *size)
            }
            Type::Optional(inner) => {
                let inner_id = inner.to_type_id(registry);
                registry.create_optional(inner_id)
            }
            Type::Function {
                parameters,
                return_type,
            } => {
                let param_ids = parameters.iter().map(|p| p.to_type_id(registry)).collect();
                let return_id = return_type.to_type_id(registry);
                registry.create_function(param_ids, return_id)
            }
        }
    }

    /// Converts a TypeId back to legacy Type for compatibility
    ///
    /// This allows gradual migration - new code uses TypeId internally,
    /// but can still communicate with legacy code.
    pub fn from_type_id(id: TypeId, registry: &TypeRegistry) -> Option<Self> {
        let meta = registry.get(id)?;
        match &meta.kind {
            TypeKind::Int => Some(Type::Int),
            TypeKind::Float => Some(Type::Float),
            TypeKind::Text => Some(Type::Text),
            TypeKind::Bool => Some(Type::Bool),
            TypeKind::Unit => Some(Type::Unit),
            TypeKind::Array { element, size } => {
                let element_type = Self::from_type_id(*element, registry)?;
                Some(Type::Array(Box::new(element_type), *size))
            }
            TypeKind::Optional { inner } => {
                let inner_type = Self::from_type_id(*inner, registry)?;
                Some(Type::Optional(Box::new(inner_type)))
            }
            TypeKind::Function {
                parameters,
                return_type,
            } => {
                let param_types = parameters
                    .iter()
                    .map(|p| Self::from_type_id(*p, registry))
                    .collect::<Option<Vec<_>>>()?;
                let return_ty = Self::from_type_id(*return_type, registry)?;
                Some(Type::Function {
                    parameters: param_types,
                    return_type: Box::new(return_ty),
                })
            }
            TypeKind::Struct { name, .. } => {
                // For now, represent user types as Text (will be properly handled in future)
                eprintln!("Warning: Converting struct '{name}' to Text type");
                Some(Type::Text)
            }
            TypeKind::Enum { name, .. } => {
                // For now, represent user types as Text (will be properly handled in future)
                eprintln!("Warning: Converting enum '{name}' to Text type");
                Some(Type::Text)
            }
            TypeKind::Generic { .. } => None,
        }
    }

    /// Gets the memory strategy for this type
    pub fn memory_strategy(&self) -> MemoryStrategy {
        match self {
            Type::Int | Type::Float | Type::Bool | Type::Unit => MemoryStrategy::Copy,
            Type::Text => MemoryStrategy::CoW,
            Type::Array(elem, _) => {
                // Arrays inherit strategy from elements (simplified)
                if matches!(**elem, Type::Int | Type::Float | Type::Bool | Type::Unit) {
                    MemoryStrategy::Copy
                } else {
                    MemoryStrategy::CoW
                }
            }
            Type::Optional(inner) => inner.memory_strategy(),
            Type::Function { .. } => MemoryStrategy::Copy,
        }
    }

    /// Returns true if this is the Unit type
    pub const fn is_unit(&self) -> bool {
        matches!(self, Type::Unit)
    }

    /// Returns true if this is an Optional type
    pub const fn is_optional(&self) -> bool {
        matches!(self, Type::Optional(_))
    }

    /// Returns true if this is an Array type
    pub const fn is_array(&self) -> bool {
        matches!(self, Type::Array(_, _))
    }

    /// Returns true if this is a Function type
    pub const fn is_function(&self) -> bool {
        matches!(self, Type::Function { .. })
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Text => write!(f, "Text"),
            Type::Bool => write!(f, "Bool"),
            Type::Unit => write!(f, "Unit"),
            Type::Array(element, size) => write!(f, "[{element}; {size}]"),
            Type::Optional(inner) => write!(f, "Optional<{inner}>"),
            Type::Function {
                parameters,
                return_type,
            } => {
                let params = parameters
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "fn({params}) -> {return_type}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        assert_eq!(Type::Int.to_string(), "Int");
        assert_eq!(Type::Float.to_string(), "Float");
        assert_eq!(Type::Text.to_string(), "Text");
    }

    #[test]
    fn test_complex_types() {
        let array = Type::Array(Box::new(Type::Int), 5);
        assert_eq!(array.to_string(), "[Int; 5]");

        let opt = Type::Optional(Box::new(Type::Int));
        assert_eq!(opt.to_string(), "Optional<Int>");
    }

    #[test]
    fn test_type_id_conversion() {
        let mut registry = TypeRegistry::new();

        // Test primitive conversion
        assert_eq!(Type::Int.to_type_id(&mut registry), TypeId::INT);
        assert_eq!(Type::Float.to_type_id(&mut registry), TypeId::FLOAT);

        // Test round-trip conversion
        let original = Type::Array(Box::new(Type::Int), 3);
        let type_id = original.to_type_id(&mut registry);
        let converted = Type::from_type_id(type_id, &registry).unwrap();
        assert_eq!(original, converted);
    }

    #[test]
    fn test_memory_strategy() {
        assert_eq!(Type::Int.memory_strategy(), MemoryStrategy::Copy);
        assert_eq!(Type::Text.memory_strategy(), MemoryStrategy::CoW);
        assert_eq!(
            Type::Array(Box::new(Type::Int), 5).memory_strategy(),
            MemoryStrategy::Copy
        );
        assert_eq!(
            Type::Array(Box::new(Type::Text), 5).memory_strategy(),
            MemoryStrategy::CoW
        );
    }

    #[test]
    fn test_optional_types() {
        let opt = Type::Optional(Box::new(Type::Int));
        assert_eq!(opt.to_string(), "Optional<Int>");
    }

    #[test]
    fn test_array_types() {
        let array = Type::Array(Box::new(Type::Int), 10);
        assert_eq!(array.to_string(), "[Int; 10]");
    }

    #[test]
    fn test_type_display() {
        let func = Type::Function {
            parameters: vec![Type::Int, Type::Float],
            return_type: Box::new(Type::Bool),
        };
        assert_eq!(func.to_string(), "fn(Int, Float) -> Bool");
    }
}
