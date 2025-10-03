//! Rive Intermediate Representation (RIR)
//!
//! RIR is a Rust-oriented intermediate representation designed specifically for
//! generating high-quality Rust code. Unlike traditional IRs (LLVM, GCC RTL),
//! RIR preserves high-level structure while making compiler decisions explicit.
//!
//! # Design Philosophy
//!
//! - **Rust-First**: Designed for Rust generation, not machine code
//! - **Structured**: Preserves if/while/match, no goto or basic blocks
//! - **Explicit**: All compiler decisions are visible (memory ops, types)
//! - **Optimizable**: Easy to analyze and transform
//! - **Debuggable**: Human-readable for compiler development
//!
//! # Example
//!
//! ```rust
//! use rive_ir::{RirModule, RirFunction, RirBlock, ExprBuilder};
//! use rive_core::{type_system::TypeRegistry, span::{Span, Location}};
//!
//! let registry = TypeRegistry::new();
//! let module = RirModule::new(registry);
//! let span = Span::new(Location::new(1, 1), Location::new(1, 10));
//!
//! // Create a simple function
//! let func = RirFunction::new(
//!     "main".to_string(),
//!     vec![],
//!     rive_core::type_system::TypeId::UNIT,
//!     RirBlock::new(span),
//!     span,
//! );
//! ```

mod builder;
mod display;
mod expression;
mod lowering;
mod module;
mod statement;

// Re-export main types
pub use builder::{BlockBuilder, ExprBuilder, RirBuilder};
pub use expression::{BinaryOp, RirExpression, UnaryOp};
pub use lowering::AstLowering;
pub use module::{RirBlock, RirFunction, RirModule, RirParameter};
pub use statement::{RirPattern, RirStatement};
