use super::{MemoryStrategy, TypeId, TypeKind};

/// Complete metadata for a type
///
/// This structure contains everything we need to know about a type:
/// - Its structure (TypeKind)
/// - How it's managed in memory (MemoryStrategy)
/// - Additional properties for code generation
#[derive(Debug, Clone, PartialEq)]
pub struct TypeMetadata {
    /// Unique identifier for this type
    pub id: TypeId,

    /// The kind/structure of the type
    pub kind: TypeKind,

    /// Memory management strategy
    pub memory_strategy: MemoryStrategy,

    /// Whether this type is explicitly marked as @unique
    pub explicit_unique: bool,
}

impl TypeMetadata {
    /// Creates metadata for a primitive type
    pub fn primitive(id: TypeId, kind: TypeKind) -> Self {
        let memory_strategy = MemoryStrategy::for_primitive(&kind.name());
        Self {
            id,
            kind,
            memory_strategy,
            explicit_unique: false,
        }
    }

    /// Creates metadata for a composite type (array, optional, etc.)
    pub fn composite(id: TypeId, kind: TypeKind, memory_strategy: MemoryStrategy) -> Self {
        Self {
            id,
            kind,
            memory_strategy,
            explicit_unique: false,
        }
    }

    /// Creates metadata for a user-defined type
    pub fn user_defined(
        id: TypeId,
        kind: TypeKind,
        memory_strategy: MemoryStrategy,
        explicit_unique: bool,
    ) -> Self {
        Self {
            id,
            kind,
            memory_strategy,
            explicit_unique,
        }
    }

    /// Returns true if this type can be copied implicitly
    pub fn is_copy(&self) -> bool {
        self.memory_strategy.is_copy()
    }

    /// Returns true if this type uses reference counting
    pub fn uses_rc(&self) -> bool {
        self.memory_strategy.uses_rc()
    }

    /// Returns true if this type must be moved (cannot be copied or shared)
    pub fn is_move_only(&self) -> bool {
        self.memory_strategy.is_unique() || self.explicit_unique
    }

    /// Returns the Rust type representation for code generation
    pub fn rust_type(&self, registry: &super::TypeRegistry) -> String {
        match &self.kind {
            TypeKind::Int => "i64".to_string(),
            TypeKind::Float => "f64".to_string(),
            TypeKind::Text => {
                if self.uses_rc() {
                    "std::rc::Rc<String>".to_string()
                } else {
                    "String".to_string()
                }
            }
            TypeKind::Bool => "bool".to_string(),
            TypeKind::Unit => "()".to_string(),
            TypeKind::Array { element, size } => {
                let elem_type = registry.rust_type(*element);
                format!("[{elem_type}; {size}]")
            }
            TypeKind::Optional { inner } => {
                let inner_type = registry.rust_type(*inner);
                format!("Option<{inner_type}>")
            }
            TypeKind::Function {
                parameters,
                return_type,
            } => {
                let param_types = parameters
                    .iter()
                    .map(|p| registry.rust_type(*p))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_type = registry.rust_type(*return_type);
                format!("fn({param_types}) -> {ret_type}")
            }
            TypeKind::Struct { name, .. } => {
                if self.uses_rc() {
                    format!("std::rc::Rc<std::cell::RefCell<{name}>>")
                } else {
                    name.clone()
                }
            }
            TypeKind::Enum { name, .. } => name.clone(),
            TypeKind::Generic { name } => name.clone(),
        }
    }
}

impl std::fmt::Display for TypeMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.kind, self.memory_strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_metadata() {
        let int_meta = TypeMetadata::primitive(TypeId::INT, TypeKind::Int);
        assert_eq!(int_meta.id, TypeId::INT);
        assert_eq!(int_meta.memory_strategy, MemoryStrategy::Copy);
        assert!(int_meta.is_copy());
        assert!(!int_meta.uses_rc());
    }

    #[test]
    fn test_text_metadata() {
        let text_meta = TypeMetadata::primitive(TypeId::TEXT, TypeKind::Text);
        assert_eq!(text_meta.memory_strategy, MemoryStrategy::CoW);
        assert!(!text_meta.is_copy());
        assert!(text_meta.uses_rc());
    }

    #[test]
    fn test_unique_type() {
        let unique_meta = TypeMetadata::user_defined(
            TypeId::new(1000),
            TypeKind::Struct {
                name: "FileHandle".to_string(),
                fields: vec![],
            },
            MemoryStrategy::Unique,
            true,
        );
        assert!(unique_meta.is_move_only());
    }
}
