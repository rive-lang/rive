use rive_lexer::tokenize;
use rive_parser::parse;
use rive_semantic::analyze_with_registry;

#[test]
fn test_tuple_type_checking() {
    let source = r#"
fun main() {
    let t = (1, "hello", true)
    let first: Int = t.0
    let second: Text = t.1
    let third: Bool = t.2
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let (program, type_registry) = parse(&tokens).expect("Failed to parse");
    let result = analyze_with_registry(&program, type_registry);
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_list_type_checking() {
    let source = r#"
fun main() {
    let list = List(1, 2, 3)
    let len: Int = list.len()
    let empty: Bool = list.is_empty()
    list.append(4)
    let item = list.get(0)
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let (program, type_registry) = parse(&tokens).expect("Failed to parse");
    let result = analyze_with_registry(&program, type_registry);
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_dict_type_checking() {
    let source = r#"
fun main() {
    let dict = {"name": "Alice", "age": "30"}
    let len: Int = dict.len()
    let name = dict.get("name")
    let has_key: Bool = dict.contains_key("age")
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let (program, type_registry) = parse(&tokens).expect("Failed to parse");
    let result = analyze_with_registry(&program, type_registry);
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_primitive_methods() {
    let source = r#"
fun main() {
    let num = 42
    let float_val: Float = num.to_float()
    
    let text = "hello"
    let len: Int = text.len()
    let upper: Text = text.to_upper()
    let trimmed: Text = text.trim()
    let contains: Bool = text.contains("ll")
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let (program, type_registry) = parse(&tokens).expect("Failed to parse");
    let result = analyze_with_registry(&program, type_registry);
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_type_mismatch_errors() {
    let source = r#"
fun main() {
    let list = List(1, 2, 3)
    let wrong_type: Text = list.len()  // Should fail: Int != Text
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let (program, type_registry) = parse(&tokens).expect("Failed to parse");
    let result = analyze_with_registry(&program, type_registry);
    assert!(result.is_err(), "Should have failed type checking");
}

#[test]
fn test_method_not_found() {
    let source = r#"
fun main() {
    let num = 42
    let result = num.nonexistent_method()  // Should fail: method doesn't exist
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let (program, type_registry) = parse(&tokens).expect("Failed to parse");
    let result = analyze_with_registry(&program, type_registry);
    assert!(result.is_err(), "Should have failed: method not found");
}

#[test]
fn test_nested_collection_types() {
    let source = r#"
fun main() {
    let simple_list = List(1, 2, 3)
    let simple_dict = {"x": "1", "y": "2"}
    let simple_tuple = (1, "hello")
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let (program, type_registry) = parse(&tokens).expect("Failed to parse");
    let result = analyze_with_registry(&program, type_registry);
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}
