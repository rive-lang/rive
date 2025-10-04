//! Loop context management for type checking.

use rive_core::type_system::TypeId;

/// Context for loop type checking.
#[derive(Debug, Clone)]
pub struct LoopContext {
    /// Type that this loop returns (if break has value)
    pub break_type: Option<TypeId>,
    /// Whether break statement was seen
    pub has_break: bool,
}

impl LoopContext {
    /// Creates a new loop context.
    pub fn new() -> Self {
        Self {
            break_type: None,
            has_break: false,
        }
    }
}

impl Default for LoopContext {
    fn default() -> Self {
        Self::new()
    }
}
