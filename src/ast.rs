//! Abstract syntax tree definitions for Forge v0.1.

/// Top-level program node.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Program {
    /// Ordered statements in source form.
    pub statements: Vec<Statement>,
}

/// Supported statement kinds in v0.1.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// `name = expression`
    Assignment {
        name: String,
        value: Expr,
        line: usize,
        column: usize,
    },
    /// `print expression` or `print expression as UNIT`
    Print {
        value: Expr,
        as_unit: Option<UnitExpr>,
        line: usize,
        column: usize,
    },
}

/// Supported expression kinds in v0.1.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal (e.g. `12`, `12.5`, `80e3`).
    Number(f64),
    /// Quantity literal (e.g. `12 kN`, `300 mm^2`).
    Quantity { value: f64, unit: UnitExpr },
    /// Variable reference.
    Variable(String),
    /// Parenthesized expression.
    Group(Box<Expr>),
    /// Unary operation.
    Unary { op: UnaryOp, expression: Box<Expr> },
    /// Binary operation.
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

/// Unary operators in v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Arithmetic negation (`-x`).
    Negate,
}

/// Binary operators in v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// Addition.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division.
    Divide,
    /// Exponentiation.
    Power,
}

/// Unit expression tree used for quantities and print conversions.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitExpr {
    /// Base unit symbol (e.g. `mm`, `kg`, `kN`).
    Unit(String),
    /// Product of unit expressions (`kN*m`).
    Multiply(Box<UnitExpr>, Box<UnitExpr>),
    /// Division of unit expressions (`kg/m^3`).
    Divide(Box<UnitExpr>, Box<UnitExpr>),
    /// Integer unit power (`mm^2`).
    Power { base: String, exponent: u32 },
}
