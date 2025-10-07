//! Tests for type declarations, interfaces, and implementations.

use rive_lexer::tokenize;
use rive_parser::{parse, Item};

#[test]
fn test_simple_type_declaration() {
    let source = r#"
type Point(x: Int, y: Int)

fun main() {}
"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    assert_eq!(program.items.len(), 2);

    let Item::TypeDecl(type_decl) = &program.items[0] else {
        panic!("Expected type declaration");
    };
    assert_eq!(type_decl.name, "Point");
    assert_eq!(type_decl.ctor_params.len(), 2);
    assert_eq!(type_decl.ctor_params[0].name, "x");
    assert_eq!(type_decl.ctor_params[1].name, "y");
    assert_eq!(type_decl.methods.len(), 0);
    assert_eq!(type_decl.inline_impls.len(), 0);
}

#[test]
fn test_type_with_mutable_field() {
    let source = r#"
type Counter(mut count: Int)

fun main() {}
"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::TypeDecl(type_decl) = &program.items[0] else {
        panic!("Expected type declaration");
    };
    assert_eq!(type_decl.name, "Counter");
    assert_eq!(type_decl.ctor_params.len(), 1);
    assert_eq!(type_decl.ctor_params[0].name, "count");
    assert!(type_decl.ctor_params[0].mutable);
}

#[test]
fn test_type_with_instance_method() {
    let source = r#"
type Point(x: Int, y: Int) {
    fun distance(): Float {
        return 0.0
    }
}

fun main() {}
"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::TypeDecl(type_decl) = &program.items[0] else {
        panic!("Expected type declaration");
    };
    assert_eq!(type_decl.methods.len(), 1);
    assert_eq!(type_decl.methods[0].name, "distance");
    assert!(!type_decl.methods[0].is_static);
}

#[test]
fn test_type_with_static_method() {
    let source = r#"
type Point(x: Int, y: Int) {
    static fun origin(): Point {
        return Point(0, 0)
    }
}

fun main() {}
"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::TypeDecl(type_decl) = &program.items[0] else {
        panic!("Expected type declaration");
    };
    assert_eq!(type_decl.methods.len(), 1);
    assert_eq!(type_decl.methods[0].name, "origin");
    assert!(type_decl.methods[0].is_static);
}

#[test]
fn test_interface_declaration() {
    let source = r#"
interface Drawable {
    fun draw()
}

fun main() {}
"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::InterfaceDecl(interface) = &program.items[0] else {
        panic!("Expected interface declaration");
    };
    assert_eq!(interface.name, "Drawable");
    assert_eq!(interface.methods.len(), 1);
    assert_eq!(interface.methods[0].name, "draw");
}

#[test]
fn test_impl_block_for_interface() {
    let source = r#"
impl Drawable for Point {
    fun draw() {
        print("Drawing point")
    }
}

fun main() {}
"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::ImplBlock(impl_block) = &program.items[0] else {
        panic!("Expected impl block");
    };
    assert_eq!(impl_block.target_type, "Point");
    assert_eq!(impl_block.interface, Some("Drawable".to_string()));
    assert_eq!(impl_block.methods.len(), 1);
    assert_eq!(impl_block.methods[0].name, "draw");
}

#[test]
fn test_extend_block() {
    let source = r#"
extend Point {
    fun magnitude(): Float {
        return 0.0
    }
}

fun main() {}
"#;
    let tokens = tokenize(source).unwrap();
    let (program, _type_registry) = parse(&tokens).unwrap();

    let Item::ImplBlock(impl_block) = &program.items[0] else {
        panic!("Expected impl block");
    };
    assert_eq!(impl_block.target_type, "Point");
    assert_eq!(impl_block.interface, None);
    assert_eq!(impl_block.methods.len(), 1);
    assert_eq!(impl_block.methods[0].name, "magnitude");
}

