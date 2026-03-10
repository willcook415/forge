//! Interpreter for executing Forge AST nodes.

use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Program, Statement, UnaryOp, UnitExpr};
use crate::error::{ForgeError, ForgeResult};
use crate::units::{Dimension, Quantity, UnitRegistry};

/// Interpreter for Forge v0.1.
#[derive(Debug, Default)]
pub struct Interpreter {
    environment: HashMap<String, Quantity>,
}

impl Interpreter {
    /// Creates an interpreter instance.
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
        }
    }

    /// Evaluates a program and collects printable outputs.
    pub fn evaluate(&mut self, program: &Program) -> ForgeResult<Vec<String>> {
        let mut output = Vec::new();

        for statement in &program.statements {
            match statement {
                Statement::Assignment {
                    name,
                    value,
                    line,
                    column,
                } => {
                    let result = self.evaluate_expr(value, *line, *column)?;
                    self.environment.insert(name.clone(), result);
                }
                Statement::Print {
                    value,
                    as_unit,
                    line,
                    column,
                } => {
                    let result = self.evaluate_expr(value, *line, *column)?;
                    if let Some(target_expr) = as_unit {
                        let target = UnitRegistry::resolve_expr(target_expr)
                            .map_err(|error| with_statement_span(error, *line, *column))?;
                        let converted = result
                            .convert_to(target)
                            .map_err(|error| with_statement_span(error, *line, *column))?;
                        output.push(format!(
                            "{} {}",
                            format_number(converted),
                            format_unit_expr(target_expr)
                        ));
                    } else if result.dimension == Dimension::default() {
                        output.push(format_number(result.value_si));
                    } else {
                        output.push(format!("{} {}", format_number(result.value_si), result.dimension));
                    }
                }
            }
        }

        Ok(output)
    }

    fn evaluate_expr(&self, expr: &Expr, line: usize, column: usize) -> ForgeResult<Quantity> {
        match expr {
            Expr::Number(value) => Ok(Quantity::from_si(*value, Dimension::default())),
            Expr::Quantity { value, unit } => {
                let resolved = UnitRegistry::resolve_expr(unit)
                    .map_err(|error| with_statement_span(error, line, column))?;
                Ok(Quantity::from_unit(*value, resolved))
            }
            Expr::Variable(name) => self.environment.get(name).copied().ok_or_else(|| {
                ForgeError::with_span(
                    format!(
                        "Unknown variable '{}'. Assign the variable before using it.",
                        name
                    ),
                    line,
                    column,
                )
            }),
            Expr::Group(inner) => self.evaluate_expr(inner, line, column),
            Expr::Unary { op, expression } => match op {
                UnaryOp::Negate => {
                    let value = self.evaluate_expr(expression, line, column)?;
                    Ok(Quantity::from_si(-value.value_si, value.dimension))
                }
            },
            Expr::Binary { left, op, right } => {
                let left_value = self.evaluate_expr(left, line, column)?;
                let right_value = self.evaluate_expr(right, line, column)?;

                match op {
                    BinaryOp::Add => left_value
                        .checked_add(&right_value)
                        .map_err(|error| with_statement_span(error, line, column)),
                    BinaryOp::Subtract => left_value
                        .checked_sub(&right_value)
                        .map_err(|error| with_statement_span(error, line, column)),
                    BinaryOp::Multiply => Ok(left_value.multiply(&right_value)),
                    BinaryOp::Divide => Ok(left_value.divide(&right_value)),
                    BinaryOp::Power => {
                        if right_value.dimension != Dimension::default() {
                            return Err(ForgeError::with_span(
                                format!(
                                    "Invalid exponent usage.\nExponent must be dimensionless.\nFound exponent dimension: {}",
                                    right_value.dimension
                                ),
                                line,
                                column,
                            ));
                        }

                        let exponent = float_to_i32(right_value.value_si).ok_or_else(|| {
                            ForgeError::with_span(
                                "Invalid exponent usage.\nExponent must be an integer in Forge v0.1.",
                                line,
                                column,
                            )
                        })?;
                        Ok(Quantity::from_si(
                            left_value.value_si.powi(exponent),
                            left_value.dimension.powi(exponent),
                        ))
                    }
                }
            }
        }
    }
}

fn format_unit_expr(expr: &UnitExpr) -> String {
    match expr {
        UnitExpr::Unit(symbol) => symbol.clone(),
        UnitExpr::Multiply(left, right) => {
            format!("{}*{}", format_unit_expr(left), format_unit_expr(right))
        }
        UnitExpr::Divide(left, right) => {
            format!("{}/{}", format_unit_expr(left), format_unit_expr(right))
        }
        UnitExpr::Power { base, exponent } => format!("{base}^{exponent}"),
    }
}

fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }

    let significant_figures: i32 = 6;
    let absolute = value.abs();
    let exponent = absolute.log10().floor() as i32;
    let scale_exponent = significant_figures - 1 - exponent;

    let rounded = if scale_exponent >= 0 {
        let factor = 10f64.powi(scale_exponent);
        (value * factor).round() / factor
    } else {
        let factor = 10f64.powi(-scale_exponent);
        (value / factor).round() * factor
    };

    let normalized = if rounded == -0.0 { 0.0 } else { rounded };
    let decimals = if scale_exponent > 0 {
        scale_exponent as usize
    } else {
        0
    };

    let mut text = format!("{normalized:.decimals$}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
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
    use std::fs;
    use std::path::PathBuf;

    use crate::interpreter::Interpreter;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::semantic::SemanticAnalyzer;

    fn run_source(source: &str) -> Result<Vec<String>, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|error| error.to_string())?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|error| error.to_string())?;
        let analyzer = SemanticAnalyzer::new();
        analyzer
            .validate(&program)
            .map_err(|error| error.to_string())?;
        let mut interpreter = Interpreter::new();
        interpreter.evaluate(&program).map_err(|error| error.to_string())
    }

    fn run_example(filename: &str) -> Result<Vec<String>, String> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(filename);
        let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
        run_source(&source)
    }

    #[test]
    fn axial_stress_example_output() {
        let output = run_example("axial_stress.forge").expect("example should run");
        assert_eq!(output, vec!["40 MPa"]);
    }

    #[test]
    fn hydrostatic_pressure_example_output() {
        let output = run_example("hydrostatic_pressure.forge").expect("example should run");
        assert_eq!(output, vec!["24.525 kPa"]);
    }

    #[test]
    fn kinetic_energy_example_output() {
        let output = run_example("kinetic_energy.forge").expect("example should run");
        assert_eq!(output, vec!["240000 [L^2 M T^-2]"]);
    }

    #[test]
    fn stress_print_variants_example_output() {
        let output = run_example("stress_print_variants.forge").expect("example should run");
        assert_eq!(output, vec!["40000000 [L^-1 M T^-2]", "40 MPa"]);
    }

    #[test]
    fn unit_conversion_example_output() {
        let output = run_example("unit_conversion.forge").expect("example should run");
        assert_eq!(output, vec!["2.5 m"]);
    }

    #[test]
    fn evaluates_arithmetic_operations() {
        let source = "a = 10\nb = 3\nprint a + b\nprint a - b\nprint a * b\nprint a / b\nprint -b\nprint 2^3";
        let output = run_source(source).expect("script should run");
        assert_eq!(output, vec!["13", "7", "30", "3.33333", "-3", "8"]);
    }

    #[test]
    fn evaluates_quantity_arithmetic_and_dimension_composition() {
        let source = "force = 12 kN\narea = 300 mm^2\nstress = force / area\nprint stress";
        let output = run_source(source).expect("script should run");
        assert_eq!(output, vec!["40000000 [L^-1 M T^-2]"]);
    }

    #[test]
    fn evaluates_unit_conversion_in_print() {
        let source = "force = 12 kN\nprint force as N";
        let output = run_source(source).expect("script should run");
        assert_eq!(output, vec!["12000 N"]);
    }

    #[test]
    fn invalid_dimensions_example_reports_error() {
        let error = run_example("invalid_dimensions.forge").expect_err("example should fail");
        assert!(error.contains("Cannot add incompatible quantities"));
        assert!(error.contains("Left operand dimension: [L]"));
        assert!(error.contains("Right operand dimension: [T]"));
        assert!(error.contains("line 4 column 1"));
    }

    #[test]
    fn reports_invalid_conversion_error() {
        let source = "force = 12 kN\nprint force as MPa";
        let error = run_source(source).expect_err("script should fail");
        assert!(error.contains("Cannot convert expression to the requested unit"));
        assert!(error.contains("Expression dimension: [L M T^-2]"));
        assert!(error.contains("Target unit dimension: [L^-1 M T^-2]"));
        assert!(error.contains("line 2 column 1"));
    }

    #[test]
    fn formats_output_to_six_significant_figures() {
        let output = run_source("x = 12.3456789\nprint x").expect("script should run");
        assert_eq!(output, vec!["12.3457"]);
    }

    #[test]
    fn removes_floating_point_noise_in_output() {
        let output = run_source("x = 40.0000000001\nprint x").expect("script should run");
        assert_eq!(output, vec!["40"]);
    }

    #[test]
    fn formats_small_values_consistently() {
        let output = run_source("x = 0.000123456789\nprint x").expect("script should run");
        assert_eq!(output, vec!["0.000123457"]);
    }
}
