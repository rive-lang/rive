// Control flow lowering from AST to RIR.
//
// This file is included in lowering.rs and implements lowering for control flow constructs:
// - If expressions/statements
// - Loops (while, for, loop)
// - Break and continue statements
// - Match expressions
// - Range expressions

impl AstLowering {
    /// Lowers an if expression to RIR.
    ///
    /// For expression context (needs value), generates If expression.
    /// For statement context, generates If statement.
    pub(crate) fn lower_if_expr(&mut self, if_expr: &If) -> Result<RirExpression> {
        let condition = Box::new(self.lower_expression(&if_expr.condition)?);
        let then_block = self.lower_block(&if_expr.then_block)?;

        // If must have else to be an expression
        let else_block = if let Some(else_blk) = &if_expr.else_block {
            self.lower_block(else_blk)?
        } else {
            // This should have been caught by semantic analysis
            return Err(Error::Semantic(
                "If expression must have else branch".to_string(),
            ));
        };

        // Get result type from the semantic analysis
        // For now, we use the type from the then block's last expression
        let result_type = then_block
            .final_expr
            .as_ref()
            .map(|e| e.type_id())
            .unwrap_or(self.type_registry.get_unit());

        Ok(RirExpression::If {
            condition,
            then_block,
            else_block,
            result_type,
            span: if_expr.span,
        })
    }

    /// Lowers an if as a statement.
    #[allow(dead_code)] // Will be used in Phase 6
    pub(crate) fn lower_if_stmt(&mut self, if_expr: &If) -> Result<RirStatement> {
        let condition = Box::new(self.lower_expression(&if_expr.condition)?);
        let then_block = self.lower_block(&if_expr.then_block)?;

        // Handle else-if chain by converting to nested if-else
        let else_block = if !if_expr.else_if_branches.is_empty() || if_expr.else_block.is_some() {
            // Build nested if-else from else-if chain
            let mut current_else = if_expr.else_block.as_ref().map(|b| self.lower_block(b)).transpose()?;

            // Process else-if branches in reverse order
            for else_if in if_expr.else_if_branches.iter().rev() {
                let else_if_cond = Box::new(self.lower_expression(&else_if.condition)?);
                let else_if_then = self.lower_block(&else_if.block)?;

                let nested_if = RirStatement::If {
                    condition: else_if_cond,
                    then_block: else_if_then,
                    else_block: current_else,
                    span: else_if.span,
                };

                current_else = Some(RirBlock {
                    statements: vec![nested_if],
                    final_expr: None,
                    span: else_if.span,
                });
            }

            current_else
        } else {
            None
        };

        Ok(RirStatement::If {
            condition,
            then_block,
            else_block,
            span: if_expr.span,
        })
    }

    /// Lowers a while loop to RIR.
    pub(crate) fn lower_while(&mut self, while_loop: &While) -> Result<RirStatement> {
        let condition = Box::new(self.lower_expression(&while_loop.condition)?);
        
        // Enter loop context for label generation
        let label = self.enter_loop();
        
        let body = self.lower_block(&while_loop.body)?;
        
        // Exit loop context
        self.exit_loop();

        Ok(RirStatement::While {
            condition,
            body,
            label,
            span: while_loop.span,
        })
    }

    /// Lowers a for loop to RIR.
    pub(crate) fn lower_for(&mut self, for_loop: &For) -> Result<RirStatement> {
        // Extract range from iterable
        let (start, end, inclusive) = match &*for_loop.iterable {
            Expression::Range(range) => {
                let start = self.lower_expression(&range.start)?;
                let end = self.lower_expression(&range.end)?;
                (Box::new(start), Box::new(end), range.inclusive)
            }
            _ => {
                return Err(Error::Semantic(
                    "For loop iterable must be a range".to_string(),
                ));
            }
        };

        // Enter new scope for loop variable
        self.enter_scope();

        // Define loop variable (Int type for ranges, immutable)
        let int_type = self.type_registry.get_int();
        self.define_variable(for_loop.variable.clone(), int_type, false);

        // Enter loop context for label generation
        let label = self.enter_loop();

        let body = self.lower_block(&for_loop.body)?;

        // Exit loop context
        self.exit_loop();

        // Exit scope
        self.exit_scope();

        Ok(RirStatement::For {
            variable: for_loop.variable.clone(),
            start,
            end,
            inclusive,
            body,
            label,
            span: for_loop.span,
        })
    }

    /// Lowers an infinite loop to RIR.
    pub(crate) fn lower_loop(&mut self, loop_expr: &Loop) -> Result<RirStatement> {
        // Enter loop context for label generation
        let label = self.enter_loop();

        let body = self.lower_block(&loop_expr.body)?;

        // Exit loop context
        self.exit_loop();

        Ok(RirStatement::Loop {
            body,
            label,
            span: loop_expr.span,
        })
    }

    /// Lowers a break statement to RIR.
    pub(crate) fn lower_break(&mut self, break_stmt: &Break) -> Result<RirStatement> {
        let depth = break_stmt.depth.unwrap_or(1) as usize;
        
        // Convert depth to label
        let label = self.get_loop_label(depth)?;

        let value = break_stmt
            .value
            .as_ref()
            .map(|v| self.lower_expression(v))
            .transpose()?
            .map(Box::new);

        Ok(RirStatement::Break {
            label,
            value,
            span: break_stmt.span,
        })
    }

    /// Lowers a continue statement to RIR.
    pub(crate) fn lower_continue(&mut self, continue_stmt: &Continue) -> Result<RirStatement> {
        let depth = continue_stmt.depth.unwrap_or(1) as usize;
        
        // Convert depth to label
        let label = self.get_loop_label(depth)?;

        Ok(RirStatement::Continue {
            label,
            span: continue_stmt.span,
        })
    }

    /// Lowers a match expression to RIR.
    pub(crate) fn lower_match_expr(&mut self, match_expr: &Match) -> Result<RirExpression> {
        let scrutinee = Box::new(self.lower_expression(&match_expr.scrutinee)?);

        let arms: Result<Vec<_>> = match_expr
            .arms
            .iter()
            .map(|arm| {
                let pattern = self.lower_pattern(&arm.pattern)?;
                let body = Box::new(self.lower_expression(&arm.body)?);
                Ok((pattern, body))
            })
            .collect();

        let arms = arms?;

        // Get result type from first arm (all arms have same type after semantic analysis)
        let result_type = arms
            .first()
            .map(|(_, expr)| expr.type_id())
            .unwrap_or(self.type_registry.get_unit());

        Ok(RirExpression::Match {
            scrutinee,
            arms,
            result_type,
            span: match_expr.span,
        })
    }

    /// Lowers a match as a statement.
    #[allow(dead_code)] // Will be used in Phase 6
    pub(crate) fn lower_match_stmt(&mut self, match_expr: &Match) -> Result<RirStatement> {
        let scrutinee = Box::new(self.lower_expression(&match_expr.scrutinee)?);

        let arms: Result<Vec<_>> = match_expr
            .arms
            .iter()
            .map(|arm| {
                let pattern = self.lower_pattern(&arm.pattern)?;
                
                // Convert expression to block
                let body_expr = self.lower_expression(&arm.body)?;
                let body = RirBlock {
                    statements: vec![RirStatement::Expression {
                        expr: Box::new(body_expr),
                        span: arm.span,
                    }],
                    final_expr: None,
                    span: arm.span,
                };
                
                Ok((pattern, body))
            })
            .collect();

        Ok(RirStatement::Match {
            scrutinee,
            arms: arms?,
            span: match_expr.span,
        })
    }

    /// Lowers a pattern to RIR.
    fn lower_pattern(&self, pattern: &Pattern) -> Result<RirPattern> {
        Ok(match pattern {
            Pattern::Integer { value, span } => RirPattern::IntLiteral {
                value: *value,
                span: *span,
            },
            Pattern::Float { value, span } => RirPattern::FloatLiteral {
                value: *value,
                span: *span,
            },
            Pattern::String { value, span } => RirPattern::StringLiteral {
                value: value.clone(),
                span: *span,
            },
            Pattern::Boolean { value, span } => RirPattern::BoolLiteral {
                value: *value,
                span: *span,
            },
            Pattern::Null { span: _ } => {
                return Err(Error::Semantic(
                    "Null patterns not yet supported".to_string(),
                ));
            }
            Pattern::Wildcard { span } => RirPattern::Wildcard { span: *span },
        })
    }

    /// Lowers a range expression to RIR.
    ///
    /// For Phase 1, ranges are only used in for loops, so we don't need
    /// a dedicated Range RIR node.
    pub(crate) fn lower_range(&mut self, _range: &Range) -> Result<RirExpression> {
        // Range should only appear in for loop context
        // If we reach here, it's an error
        Err(Error::Semantic(
            "Range expressions can only be used in for loops".to_string(),
        ))
    }

    /// Enters a new loop scope and returns the label for this loop.
    fn enter_loop(&mut self) -> Option<String> {
        self.loop_depth += 1;
        
        // Generate label without the ' prefix (will be added in codegen)
        let label = Some(format!("loop_{}", self.loop_depth));
        
        self.loop_labels.push(label.clone());
        label
    }

    /// Exits the current loop scope.
    fn exit_loop(&mut self) {
        self.loop_labels.pop();
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }

    /// Gets the loop label at the specified depth (1 = current loop).
    fn get_loop_label(&self, depth: usize) -> Result<Option<String>> {
        if depth == 0 || depth > self.loop_labels.len() {
            return Err(Error::Semantic(format!(
                "Invalid loop depth: {}",
                depth
            )));
        }

        // depth = 1 means current loop (last in stack)
        // depth = 2 means parent loop (second from last)
        let index = self.loop_labels.len() - depth;
        Ok(self.loop_labels[index].clone())
    }
}

