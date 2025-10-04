//! Code generation implementation modules.

mod control_flow;
mod core;
mod expressions;
mod inline;
mod labels;
mod statements;
mod types;
mod utils;

pub use core::CodeGenerator;
