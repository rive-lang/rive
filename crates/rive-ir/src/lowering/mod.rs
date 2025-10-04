//! AST to RIR lowering implementation.

mod control_flow;
mod core;
mod expressions;
mod helpers;
mod program;
mod statements;

pub use core::AstLowering;
