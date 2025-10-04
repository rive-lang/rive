//! Rive Intermediate Representation (RIR).
//!
//! This crate defines the intermediate representation used between AST and code generation.
//! RIR is a lower-level representation that is easier to optimize and generate code from.

mod builder;
mod display;
mod expression;
mod lowering;
mod module;
mod statement;

pub use builder::{BlockBuilder, ExprBuilder, RirBuilder};
pub use expression::{BinaryOp, RirExpression, UnaryOp};
pub use lowering::AstLowering;
pub use module::{RirBlock, RirFunction, RirModule, RirParameter};
pub use statement::{RirPattern, RirStatement};

use rive_core::Result;
use rive_core::type_system::TypeRegistry;
use rive_parser::ast::Program;

/// Lowers AST to RIR.
///
/// # Arguments
/// * `program` - The parsed AST program
/// * `type_registry` - Type registry from semantic analysis
///
/// # Returns
/// A RIR module ready for code generation
///
/// # Errors
/// Returns an error if lowering fails
pub fn lower(program: &Program, type_registry: TypeRegistry) -> Result<RirModule> {
    let mut lowering = AstLowering::new(type_registry);
    lowering.lower_program(program)
}
