//! Semantic validation for Forge programs.

use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Program, Statement, UnaryOp};
use crate::error::{ForgeError, ForgeResult};
use crate::units::{Dimension, UnitRegistry};

/// Semantic analyzer for name resolution and dimensional checks.
#[derive(Debug, Default)]
pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    /// Creates a semantic analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Validates a program according to Forge v0.1 rules.
    pub fn validate(&self, program: &Program) -> ForgeResult<()> {
        let mut symbols: HashMap<String, Dimension> = HashMap::new();

        for statement in &program.statements {
            match statement {
                Statement::Assignment {
                    name,
                    value,
                    line,
                    column,
                } => {
                    let dimension = self.infer_dimension(value, &symbols, *line, *column)?;
                    symbols.insert(name.clone(), dimension);
                }
                Statement::Print {
                    value,
                    as_unit,
                    line,
                    column,
                } => {
                    let value_dimension = self.infer_dimension(value, &symbols, *line, *column)?;
                    if let Some(unit_expr) = as_unit {
                        let target = UnitRegistry::resolve_expr(unit_expr)
                            .map_err(|error| with_statement_span(error, *line, *column))?;
                        if !value_dimension.is_compatible_with(target.dimension) {
                            return Err(ForgeError::with_span(
                                format!(
                                    "Cannot convert expression to the requested unit.\nExpression dimension: {}\nTarget unit dimension: {}",
                                    value_dimension,
                                    target.dimension
                                ),
                                *line,
                                *column,
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn infer_dimension(
        &self,
        expression: &Expr,
        symbols: &HashMap<String, Dimension>,
        line: usize,
        column: usize,
    ) -> ForgeResult<Dimension> {
        match expression {
            Expr::Number(_) => Ok(Dimension::default()),
            Expr::Quantity { unit, .. } => UnitRegistry::resolve_expr(unit)
                .map(|resolved| resolved.dimension)
                .map_err(|error| with_statement_span(error, line, column)),
            Expr::Variable(name) => symbols.get(name).copied().ok_or_else(|| {
                ForgeError::with_span(
                    format!(
                        "Unknown variable '{}'. Assign the variable before using it.",
                        name
                    ),
                    line,
                    column,
                )
            }),
            Expr::Group(inner) => self.infer_dimension(inner, symbols, line, column),
            Expr::Unary { op, expression } => match op {
                UnaryOp::Negate => self.infer_dimension(expression, symbols, line, column),
            },
            Expr::Binary { left, op, right } => {
                let left_dimension = self.infer_dimension(left, symbols, line, column)?;
                let right_dimension = self.infer_dimension(right, symbols, line, column)?;

                match op {
                    BinaryOp::Add | BinaryOp::Subtract => {
                        if !left_dimension.is_compatible_with(right_dimension) {
                            let headline = if matches!(op, BinaryOp::Add) {
                                "Cannot add incompatible quantities."
                            } else {
                                "Cannot subtract incompatible quantities."
                            };
                            return Err(ForgeError::with_span(
                                format!(
                                    "{headline}\nLeft operand dimension: {}\nRight operand dimension: {}",
                                    left_dimension,
                                    right_dimension
                                ),
                                line,
                                column,
                            ));
                        }
                        Ok(left_dimension)
                    }
                    BinaryOp::Multiply => Ok(left_dimension.multiply(right_dimension)),
                    BinaryOp::Divide => Ok(left_dimension.divide(right_dimension)),
                    BinaryOp::Power => {
                        if !right_dimension.is_compatible_with(Dimension::default()) {
                            return Err(ForgeError::with_span(
                                format!(
                                    "Invalid exponent usage.\nExponent must be dimensionless.\nFound exponent dimension: {}",
                                    right_dimension
                                ),
                                line,
                                column,
                            ));
                        }

                        let exponent = extract_integer_exponent(right).ok_or_else(|| {
                            ForgeError::with_span(
                                "Invalid exponent usage.\nExponent must be an integer literal in Forge v0.1.",
                                line,
                                column,
                            )
                        })?;

                        Ok(left_dimension.powi(exponent))
                    }
                }
            }
        }
    }
}

fn extract_integer_exponent(expression: &Expr) -> Option<i32> {
    match expression {
        Expr::Number(value) => float_to_i32(*value),
        Expr::Unary {
            op: UnaryOp::Negate,
            expression,
        } => extract_integer_exponent(expression).map(|value| -value),
        Expr::Group(inner) => extract_integer_exponent(inner),
        _ => None,
    }
}

fn float_to_i32(value: f64) -> Option<i32> {
    let rounded = value.round();
    if (value - rounded).abs() > 1e-12 {
        return None;
    }
    if rounded < i32::MIN as f64 || rounded > i32::MAX as f64 {
        return None;
    }
    Some(rounded as i32)
}

fn with_statement_span(error: ForgeError, line: usize, column: usize) -> ForgeError {
    if error.line.is_some() && error.column.is_some() {
        error
    } else {
        ForgeError::with_span(error.message, line, column)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::semantic::SemanticAnalyzer;

    fn validate(source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|error| error.to_string())?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|error| error.to_string())?;
        let analyzer = SemanticAnalyzer::new();
        analyzer.validate(&program).map_err(|error| error.to_string())
    }

    #[test]
    fn rejects_undefined_variable_use() {
        let error = validate("print stress").expect_err("validation should fail");
        assert!(error.contains("Unknown variable 'stress'"));
        assert!(error.contains("line 1 column 1"));
    }

    #[test]
    fn rejects_addition_of_incompatible_dimensions() {
        let error = validate("x = 1 m + 2 s").expect_err("validation should fail");
        assert!(error.contains("Cannot add incompatible quantities"));
        assert!(error.contains("Left operand dimension: [L]"));
        assert!(error.contains("Right operand dimension: [T]"));
        assert!(error.contains("line 1 column 1"));
    }

    #[test]
    fn rejects_dimensionful_exponent() {
        let error = validate("x = 2 ^ 1 m").expect_err("validation should fail");
        assert!(error.contains("Invalid exponent usage"));
        assert!(error.contains("Exponent must be dimensionless"));
        assert!(error.contains("[L]"));
    }

    #[test]
    fn rejects_non_integer_exponent() {
        let error = validate("x = 2 ^ 1.5").expect_err("validation should fail");
        assert!(error.contains("Invalid exponent usage"));
        assert!(error.contains("integer literal"));
        assert!(error.contains("line 1 column 1"));
    }

    #[test]
    fn rejects_incompatible_print_conversion() {
        let source = "force = 12 kN\nprint force as MPa";
        let error = validate(source).expect_err("validation should fail");
        assert!(error.contains("Cannot convert expression to the requested unit"));
        assert!(error.contains("Expression dimension: [L M T^-2]"));
        assert!(error.contains("Target unit dimension: [L^-1 M T^-2]"));
        assert!(error.contains("line 2 column 1"));
    }

    #[test]
    fn rejects_unknown_units() {
        let error = validate("x = 10 cm").expect_err("validation should fail");
        assert!(error.contains("Unknown unit 'cm'"));
        assert!(error.contains("Supported units are"));
        assert!(error.contains("line 1 column 1"));
    }
}
