/// Extensible type system for Rive language
///
/// This module provides a flexible type system that supports:
/// - Built-in primitive types
/// - User-defined types (struct, enum)
/// - Memory management strategies (Copy, CoW, Unique)
/// - Type registration and lookup
mod memory_strategy;
mod registry;
mod type_id;
mod type_kind;
mod type_metadata;

pub use memory_strategy::MemoryStrategy;
pub use registry::TypeRegistry;
pub use type_id::TypeId;
pub use type_kind::TypeKind;
pub use type_metadata::TypeMetadata;
