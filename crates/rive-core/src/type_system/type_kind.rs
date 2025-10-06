use super::TypeId;

/// The kind/variant of a type
///
/// This enum represents the structure of a type without memory management details.
/// It separates "what a type is" from "how it's managed in memory".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    /// Primitive integer type (i64)
    Int,

    /// Primitive floating-point type (f64)
    Float,

    /// Primitive string type (UTF-8)
    Text,

    /// Primitive boolean type
    Bool,

    /// Unit type (void/no return value)
    Unit,

    /// Null type (bottom type for nullable values)
    Null,

    /// Array type with element type and size
    Array { element: TypeId, size: usize },

    /// Tuple type with element types
    Tuple { elements: Vec<TypeId> },

    /// List type (dynamic array)
    List { element: TypeId },

    /// Map/dictionary type
    Map { key: TypeId, value: TypeId },

    /// Optional/nullable type
    Optional { inner: TypeId },

    /// Function type
    Function {
        parameters: Vec<TypeId>,
        return_type: TypeId,
    },

    /// User-defined struct type
    Struct {
        name: String,
        fields: Vec<(String, TypeId)>,
    },

    /// User-defined enum type
    Enum {
        name: String,
        variants: Vec<(String, Option<TypeId>)>,
    },

    /// Generic type parameter (for future use)
    Generic { name: String },
}

impl TypeKind {
    /// Returns true if this is a primitive type
    pub const fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::Int | Self::Float | Self::Text | Self::Bool | Self::Unit | Self::Null
        )
    }

    /// Returns true if this is a composite type (contains other types)
    pub const fn is_composite(&self) -> bool {
        matches!(
            self,
            Self::Array { .. }
                | Self::Tuple { .. }
                | Self::List { .. }
                | Self::Map { .. }
                | Self::Optional { .. }
                | Self::Function { .. }
        )
    }

    /// Returns true if this is a user-defined type
    pub const fn is_user_defined(&self) -> bool {
        matches!(self, Self::Struct { .. } | Self::Enum { .. })
    }

    /// Returns the name of the type for display purposes
    pub fn name(&self) -> String {
        match self {
            Self::Int => "Int".to_string(),
            Self::Float => "Float".to_string(),
            Self::Text => "Text".to_string(),
            Self::Bool => "Bool".to_string(),
            Self::Unit => "Unit".to_string(),
            Self::Null => "Null".to_string(),
            Self::Array { .. } => "Array".to_string(),
            Self::Tuple { .. } => "Tuple".to_string(),
            Self::List { .. } => "List".to_string(),
            Self::Map { .. } => "Map".to_string(),
            Self::Optional { .. } => "Optional".to_string(),
            Self::Function { .. } => "Function".to_string(),
            Self::Struct { name, .. } => name.clone(),
            Self::Enum { name, .. } => name.clone(),
            Self::Generic { name } => name.clone(),
        }
    }
}

impl std::fmt::Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        assert!(TypeKind::Int.is_primitive());
        assert!(TypeKind::Float.is_primitive());
        assert!(TypeKind::Bool.is_primitive());
        assert!(
            !TypeKind::Array {
                element: TypeId::INT,
                size: 5
            }
            .is_primitive()
        );
    }

    #[test]
    fn test_composite_types() {
        assert!(
            TypeKind::Array {
                element: TypeId::INT,
                size: 5
            }
            .is_composite()
        );
        assert!(TypeKind::Optional { inner: TypeId::INT }.is_composite());
        assert!(!TypeKind::Int.is_composite());
    }

    #[test]
    fn test_user_defined_types() {
        let struct_type = TypeKind::Struct {
            name: "Point".to_string(),
            fields: vec![],
        };
        assert!(struct_type.is_user_defined());
        assert!(!TypeKind::Int.is_user_defined());
    }
}
