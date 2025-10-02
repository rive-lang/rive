//! Optimization pass trait.

use crate::RirModule;

/// Trait for optimization passes
pub trait OptimizationPass {
    /// Returns the name of the pass
    fn name(&self) -> &str;

    /// Runs the optimization pass on a module
    ///
    /// Returns `true` if any changes were made, `false` otherwise
    fn run(&self, module: &mut RirModule) -> bool;
}
