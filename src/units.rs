//! Unit, dimension, and quantity types for Forge v0.1.

use std::fmt;

use crate::ast::UnitExpr;
use crate::error::{ForgeError, ForgeResult};

/// Dimension vector using base dimensions Length, Mass, and Time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dimension {
    /// Exponent for Length (L).
    pub length: i32,
    /// Exponent for Mass (M).
    pub mass: i32,
    /// Exponent for Time (T).
    pub time: i32,
}

impl Dimension {
    /// Creates a dimension vector from base exponents.
    pub const fn new(length: i32, mass: i32, time: i32) -> Self {
        Self { length, mass, time }
    }

    /// Combines dimensions for multiplication.
    pub fn multiply(self, other: Self) -> Self {
        Self {
            length: self.length + other.length,
            mass: self.mass + other.mass,
            time: self.time + other.time,
        }
    }

    /// Combines dimensions for division.
    pub fn divide(self, other: Self) -> Self {
        Self {
            length: self.length - other.length,
            mass: self.mass - other.mass,
            time: self.time - other.time,
        }
    }

    /// Raises a dimension vector to an integer exponent.
    pub fn powi(self, exponent: i32) -> Self {
        Self {
            length: self.length * exponent,
            mass: self.mass * exponent,
            time: self.time * exponent,
        }
    }

    /// Returns true when dimensions are identical.
    pub fn is_compatible_with(self, other: Self) -> bool {
        self == other
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        append_dimension_part(&mut parts, "L", self.length);
        append_dimension_part(&mut parts, "M", self.mass);
        append_dimension_part(&mut parts, "T", self.time);

        if parts.is_empty() {
            write!(f, "[1]")
        } else {
            write!(f, "[{}]", parts.join(" "))
        }
    }
}

fn append_dimension_part(parts: &mut Vec<String>, symbol: &str, exponent: i32) {
    if exponent == 0 {
        return;
    }
    if exponent == 1 {
        parts.push(symbol.to_string());
    } else {
        parts.push(format!("{symbol}^{exponent}"));
    }
}

/// Runtime quantity represented in SI-scaled value plus dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    /// Scalar value in base SI scale.
    pub value_si: f64,
    /// Physical dimension.
    pub dimension: Dimension,
}

impl Quantity {
    /// Creates a quantity from an SI value and dimension.
    pub fn from_si(value_si: f64, dimension: Dimension) -> Self {
        Self { value_si, dimension }
    }

    /// Creates a quantity from a value in the provided unit.
    pub fn from_unit(value: f64, unit: ResolvedUnit) -> Self {
        Self {
            value_si: value * unit.scale_to_si,
            dimension: unit.dimension,
        }
    }

    /// Returns true if two quantities can be added/subtracted.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.dimension.is_compatible_with(other.dimension)
    }

    /// Converts SI value to a target unit value.
    pub fn convert_to(&self, target: ResolvedUnit) -> ForgeResult<f64> {
        if !self.dimension.is_compatible_with(target.dimension) {
            return Err(ForgeError::new(format!(
                "Cannot convert incompatible quantities.\nSource quantity dimension: {}\nTarget unit dimension: {}",
                self.dimension,
                target.dimension
            )));
        }

        Ok(self.value_si / target.scale_to_si)
    }

    /// Adds quantities with matching dimensions.
    pub fn checked_add(&self, other: &Self) -> ForgeResult<Self> {
        if !self.is_compatible_with(other) {
            return Err(ForgeError::new(format!(
                "Cannot add incompatible quantities.\nLeft operand dimension: {}\nRight operand dimension: {}",
                self.dimension,
                other.dimension
            )));
        }

        Ok(Self {
            value_si: self.value_si + other.value_si,
            dimension: self.dimension,
        })
    }

    /// Subtracts quantities with matching dimensions.
    pub fn checked_sub(&self, other: &Self) -> ForgeResult<Self> {
        if !self.is_compatible_with(other) {
            return Err(ForgeError::new(format!(
                "Cannot subtract incompatible quantities.\nLeft operand dimension: {}\nRight operand dimension: {}",
                self.dimension,
                other.dimension
            )));
        }

        Ok(Self {
            value_si: self.value_si - other.value_si,
            dimension: self.dimension,
        })
    }

    /// Multiplies quantities and composes dimensions.
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            value_si: self.value_si * other.value_si,
            dimension: self.dimension.multiply(other.dimension),
        }
    }

    /// Divides quantities and composes dimensions.
    pub fn divide(&self, other: &Self) -> Self {
        Self {
            value_si: self.value_si / other.value_si,
            dimension: self.dimension.divide(other.dimension),
        }
    }
}

/// Unit definition in v0.1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitDef {
    /// Canonical unit symbol.
    pub symbol: &'static str,
    /// Multiplier to convert unit value to SI base scale.
    pub scale_to_si: f64,
    /// Physical dimension for the unit.
    pub dimension: Dimension,
}

/// A fully resolved (possibly composed) unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedUnit {
    /// Multiplier to convert a value in this unit to SI.
    pub scale_to_si: f64,
    /// Physical dimension for the resolved unit.
    pub dimension: Dimension,
}

impl ResolvedUnit {
    fn from_def(def: UnitDef) -> Self {
        Self {
            scale_to_si: def.scale_to_si,
            dimension: def.dimension,
        }
    }

    fn multiply(self, other: Self) -> Self {
        Self {
            scale_to_si: self.scale_to_si * other.scale_to_si,
            dimension: self.dimension.multiply(other.dimension),
        }
    }

    fn divide(self, other: Self) -> Self {
        Self {
            scale_to_si: self.scale_to_si / other.scale_to_si,
            dimension: self.dimension.divide(other.dimension),
        }
    }

    fn powi(self, exponent: i32) -> Self {
        Self {
            scale_to_si: self.scale_to_si.powi(exponent),
            dimension: self.dimension.powi(exponent),
        }
    }
}

/// Built-in registry for the v0.1 unit set.
pub struct UnitRegistry;

impl UnitRegistry {
    /// Resolves a unit symbol to a known unit definition.
    pub fn resolve(symbol: &str) -> Option<UnitDef> {
        match symbol {
            "m" => Some(UnitDef {
                symbol: "m",
                scale_to_si: 1.0,
                dimension: Dimension::new(1, 0, 0),
            }),
            "mm" => Some(UnitDef {
                symbol: "mm",
                scale_to_si: 0.001,
                dimension: Dimension::new(1, 0, 0),
            }),
            "s" => Some(UnitDef {
                symbol: "s",
                scale_to_si: 1.0,
                dimension: Dimension::new(0, 0, 1),
            }),
            "kg" => Some(UnitDef {
                symbol: "kg",
                scale_to_si: 1.0,
                dimension: Dimension::new(0, 1, 0),
            }),
            "N" => Some(UnitDef {
                symbol: "N",
                scale_to_si: 1.0,
                dimension: Dimension::new(1, 1, -2),
            }),
            "kN" => Some(UnitDef {
                symbol: "kN",
                scale_to_si: 1000.0,
                dimension: Dimension::new(1, 1, -2),
            }),
            "Pa" => Some(UnitDef {
                symbol: "Pa",
                scale_to_si: 1.0,
                dimension: Dimension::new(-1, 1, -2),
            }),
            "kPa" => Some(UnitDef {
                symbol: "kPa",
                scale_to_si: 1000.0,
                dimension: Dimension::new(-1, 1, -2),
            }),
            "MPa" => Some(UnitDef {
                symbol: "MPa",
                scale_to_si: 1_000_000.0,
                dimension: Dimension::new(-1, 1, -2),
            }),
            _ => None,
        }
    }

    /// Resolves a parsed unit expression to scale and dimensions.
    pub fn resolve_expr(expr: &UnitExpr) -> ForgeResult<ResolvedUnit> {
        match expr {
            UnitExpr::Unit(symbol) => Self::resolve(symbol)
                .map(ResolvedUnit::from_def)
                .ok_or_else(|| ForgeError::new(unknown_unit_message(symbol))),
            UnitExpr::Multiply(left, right) => {
                let left_resolved = Self::resolve_expr(left)?;
                let right_resolved = Self::resolve_expr(right)?;
                Ok(left_resolved.multiply(right_resolved))
            }
            UnitExpr::Divide(left, right) => {
                let left_resolved = Self::resolve_expr(left)?;
                let right_resolved = Self::resolve_expr(right)?;
                Ok(left_resolved.divide(right_resolved))
            }
            UnitExpr::Power { base, exponent } => {
                let base_unit = Self::resolve(base)
                    .map(ResolvedUnit::from_def)
                    .ok_or_else(|| ForgeError::new(unknown_unit_message(base)))?;
                Ok(base_unit.powi(*exponent as i32))
            }
        }
    }
}

fn unknown_unit_message(symbol: &str) -> String {
    format!(
        "Unknown unit '{}'. Supported units are: m, mm, s, kg, N, kN, Pa, kPa, MPa.",
        symbol
    )
}

#[cfg(test)]
mod tests {
    use crate::ast::UnitExpr;
    use crate::units::{Dimension, Quantity, UnitRegistry};

    fn approx_equal(left: f64, right: f64) {
        let diff = (left - right).abs();
        assert!(
            diff < 1e-10,
            "values differ: left={left} right={right} diff={diff}"
        );
    }

    #[test]
    fn looks_up_builtin_units() {
        let m = UnitRegistry::resolve("m").expect("m should exist");
        assert_eq!(m.scale_to_si, 1.0);
        assert_eq!(m.dimension, Dimension::new(1, 0, 0));

        let mm = UnitRegistry::resolve("mm").expect("mm should exist");
        assert_eq!(mm.scale_to_si, 0.001);
        assert_eq!(mm.dimension, Dimension::new(1, 0, 0));

        let mpa = UnitRegistry::resolve("MPa").expect("MPa should exist");
        assert_eq!(mpa.scale_to_si, 1_000_000.0);
        assert_eq!(mpa.dimension, Dimension::new(-1, 1, -2));

        assert!(UnitRegistry::resolve("cm").is_none());
    }

    #[test]
    fn converts_values_using_si_scale() {
        let kn = UnitRegistry::resolve_expr(&UnitExpr::Unit("kN".to_string()))
            .expect("kN should resolve");
        let n = UnitRegistry::resolve_expr(&UnitExpr::Unit("N".to_string())).expect("N should resolve");

        let force = Quantity::from_unit(12.0, kn);
        approx_equal(force.value_si, 12_000.0);

        let value_in_n = force.convert_to(n).expect("conversion should succeed");
        approx_equal(value_in_n, 12_000.0);

        let value_in_kn = force.convert_to(kn).expect("conversion should succeed");
        approx_equal(value_in_kn, 12.0);
    }

    #[test]
    fn converts_between_millimeters_and_meters() {
        let mm = UnitRegistry::resolve_expr(&UnitExpr::Unit("mm".to_string()))
            .expect("mm should resolve");
        let m = UnitRegistry::resolve_expr(&UnitExpr::Unit("m".to_string())).expect("m should resolve");

        let length = Quantity::from_unit(2500.0, mm);
        approx_equal(length.value_si, 2.5);
        let value_in_m = length.convert_to(m).expect("conversion should succeed");
        approx_equal(value_in_m, 2.5);
    }

    #[test]
    fn resolves_composed_unit_expressions() {
        let mm_sq = UnitRegistry::resolve_expr(&UnitExpr::Power {
            base: "mm".to_string(),
            exponent: 2,
        })
        .expect("mm^2 should resolve");
        approx_equal(mm_sq.scale_to_si, 1e-6);
        assert_eq!(mm_sq.dimension, Dimension::new(2, 0, 0));

        let density_unit = UnitRegistry::resolve_expr(&UnitExpr::Divide(
            Box::new(UnitExpr::Unit("kg".to_string())),
            Box::new(UnitExpr::Power {
                base: "m".to_string(),
                exponent: 3,
            }),
        ))
        .expect("kg/m^3 should resolve");
        approx_equal(density_unit.scale_to_si, 1.0);
        assert_eq!(density_unit.dimension, Dimension::new(-3, 1, 0));

        let moment_unit = UnitRegistry::resolve_expr(&UnitExpr::Multiply(
            Box::new(UnitExpr::Unit("kN".to_string())),
            Box::new(UnitExpr::Unit("m".to_string())),
        ))
        .expect("kN*m should resolve");
        approx_equal(moment_unit.scale_to_si, 1000.0);
        assert_eq!(moment_unit.dimension, Dimension::new(2, 1, -2));
    }

    #[test]
    fn composes_dimensions_via_quantity_multiply_and_divide() {
        let m = UnitRegistry::resolve_expr(&UnitExpr::Unit("m".to_string())).expect("m should resolve");
        let s = UnitRegistry::resolve_expr(&UnitExpr::Unit("s".to_string())).expect("s should resolve");

        let length = Quantity::from_unit(2.0, m);
        let time = Quantity::from_unit(4.0, s);

        let speed = length.divide(&time);
        approx_equal(speed.value_si, 0.5);
        assert_eq!(speed.dimension, Dimension::new(1, 0, -1));

        let distance = speed.multiply(&time);
        approx_equal(distance.value_si, 2.0);
        assert_eq!(distance.dimension, Dimension::new(1, 0, 0));
    }

    #[test]
    fn enforces_conversion_and_arithmetic_compatibility() {
        let n = UnitRegistry::resolve_expr(&UnitExpr::Unit("N".to_string())).expect("N should resolve");
        let kn =
            UnitRegistry::resolve_expr(&UnitExpr::Unit("kN".to_string())).expect("kN should resolve");
        let pa =
            UnitRegistry::resolve_expr(&UnitExpr::Unit("Pa".to_string())).expect("Pa should resolve");

        let force_a = Quantity::from_unit(500.0, n);
        let force_b = Quantity::from_unit(0.5, kn);
        assert!(force_a.is_compatible_with(&force_b));

        let sum = force_a.checked_add(&force_b).expect("add should succeed");
        approx_equal(sum.value_si, 1000.0);

        let diff = sum.checked_sub(&force_a).expect("sub should succeed");
        approx_equal(diff.value_si, 500.0);

        let convert_error = force_a
            .convert_to(pa)
            .expect_err("force to pressure conversion should fail");
        assert!(convert_error.message.contains("Cannot convert incompatible quantities"));
        assert!(convert_error.message.contains("Source quantity dimension"));

        let add_error = force_a
            .checked_add(&Quantity::from_unit(1.0, pa))
            .expect_err("add with incompatible dimensions should fail");
        assert!(add_error.message.contains("Cannot add incompatible quantities"));
        assert!(add_error.message.contains("Left operand dimension"));

        let sub_error = force_a
            .checked_sub(&Quantity::from_unit(1.0, pa))
            .expect_err("sub with incompatible dimensions should fail");
        assert!(sub_error.message.contains("Cannot subtract incompatible quantities"));
    }

    #[test]
    fn reports_unknown_unit_with_supported_list() {
        let error = UnitRegistry::resolve_expr(&UnitExpr::Unit("cm".to_string()))
            .expect_err("unknown unit should fail");
        assert!(error.message.contains("Unknown unit 'cm'"));
        assert!(error.message.contains("Supported units"));
    }
}
