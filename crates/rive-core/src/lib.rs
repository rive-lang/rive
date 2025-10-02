//! Core types and utilities for the Rive language compiler.
//!
//! This crate provides fundamental types, error handling, and shared utilities
//! used across all compiler stages.

pub mod error;
pub mod span;
pub mod type_system;
pub mod types;

pub use error::{Error, Result};
pub use span::Span;
pub use type_system::{MemoryStrategy, TypeId, TypeKind, TypeMetadata, TypeRegistry};
