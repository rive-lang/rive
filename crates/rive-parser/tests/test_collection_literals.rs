use rive_lexer::tokenize;
use rive_parser::parse;

#[test]
fn test_tuple_literal() {
    let source = r#"
fun main() {
    let t = (1, "hello", true)
    print(t)
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let result = parse(&tokens);
    assert!(
        result.is_ok(),
        "Failed to parse tuple literal: {:?}",
        result.err()
    );
}

#[test]
fn test_list_literal() {
    let source = r#"
fun main() {
    let list = [1, 2, 3, 4, 5]
    print(list.len())
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let result = parse(&tokens);
    assert!(
        result.is_ok(),
        "Failed to parse list literal: {:?}",
        result.err()
    );
}

#[test]
fn test_dict_literal() {
    let source = r#"
fun main() {
    let dict = {"name": "Alice", "age": 30}
    print(dict.len())
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let result = parse(&tokens);
    assert!(
        result.is_ok(),
        "Failed to parse dict literal: {:?}",
        result.err()
    );
}

#[test]
fn test_method_calls() {
    let source = r#"
fun main() {
    let list = [1, 2, 3]
    let len = list.len()
    let empty = list.is_empty()
    list.append(4)
    let item = list.get(0)
    
    let text = "hello"
    let upper = text.to_upper()
    let trimmed = text.trim()
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let result = parse(&tokens);
    assert!(
        result.is_ok(),
        "Failed to parse method calls: {:?}",
        result.err()
    );
}

#[test]
fn test_field_access() {
    let source = r#"
fun main() {
    let t = (1, "hello", true)
    let first = t.0
    let second = t.1
    let third = t.2
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let result = parse(&tokens);
    assert!(
        result.is_ok(),
        "Failed to parse field access: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_collections() {
    let source = r#"
fun main() {
    let nested = [["a", "b"], ["c", "d"]]
    let dict_list = [{"x": 1}, {"y": 2}]
    let tuple_list = [(1, 2), (3, 4)]
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let result = parse(&tokens);
    assert!(
        result.is_ok(),
        "Failed to parse nested collections: {:?}",
        result.err()
    );
}

#[test]
fn test_empty_collections() {
    let source = r#"
fun main() {
    let empty_list = []
    let empty_dict = {}
    let empty_tuple = ()
}
"#;
    let tokens = tokenize(source).expect("Failed to tokenize");
    let result = parse(&tokens);
    assert!(
        result.is_ok(),
        "Failed to parse empty collections: {:?}",
        result.err()
    );
}
