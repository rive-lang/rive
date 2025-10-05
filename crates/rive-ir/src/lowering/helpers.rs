//! Helper functions for AST lowering.

use crate::lowering::core::AstLowering;
use crate::{BinaryOp, RirBlock, RirExpression, RirStatement, UnaryOp};
use rive_core::type_system::{MemoryStrategy, TypeId};
use rive_parser::ast::{BinaryOperator, UnaryOperator};

impl AstLowering {
    /// Converts AST binary operator to RIR binary operator.
    pub(crate) fn lower_binary_op(&self, op: &BinaryOperator) -> BinaryOp {
        match op {
            BinaryOperator::Add => BinaryOp::Add,
            BinaryOperator::Subtract => BinaryOp::Subtract,
            BinaryOperator::Multiply => BinaryOp::Multiply,
            BinaryOperator::Divide => BinaryOp::Divide,
            BinaryOperator::Modulo => BinaryOp::Modulo,
            BinaryOperator::Equal => BinaryOp::Equal,
            BinaryOperator::NotEqual => BinaryOp::NotEqual,
            BinaryOperator::Less => BinaryOp::LessThan,
            BinaryOperator::LessEqual => BinaryOp::LessEqual,
            BinaryOperator::Greater => BinaryOp::GreaterThan,
            BinaryOperator::GreaterEqual => BinaryOp::GreaterEqual,
            BinaryOperator::And => BinaryOp::And,
            BinaryOperator::Or => BinaryOp::Or,
        }
    }

    /// Converts AST unary operator to RIR unary operator.
    pub(crate) fn lower_unary_op(&self, op: &UnaryOperator) -> UnaryOp {
        match op {
            UnaryOperator::Negate => UnaryOp::Negate,
            UnaryOperator::Not => UnaryOp::Not,
        }
    }

    /// Infers the result type of a binary operation.
    pub(crate) fn infer_binary_result_type(
        &self,
        left: &RirExpression,
        _right: &RirExpression,
        op: BinaryOp,
    ) -> TypeId {
        if op.is_comparison() || op.is_logical() {
            TypeId::BOOL
        } else {
            left.type_id()
        }
    }

    /// Determines the memory strategy for a given type.
    pub(crate) const fn determine_memory_strategy(&self, type_id: TypeId) -> MemoryStrategy {
        match type_id {
            TypeId::INT | TypeId::FLOAT | TypeId::BOOL | TypeId::UNIT => MemoryStrategy::Copy,
            _ => MemoryStrategy::CoW,
        }
    }

    /// Gets or creates a nullable version of the given type.
    /// If the type is already nullable, returns it as-is.
    pub(crate) fn get_or_create_nullable(&mut self, type_id: TypeId) -> TypeId {
        // Check if already nullable
        if let Some(meta) = self.type_registry.get(type_id)
            && matches!(meta.kind, rive_core::type_system::TypeKind::Optional { .. })
        {
            return type_id; // Already nullable
        }

        // Create nullable version
        self.type_registry.create_optional(type_id)
    }

    /// Infers the result type of a loop by finding break statements with values.
    /// Returns Optional<T> where T is the break value type, or Optional<Unit> if no break with value.
    pub(crate) fn infer_loop_result_type(&mut self, body: &RirBlock) -> TypeId {
        // Look for break statements with values in the body
        for stmt in &body.statements {
            if let Some(inner_type) = Self::find_break_value_type(stmt) {
                // Return Optional<T> where T is the break value type
                return self.get_or_create_nullable(inner_type);
            }
        }

        // No break with value found, return Optional<Unit> (nullable)
        self.get_or_create_nullable(TypeId::UNIT)
    }

    /// Find break statement with value and return its type.
    fn find_break_value_type(stmt: &RirStatement) -> Option<TypeId> {
        match stmt {
            RirStatement::Break {
                value: Some(expr), ..
            } => Some(expr.type_id()),
            RirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                // Check both branches
                for s in &then_block.statements {
                    if let Some(type_id) = Self::find_break_value_type(s) {
                        return Some(type_id);
                    }
                }
                if let Some(else_b) = else_block {
                    for s in &else_b.statements {
                        if let Some(type_id) = Self::find_break_value_type(s) {
                            return Some(type_id);
                        }
                    }
                }
                None
            }
            RirStatement::Match { arms, .. } => {
                // Check all match arms
                for (_, body) in arms {
                    for s in &body.statements {
                        if let Some(type_id) = Self::find_break_value_type(s) {
                            return Some(type_id);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}
