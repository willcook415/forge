//! Error types for Forge diagnostics.

use std::fmt;

/// Convenience result type used across Forge modules.
pub type ForgeResult<T> = Result<T, ForgeError>;

/// A readable error with optional source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgeError {
    /// Human-readable message.
    pub message: String,
    /// 1-based line number when available.
    pub line: Option<usize>,
    /// 1-based column number when available.
    pub column: Option<usize>,
}

impl ForgeError {
    /// Creates a new error without location information.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
        }
    }

    /// Creates a new error with line and column information.
    pub fn with_span(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            message: message.into(),
            line: Some(line),
            column: Some(column),
        }
    }
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                if let Some((headline, detail)) = self.message.split_once('\n') {
                    write!(f, "{headline} at line {line} column {column}\n{detail}")
                } else {
                    write!(f, "{} at line {line} column {column}", self.message)
                }
            }
            _ => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ForgeError {}
