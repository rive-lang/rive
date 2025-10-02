//! RIR optimizer - performs optimization passes on RIR.

mod constant_folding;
mod dead_code_elimination;
mod pass;

pub use constant_folding::ConstantFoldingPass;
pub use dead_code_elimination::DeadCodeEliminationPass;
pub use pass::OptimizationPass;

use crate::RirModule;

/// Optimizer that applies multiple passes to RIR
pub struct Optimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl Optimizer {
    /// Creates a new optimizer with default passes
    #[must_use]
    pub fn new() -> Self {
        Self {
            passes: vec![
                Box::new(ConstantFoldingPass),
                Box::new(DeadCodeEliminationPass),
            ],
        }
    }

    /// Creates an empty optimizer with no passes
    #[must_use]
    pub fn empty() -> Self {
        Self { passes: Vec::new() }
    }

    /// Adds an optimization pass
    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>) -> &mut Self {
        self.passes.push(pass);
        self
    }

    /// Runs all optimization passes on the module
    ///
    /// Runs passes in order, repeating until no changes are made
    pub fn optimize(&self, module: &mut RirModule) {
        let max_iterations = 10; // Prevent infinite loops
        let mut iteration = 0;

        loop {
            let mut changed = false;
            iteration += 1;

            if iteration > max_iterations {
                eprintln!(
                    "Warning: Optimizer reached maximum iterations ({})",
                    max_iterations
                );
                break;
            }

            for pass in &self.passes {
                if pass.run(module) {
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    /// Runs optimization passes once (no iteration)
    pub fn optimize_once(&self, module: &mut RirModule) {
        for pass in &self.passes {
            pass.run(module);
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_core::type_system::TypeRegistry;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = Optimizer::new();
        assert_eq!(optimizer.passes.len(), 2);
    }

    #[test]
    fn test_empty_optimizer() {
        let optimizer = Optimizer::empty();
        assert_eq!(optimizer.passes.len(), 0);
    }

    #[test]
    fn test_optimize_empty_module() {
        let optimizer = Optimizer::new();
        let mut module = RirModule::new(TypeRegistry::new());
        optimizer.optimize(&mut module);
        // Should not crash
    }
}
