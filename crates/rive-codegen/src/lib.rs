//! Code generation for the Rive language.
//!
//! This crate generates Rust code from Rive Intermediate Representation (RIR).

mod generator;

pub use generator::CodeGenerator;

use rive_core::Result;
use rive_ir::RirModule;

/// Generates Rust code from a RIR module.
///
/// # Arguments
/// * `module` - The RIR module to generate code from
///
/// # Returns
/// Generated Rust source code as a string
///
/// # Errors
/// Returns an error if code generation fails
///
/// # Examples
/// ```
/// use rive_codegen::CodeGenerator;
/// use rive_core::type_system::TypeRegistry;
/// use rive_ir::RirModule;
///
/// let type_registry = TypeRegistry::new();
/// let module = RirModule::new(type_registry);
/// let mut generator = CodeGenerator::new();
/// let rust_code = generator.generate(&module).unwrap();
/// ```
pub fn generate(module: &RirModule) -> Result<String> {
    let mut generator = CodeGenerator::new();
    generator.generate(module)
}
