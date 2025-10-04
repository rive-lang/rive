//! Display implementation for RIR block.

use std::fmt;

use crate::RirBlock;

impl fmt::Display for RirBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            writeln!(f, "  {stmt}")?;
        }
        if let Some(expr) = &self.final_expr {
            writeln!(f, "  {expr}")?;
        }
        Ok(())
    }
}
