/// Unique identifier for types in the Rive type system
///
/// TypeId is a lightweight, copyable identifier that can be used to reference types
/// without carrying around the full type information. This is crucial for performance
/// and for breaking circular dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeId(u64);

impl TypeId {
    /// Creates a new TypeId from a u64 value
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw u64 value
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Built-in type IDs (0-999 reserved for primitives)
    pub const INT: TypeId = TypeId(0);
    pub const FLOAT: TypeId = TypeId(1);
    pub const TEXT: TypeId = TypeId(2);
    pub const BOOL: TypeId = TypeId(3);
    pub const UNIT: TypeId = TypeId(4);
    pub const NULL: TypeId = TypeId(5);

    /// Starting ID for user-defined types
    pub const USER_DEFINED_START: u64 = 1000;
}

impl std::fmt::Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeId({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_id_equality() {
        assert_eq!(TypeId::INT, TypeId::new(0));
        assert_ne!(TypeId::INT, TypeId::FLOAT);
    }

    #[test]
    fn test_type_id_ordering() {
        assert!(TypeId::INT < TypeId::FLOAT);
        assert!(TypeId::FLOAT < TypeId::TEXT);
    }

    #[test]
    fn test_user_defined_range() {
        let user_type = TypeId::new(TypeId::USER_DEFINED_START);
        assert!(user_type.as_u64() >= TypeId::USER_DEFINED_START);
    }
}
