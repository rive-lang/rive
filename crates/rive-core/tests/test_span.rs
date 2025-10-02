//! Tests for source code span tracking.

use rive_core::span::{Location, Span};

#[test]
fn test_location_creation() {
    let loc = Location::new(10, 5);
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
}

#[test]
fn test_span_creation() {
    let start = Location::new(1, 0);
    let end = Location::new(1, 10);
    let span = Span::new(start, end);

    assert_eq!(span.start, start);
    assert_eq!(span.end, end);
}

#[test]
fn test_span_from_range() {
    let span = Span::from_range(0, 10);
    // Note: This creates a temporary span with byte offsets
    // In real usage, these would be converted to line/column
    assert_eq!(span.start.offset, 0);
    assert_eq!(span.end.offset, 10);
}

#[test]
fn test_span_contains() {
    let span = Span::new(Location::new(5, 10), Location::new(5, 20));

    let inside = Location::new(5, 15);
    assert!(span.contains(inside));

    let before = Location::new(5, 5);
    assert!(!span.contains(before));

    let after = Location::new(5, 25);
    assert!(!span.contains(after));
}

#[test]
fn test_span_merge() {
    let span1 = Span::new(Location::new(1, 0), Location::new(1, 10));
    let span2 = Span::new(Location::new(1, 5), Location::new(1, 15));

    let merged = span1.merge(span2);

    assert_eq!(merged.start, Location::new(1, 0));
    assert_eq!(merged.end, Location::new(1, 15));
}

#[test]
fn test_span_equality() {
    let span1 = Span::new(Location::new(1, 0), Location::new(1, 10));
    let span2 = Span::new(Location::new(1, 0), Location::new(1, 10));
    let span3 = Span::new(Location::new(1, 0), Location::new(1, 11));

    assert_eq!(span1, span2);
    assert_ne!(span1, span3);
}
