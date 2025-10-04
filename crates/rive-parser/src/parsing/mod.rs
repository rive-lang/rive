//! Parsing implementation modules.

mod control_flow;
mod expressions;
mod functions;
mod helpers;
mod parser;
mod primary;
mod statements;
mod types;

pub use parser::Parser;
