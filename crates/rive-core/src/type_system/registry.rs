use super::{MemoryStrategy, TypeId, TypeKind, TypeMetadata};
use std::collections::HashMap;

/// Signature of a builtin method
#[derive(Debug, Clone, PartialEq)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<TypeId>,
    pub return_type: TypeId,
}

/// Central registry for all types in the Rive type system
///
/// The registry provides:
/// - Type registration and lookup by TypeId
/// - Type creation and validation
/// - Type compatibility checking
/// - Rust code generation helpers
/// - Builtin method registration and lookup
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    types: HashMap<TypeId, TypeMetadata>,
    next_id: u64,
    name_to_id: HashMap<String, TypeId>,
    /// Maps (type_id, method_name) to method signature
    methods: HashMap<(TypeId, String), MethodSignature>,
}

impl TypeRegistry {
    /// Creates a new registry with built-in types pre-registered
    pub fn new() -> Self {
        let mut registry = Self {
            types: HashMap::new(),
            next_id: TypeId::USER_DEFINED_START,
            name_to_id: HashMap::new(),
            methods: HashMap::new(),
        };

        // Register built-in primitive types
        registry.register_builtin(TypeId::INT, TypeKind::Int, "Int");
        registry.register_builtin(TypeId::FLOAT, TypeKind::Float, "Float");
        registry.register_builtin(TypeId::TEXT, TypeKind::Text, "Text");
        registry.register_builtin(TypeId::BOOL, TypeKind::Bool, "Bool");
        registry.register_builtin(TypeId::UNIT, TypeKind::Unit, "Unit");
        registry.register_builtin(TypeId::NULL, TypeKind::Null, "Null");

        // Register builtin methods
        registry.register_builtin_methods();

        registry
    }

    /// Registers a built-in type
    fn register_builtin(&mut self, id: TypeId, kind: TypeKind, name: &str) {
        let metadata = TypeMetadata::primitive(id, kind);
        self.types.insert(id, metadata);
        self.name_to_id.insert(name.to_string(), id);
    }

    /// Generates a new unique TypeId
    pub fn generate_id(&mut self) -> TypeId {
        let id = TypeId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Registers a new type and returns its TypeId
    pub fn register(&mut self, metadata: TypeMetadata) -> TypeId {
        let id = metadata.id;
        let name = metadata.kind.name();
        self.types.insert(id, metadata);
        if !name.is_empty() {
            self.name_to_id.insert(name, id);
        }
        id
    }

    /// Looks up type metadata by TypeId
    pub fn get(&self, id: TypeId) -> Option<&TypeMetadata> {
        self.types.get(&id)
    }

    /// Looks up TypeId by type name
    pub fn get_by_name(&self, name: &str) -> Option<TypeId> {
        self.name_to_id.get(name).copied()
    }

    /// Checks if two types are compatible for assignment
    pub fn are_compatible(&self, target: TypeId, source: TypeId) -> bool {
        if target == source {
            return true;
        }

        let target_meta = self.get(target);
        let source_meta = self.get(source);

        match (target_meta, source_meta) {
            (Some(t), Some(s)) => {
                // Special case 1: T → T? implicit conversion
                // Check if target is Optional<source>
                if let TypeKind::Optional { inner } = t.kind
                    && inner == source
                {
                    return true; // T can implicitly convert to T?
                }

                // Special case 2: Null → T? implicit conversion
                // Null can convert to any optional type
                if matches!(s.kind, TypeKind::Null) && matches!(t.kind, TypeKind::Optional { .. }) {
                    return true; // Null can convert to any T?
                }

                // Fall back to kind compatibility check
                self.kinds_compatible(&t.kind, &s.kind)
            }
            _ => false,
        }
    }

    /// Checks if two TypeKinds are compatible
    fn kinds_compatible(&self, target: &TypeKind, source: &TypeKind) -> bool {
        match (target, source) {
            // Same primitive types
            (TypeKind::Int, TypeKind::Int)
            | (TypeKind::Float, TypeKind::Float)
            | (TypeKind::Text, TypeKind::Text)
            | (TypeKind::Bool, TypeKind::Bool)
            | (TypeKind::Unit, TypeKind::Unit)
            | (TypeKind::Null, TypeKind::Null) => true,

            // Arrays must have same element type and size
            (
                TypeKind::Array {
                    element: e1,
                    size: s1,
                },
                TypeKind::Array {
                    element: e2,
                    size: s2,
                },
            ) => s1 == s2 && self.are_compatible(*e1, *e2),

            // Optional types
            (TypeKind::Optional { inner: i1 }, TypeKind::Optional { inner: i2 }) => {
                self.are_compatible(*i1, *i2)
            }

            // Functions must have same signature
            (
                TypeKind::Function {
                    parameters: p1,
                    return_type: r1,
                },
                TypeKind::Function {
                    parameters: p2,
                    return_type: r2,
                },
            ) => {
                p1.len() == p2.len()
                    && p1
                        .iter()
                        .zip(p2.iter())
                        .all(|(a, b)| self.are_compatible(*a, *b))
                    && self.are_compatible(*r1, *r2)
            }

            _ => false,
        }
    }

    /// Returns the Rust type string for code generation
    pub fn rust_type(&self, id: TypeId) -> String {
        self.get(id)
            .map(|meta| meta.rust_type(self))
            .unwrap_or_else(|| format!("Unknown_{}", id.as_u64()))
    }

    /// Creates an array type and returns its TypeId
    pub fn create_array(&mut self, element: TypeId, size: usize) -> TypeId {
        let id = self.generate_id();
        let kind = TypeKind::Array { element, size };
        // Arrays use CoW if they contain non-Copy types
        let elem_meta = self.get(element);
        let memory_strategy = if elem_meta.is_some_and(|m| m.is_copy()) {
            MemoryStrategy::Copy
        } else {
            MemoryStrategy::CoW
        };
        let metadata = TypeMetadata::composite(id, kind, memory_strategy);
        self.register(metadata);
        id
    }

    /// Creates an optional type and returns its TypeId
    pub fn create_optional(&mut self, inner: TypeId) -> TypeId {
        let id = self.generate_id();
        let kind = TypeKind::Optional { inner };
        // Optional is always Copy for Copy types, CoW otherwise
        let inner_meta = self.get(inner);
        let memory_strategy = if inner_meta.is_some_and(|m| m.is_copy()) {
            MemoryStrategy::Copy
        } else {
            MemoryStrategy::CoW
        };
        let metadata = TypeMetadata::composite(id, kind, memory_strategy);
        self.register(metadata);
        id
    }

    /// Creates a function type and returns its TypeId
    pub fn create_function(&mut self, parameters: Vec<TypeId>, return_type: TypeId) -> TypeId {
        let id = self.generate_id();
        let kind = TypeKind::Function {
            parameters,
            return_type,
        };
        // Functions are always Copy (function pointers)
        let metadata = TypeMetadata::composite(id, kind, MemoryStrategy::Copy);
        self.register(metadata);
        id
    }

    /// Gets the metadata for a type
    #[inline]
    pub fn get_type_metadata(&self, id: TypeId) -> &TypeMetadata {
        self.get(id).expect("TypeId should exist in registry")
    }

    /// Gets the name of a type for error messages
    ///
    /// This uses `display_name()` which properly formats nullable types as `T?`
    /// instead of `Optional`.
    pub fn get_type_name(&self, id: TypeId) -> String {
        if let Some(meta) = self.get(id) {
            meta.display_name(self)
        } else {
            format!("Unknown({})", id.as_u64())
        }
    }

    /// Registers a builtin method for a type
    pub fn register_method(
        &mut self,
        type_id: TypeId,
        name: &str,
        parameters: Vec<TypeId>,
        return_type: TypeId,
    ) {
        let signature = MethodSignature {
            name: name.to_string(),
            parameters,
            return_type,
        };
        self.methods.insert((type_id, name.to_string()), signature);
    }

    /// Looks up a method signature for a type
    pub fn get_method(&self, type_id: TypeId, name: &str) -> Option<MethodSignature> {
        // First try direct lookup for primitive types
        if let Some(sig) = self.methods.get(&(type_id, name.to_string())) {
            return Some(sig.clone());
        }

        // For composite types (List, Map, Tuple), check the type kind
        if let Some(meta) = self.get(type_id) {
            match &meta.kind {
                TypeKind::List { element } => {
                    // List methods - return synthetic signatures
                    match name {
                        "len" => {
                            return Some(MethodSignature {
                                name: "len".to_string(),
                                parameters: vec![],
                                return_type: TypeId::INT,
                            });
                        }
                        "is_empty" => {
                            return Some(MethodSignature {
                                name: "is_empty".to_string(),
                                parameters: vec![],
                                return_type: TypeId::BOOL,
                            });
                        }
                        "get" => {
                            // Return element type; caller will wrap in Optional if needed
                            return Some(MethodSignature {
                                name: "get".to_string(),
                                parameters: vec![TypeId::INT],
                                return_type: *element,
                            });
                        }
                        "append" => {
                            return Some(MethodSignature {
                                name: "append".to_string(),
                                parameters: vec![*element],
                                return_type: TypeId::UNIT,
                            });
                        }
                        "insert" => {
                            return Some(MethodSignature {
                                name: "insert".to_string(),
                                parameters: vec![TypeId::INT, *element],
                                return_type: TypeId::UNIT,
                            });
                        }
                        "remove" => {
                            return Some(MethodSignature {
                                name: "remove".to_string(),
                                parameters: vec![TypeId::INT],
                                return_type: TypeId::UNIT,
                            });
                        }
                        "clear" | "reverse" | "sort" => {
                            return Some(MethodSignature {
                                name: name.to_string(),
                                parameters: vec![],
                                return_type: TypeId::UNIT,
                            });
                        }
                        "contains" => {
                            return Some(MethodSignature {
                                name: "contains".to_string(),
                                parameters: vec![*element],
                                return_type: TypeId::BOOL,
                            });
                        }
                        _ => {}
                    }
                }
                TypeKind::Map { key, value } => {
                    // Map methods
                    match name {
                        "len" => {
                            return Some(MethodSignature {
                                name: "len".to_string(),
                                parameters: vec![],
                                return_type: TypeId::INT,
                            });
                        }
                        "is_empty" => {
                            return Some(MethodSignature {
                                name: "is_empty".to_string(),
                                parameters: vec![],
                                return_type: TypeId::BOOL,
                            });
                        }
                        "get" => {
                            // Return value type; caller will wrap in Optional if needed
                            return Some(MethodSignature {
                                name: "get".to_string(),
                                parameters: vec![*key],
                                return_type: *value,
                            });
                        }
                        "contains_key" => {
                            return Some(MethodSignature {
                                name: "contains_key".to_string(),
                                parameters: vec![*key],
                                return_type: TypeId::BOOL,
                            });
                        }
                        "insert" => {
                            return Some(MethodSignature {
                                name: "insert".to_string(),
                                parameters: vec![*key, *value],
                                return_type: TypeId::UNIT,
                            });
                        }
                        "remove" => {
                            return Some(MethodSignature {
                                name: "remove".to_string(),
                                parameters: vec![*key],
                                return_type: TypeId::UNIT,
                            });
                        }
                        "keys" | "values" => {
                            // Return key/value type; caller will wrap in List if needed
                            let elem_type = if name == "keys" { *key } else { *value };
                            return Some(MethodSignature {
                                name: name.to_string(),
                                parameters: vec![],
                                return_type: elem_type,
                            });
                        }
                        _ => {}
                    }
                }
                TypeKind::Tuple { .. } => {
                    // Tuple methods
                    if name == "len" {
                        return Some(MethodSignature {
                            name: "len".to_string(),
                            parameters: vec![],
                            return_type: TypeId::INT,
                        });
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Registers all builtin methods
    fn register_builtin_methods(&mut self) {
        // Int methods
        self.register_method(TypeId::INT, "to_float", vec![], TypeId::FLOAT);

        // Float methods
        let opt_int = self.create_optional(TypeId::INT);
        self.register_method(TypeId::FLOAT, "to_int", vec![], opt_int);
        self.register_method(TypeId::FLOAT, "is_nan", vec![], TypeId::BOOL);
        self.register_method(TypeId::FLOAT, "is_infinite", vec![], TypeId::BOOL);
        self.register_method(TypeId::FLOAT, "is_finite", vec![], TypeId::BOOL);
        self.register_method(TypeId::FLOAT, "round", vec![], TypeId::FLOAT);

        // Text methods
        self.register_method(TypeId::TEXT, "len", vec![], TypeId::INT);
        self.register_method(TypeId::TEXT, "is_empty", vec![], TypeId::BOOL);
        self.register_method(TypeId::TEXT, "contains", vec![TypeId::TEXT], TypeId::BOOL);
        self.register_method(TypeId::TEXT, "to_upper", vec![], TypeId::TEXT);
        self.register_method(TypeId::TEXT, "to_lower", vec![], TypeId::TEXT);
        self.register_method(TypeId::TEXT, "trim", vec![], TypeId::TEXT);
        self.register_method(
            TypeId::TEXT,
            "replace",
            vec![TypeId::TEXT, TypeId::TEXT],
            TypeId::TEXT,
        );
        // split returns List<Text> - we'll need to create this type dynamically
    }

    /// Creates a tuple type and returns its TypeId
    pub fn create_tuple(&mut self, elements: Vec<TypeId>) -> TypeId {
        let id = self.generate_id();
        let kind = TypeKind::Tuple {
            elements: elements.clone(),
        };
        // Tuples are Copy if all elements are Copy
        let all_copy = elements
            .iter()
            .all(|&e| self.get(e).is_some_and(|m| m.is_copy()));
        let memory_strategy = if all_copy {
            MemoryStrategy::Copy
        } else {
            MemoryStrategy::CoW
        };
        let metadata = TypeMetadata::composite(id, kind, memory_strategy);
        self.register(metadata);
        id
    }

    /// Creates a list type and returns its TypeId
    pub fn create_list(&mut self, element: TypeId) -> TypeId {
        let id = self.generate_id();
        let kind = TypeKind::List { element };
        // Lists always use CoW (Rc<RefCell<Vec<T>>>)
        let metadata = TypeMetadata::composite(id, kind, MemoryStrategy::CoW);
        self.register(metadata);
        id
    }

    /// Creates a map type and returns its TypeId
    pub fn create_map(&mut self, key: TypeId, value: TypeId) -> TypeId {
        let id = self.generate_id();
        let kind = TypeKind::Map { key, value };
        // Maps always use CoW (Rc<RefCell<HashMap<K, V>>>)
        let metadata = TypeMetadata::composite(id, kind, MemoryStrategy::CoW);
        self.register(metadata);
        id
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_types() {
        let registry = TypeRegistry::new();
        assert!(registry.get(TypeId::INT).is_some());
        assert!(registry.get(TypeId::FLOAT).is_some());
        assert_eq!(registry.get_by_name("Int"), Some(TypeId::INT));
    }

    #[test]
    fn test_type_compatibility() {
        let registry = TypeRegistry::new();
        assert!(registry.are_compatible(TypeId::INT, TypeId::INT));
        assert!(!registry.are_compatible(TypeId::INT, TypeId::FLOAT));
    }

    #[test]
    fn test_array_creation() {
        let mut registry = TypeRegistry::new();
        let array_id = registry.create_array(TypeId::INT, 5);
        let meta = registry.get(array_id).unwrap();
        assert!(matches!(meta.kind, TypeKind::Array { .. }));
    }

    #[test]
    fn test_optional_creation() {
        let mut registry = TypeRegistry::new();
        let opt_id = registry.create_optional(TypeId::INT);
        let meta = registry.get(opt_id).unwrap();
        assert!(matches!(meta.kind, TypeKind::Optional { .. }));
    }

    #[test]
    fn test_function_creation() {
        let mut registry = TypeRegistry::new();
        let func_id = registry.create_function(vec![TypeId::INT, TypeId::INT], TypeId::INT);
        let meta = registry.get(func_id).unwrap();
        assert!(matches!(meta.kind, TypeKind::Function { .. }));
    }

    #[test]
    fn test_rust_type_generation() {
        let registry = TypeRegistry::new();
        assert_eq!(registry.rust_type(TypeId::INT), "i64");
        assert_eq!(registry.rust_type(TypeId::FLOAT), "f64");
        assert_eq!(registry.rust_type(TypeId::BOOL), "bool");
    }
}
