//! Type checking implementation modules.

mod control_flow;
mod core;
mod expressions;
mod helpers;
mod loops;
mod patterns;
mod program;
mod statements;

pub use core::TypeChecker;
