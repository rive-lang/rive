//! Integration tests for the null safety type system

use rive_core::type_system::{TypeId, TypeKind, TypeRegistry};

#[test]
fn test_null_type_registered() {
    let registry = TypeRegistry::new();

    // Null type should be registered
    let null_meta = registry.get(TypeId::NULL);
    assert!(null_meta.is_some(), "Null type should be registered");

    let null_meta = null_meta.unwrap();
    assert_eq!(null_meta.kind, TypeKind::Null);
    assert_eq!(null_meta.kind.name(), "Null");
}

#[test]
fn test_null_type_by_name() {
    let registry = TypeRegistry::new();

    // Should be able to look up Null by name
    let null_id = registry.get_by_name("Null");
    assert_eq!(null_id, Some(TypeId::NULL));
}

#[test]
fn test_implicit_t_to_optional_t_conversion() {
    let mut registry = TypeRegistry::new();

    // Create Int?
    let int_optional = registry.create_optional(TypeId::INT);

    // Int should be compatible with Int? (implicit upcast)
    assert!(
        registry.are_compatible(int_optional, TypeId::INT),
        "Int should implicitly convert to Int?"
    );

    // But Int? should NOT be compatible with Int (no implicit downcast)
    assert!(
        !registry.are_compatible(TypeId::INT, int_optional),
        "Int? should NOT implicitly convert to Int"
    );
}

#[test]
fn test_null_to_optional_t_conversion() {
    let mut registry = TypeRegistry::new();

    // Create Int?
    let int_optional = registry.create_optional(TypeId::INT);

    // Null should be compatible with Int?
    assert!(
        registry.are_compatible(int_optional, TypeId::NULL),
        "Null should implicitly convert to Int?"
    );

    // Create Text?
    let text_optional = registry.create_optional(TypeId::TEXT);

    // Null should be compatible with Text? (any optional)
    assert!(
        registry.are_compatible(text_optional, TypeId::NULL),
        "Null should implicitly convert to Text?"
    );
}

#[test]
fn test_null_not_compatible_with_non_nullable() {
    let registry = TypeRegistry::new();

    // Null should NOT be compatible with Int
    assert!(
        !registry.are_compatible(TypeId::INT, TypeId::NULL),
        "Null should NOT be compatible with non-nullable Int"
    );

    // Null should NOT be compatible with Text
    assert!(
        !registry.are_compatible(TypeId::TEXT, TypeId::NULL),
        "Null should NOT be compatible with non-nullable Text"
    );
}

#[test]
fn test_optional_display_name() {
    let mut registry = TypeRegistry::new();

    // Create Int?
    let int_optional = registry.create_optional(TypeId::INT);
    let int_opt_meta = registry.get(int_optional).unwrap();

    // Display name should be "Int?" not "Optional"
    let display_name = int_opt_meta.display_name(&registry);
    assert_eq!(
        display_name, "Int?",
        "Optional type should display as 'Int?'"
    );
}

#[test]
fn test_nested_optional_display_name() {
    let mut registry = TypeRegistry::new();

    // Create Int?
    let int_optional = registry.create_optional(TypeId::INT);

    // Create Int??
    let int_double_optional = registry.create_optional(int_optional);
    let double_opt_meta = registry.get(int_double_optional).unwrap();

    // Display name should be "Int??"
    let display_name = double_opt_meta.display_name(&registry);
    assert_eq!(
        display_name, "Int??",
        "Nested optional should display as 'Int??'"
    );
}

#[test]
fn test_optional_rust_type_generation() {
    let mut registry = TypeRegistry::new();

    // Create Int?
    let int_optional = registry.create_optional(TypeId::INT);

    // Rust type should be Option<i64>
    let rust_type = registry.rust_type(int_optional);
    assert_eq!(rust_type, "Option<i64>", "Int? should generate Option<i64>");
}

#[test]
fn test_null_rust_type_generation() {
    let registry = TypeRegistry::new();

    // Null type should generate Option<()> (fallback, shouldn't normally be used)
    let rust_type = registry.rust_type(TypeId::NULL);
    assert_eq!(
        rust_type, "Option<()>",
        "Null should generate Option<()> as fallback"
    );
}

#[test]
fn test_same_optional_compatibility() {
    let mut registry = TypeRegistry::new();

    let int_opt1 = registry.create_optional(TypeId::INT);
    let int_opt2 = registry.create_optional(TypeId::INT);

    // Two Int? types should be compatible
    assert!(
        registry.are_compatible(int_opt1, int_opt2),
        "Int? should be compatible with Int?"
    );
}

#[test]
fn test_different_optional_incompatibility() {
    let mut registry = TypeRegistry::new();

    let int_optional = registry.create_optional(TypeId::INT);
    let text_optional = registry.create_optional(TypeId::TEXT);

    // Int? and Text? should NOT be compatible
    assert!(
        !registry.are_compatible(int_optional, text_optional),
        "Int? should NOT be compatible with Text?"
    );
}

#[test]
fn test_null_is_primitive() {
    let registry = TypeRegistry::new();
    let null_meta = registry.get(TypeId::NULL).unwrap();

    // Null should be considered a primitive type
    assert!(
        null_meta.kind.is_primitive(),
        "Null should be a primitive type"
    );
}

#[test]
fn test_optional_is_composite() {
    let mut registry = TypeRegistry::new();
    let int_optional = registry.create_optional(TypeId::INT);
    let opt_meta = registry.get(int_optional).unwrap();

    // Optional should be considered a composite type
    assert!(
        opt_meta.kind.is_composite(),
        "Optional should be a composite type"
    );
}

#[test]
fn test_chained_optional_conversions() {
    let mut registry = TypeRegistry::new();

    // Create Int and Int?
    let int_optional = registry.create_optional(TypeId::INT);

    // Int → Int? works
    assert!(registry.are_compatible(int_optional, TypeId::INT));

    // Create Int??
    let int_double_optional = registry.create_optional(int_optional);

    // Int? → Int?? should work (Int? is T, Int?? is T?)
    assert!(
        registry.are_compatible(int_double_optional, int_optional),
        "Int? should convert to Int??"
    );

    // But Int → Int?? should NOT work directly
    // (Int converts to Int?, but not to Int??)
    assert!(
        !registry.are_compatible(int_double_optional, TypeId::INT),
        "Int should NOT directly convert to Int?? (must go through Int? first)"
    );
}
