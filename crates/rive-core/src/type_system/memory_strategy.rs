/// Memory management strategy for types (AVS model)
///
/// Rive uses Automatic Value Semantics (AVS) to provide intuitive memory management
/// without garbage collection. This enum represents the three fundamental strategies:
///
/// - Copy: Simple bitwise copy (i32, bool, small tuples)
/// - CoW: Reference-counted with copy-on-write (List, Text, Map)
/// - Unique: No sharing allowed, move-only (File handles, unique resources)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryStrategy {
    /// Stack-allocated, bitwise copyable types (i32, f64, bool)
    ///
    /// Assignment creates a new independent copy with no heap allocation.
    /// Example: `let b = a` copies the value
    Copy,

    /// Reference-counted with copy-on-write optimization
    ///
    /// Assignment creates a logical copy using Rc<T> and RefCell<T>.
    /// Mutations trigger automatic cloning when reference count > 1.
    /// Example: `let b = a` shares the data; `b.append(x)` clones if shared
    CoW,

    /// Move-only semantics, no sharing allowed
    ///
    /// Marked with @unique annotation in source code.
    /// Compiler enforces single ownership at compile time.
    /// Example: File handles, network connections
    Unique,
}

impl MemoryStrategy {
    /// Determines if this strategy allows implicit copies
    pub const fn is_copy(&self) -> bool {
        matches!(self, Self::Copy)
    }

    /// Determines if this strategy uses reference counting
    pub const fn uses_rc(&self) -> bool {
        matches!(self, Self::CoW)
    }

    /// Determines if this strategy enforces unique ownership
    pub const fn is_unique(&self) -> bool {
        matches!(self, Self::Unique)
    }

    /// Returns the default strategy for a primitive type name
    pub fn for_primitive(name: &str) -> Self {
        match name {
            "Int" | "Float" | "Bool" | "Unit" => Self::Copy,
            "Text" => Self::CoW,
            _ => Self::CoW, // Default for complex types
        }
    }
}

impl std::fmt::Display for MemoryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Copy => write!(f, "Copy"),
            Self::CoW => write!(f, "CoW"),
            Self::Unique => write!(f, "Unique"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_strategy_properties() {
        assert!(MemoryStrategy::Copy.is_copy());
        assert!(!MemoryStrategy::Copy.uses_rc());
        assert!(!MemoryStrategy::Copy.is_unique());

        assert!(!MemoryStrategy::CoW.is_copy());
        assert!(MemoryStrategy::CoW.uses_rc());
        assert!(!MemoryStrategy::CoW.is_unique());

        assert!(!MemoryStrategy::Unique.is_copy());
        assert!(!MemoryStrategy::Unique.uses_rc());
        assert!(MemoryStrategy::Unique.is_unique());
    }

    #[test]
    fn test_primitive_strategies() {
        assert_eq!(MemoryStrategy::for_primitive("Int"), MemoryStrategy::Copy);
        assert_eq!(MemoryStrategy::for_primitive("Float"), MemoryStrategy::Copy);
        assert_eq!(MemoryStrategy::for_primitive("Bool"), MemoryStrategy::Copy);
        assert_eq!(MemoryStrategy::for_primitive("Text"), MemoryStrategy::CoW);
    }
}
