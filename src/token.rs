//! Token definitions for Forge v0.1.

/// Token categories supported by Forge v0.1.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// Identifier such as variable names and unit names.
    Identifier(String),
    /// Numeric literal text.
    Number(String),
    /// `=`
    Equal,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `^`
    Caret,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `print`
    Print,
    /// `as`
    As,
    /// End of input marker.
    Eof,
}

/// A token with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token kind.
    pub kind: TokenKind,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
}

impl Token {
    /// Creates a token at the given 1-based source location.
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}
