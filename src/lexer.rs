//! Lexer for turning source text into Forge tokens.

use crate::error::{ForgeError, ForgeResult};
use crate::token::{Token, TokenKind};

/// Stateful lexer for Forge v0.1.
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a lexer for a source string.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenizes source text into a token stream.
    pub fn tokenize(&mut self) -> ForgeResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.consume_whitespace();
                continue;
            }

            if ch == '#' {
                self.consume_comment();
                continue;
            }

            let token_line = self.line;
            let token_column = self.column;
            let kind = match ch {
                '=' => {
                    self.advance_char();
                    TokenKind::Equal
                }
                '+' => {
                    self.advance_char();
                    TokenKind::Plus
                }
                '-' => {
                    self.advance_char();
                    TokenKind::Minus
                }
                '*' => {
                    self.advance_char();
                    TokenKind::Star
                }
                '/' => {
                    self.advance_char();
                    TokenKind::Slash
                }
                '^' => {
                    self.advance_char();
                    TokenKind::Caret
                }
                '(' => {
                    self.advance_char();
                    TokenKind::LeftParen
                }
                ')' => {
                    self.advance_char();
                    TokenKind::RightParen
                }
                c if is_identifier_start(c) => self.lex_identifier_or_keyword(),
                c if c.is_ascii_digit() => self.lex_number()?,
                _ => {
                    return Err(ForgeError::with_span(
                        format!(
                            "Invalid character '{}'. Expected a number, identifier, operator, parenthesis, comment, or whitespace.",
                            ch
                        ),
                        token_line,
                        token_column,
                    ));
                }
            };

            tokens.push(Token::new(kind, token_line, token_column));
        }

        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }

    fn lex_identifier_or_keyword(&mut self) -> TokenKind {
        let mut lexeme = String::new();
        while let Some(ch) = self.peek_char() {
            if is_identifier_continue(ch) {
                lexeme.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }

        match lexeme.as_str() {
            "print" => TokenKind::Print,
            "as" => TokenKind::As,
            _ => TokenKind::Identifier(lexeme),
        }
    }

    fn lex_number(&mut self) -> ForgeResult<TokenKind> {
        let mut literal = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                literal.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }

        if self.peek_char() == Some('.') {
            literal.push('.');
            self.advance_char();

            let fraction_start = self.position;
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    literal.push(ch);
                    self.advance_char();
                } else {
                    break;
                }
            }

            if self.position == fraction_start {
                return Err(ForgeError::with_span(
                    "Invalid numeric literal: missing digits after decimal point",
                    self.line,
                    self.column,
                ));
            }
        }

        if matches!(self.peek_char(), Some('e' | 'E')) {
            literal.push(self.peek_char().expect("peeked Some above"));
            self.advance_char();

            if matches!(self.peek_char(), Some('+' | '-')) {
                literal.push(self.peek_char().expect("peeked Some above"));
                self.advance_char();
            }

            let exponent_start = self.position;
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    literal.push(ch);
                    self.advance_char();
                } else {
                    break;
                }
            }

            if self.position == exponent_start {
                return Err(ForgeError::with_span(
                    "Invalid numeric literal: missing exponent digits",
                    self.line,
                    self.column,
                ));
            }
        }

        Ok(TokenKind::Number(literal))
    }

    fn consume_comment(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.advance_char();
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.advance_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.position += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(ch)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::token::{Token, TokenKind};

    fn kinds(tokens: &[Token]) -> Vec<TokenKind> {
        tokens.iter().map(|token| token.kind.clone()).collect()
    }

    #[test]
    fn tokenizes_simple_assignments() {
        let source = "force = 12 kN\narea = 300 mm^2";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier("force".to_string()),
                TokenKind::Equal,
                TokenKind::Number("12".to_string()),
                TokenKind::Identifier("kN".to_string()),
                TokenKind::Identifier("area".to_string()),
                TokenKind::Equal,
                TokenKind::Number("300".to_string()),
                TokenKind::Identifier("mm".to_string()),
                TokenKind::Caret,
                TokenKind::Number("2".to_string()),
                TokenKind::Eof,
            ]
        );

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[4].line, 2);
        assert_eq!(tokens[4].column, 1);
    }

    #[test]
    fn tokenizes_print_statements() {
        let source = "print stress\nprint stress as MPa";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Print,
                TokenKind::Identifier("stress".to_string()),
                TokenKind::Print,
                TokenKind::Identifier("stress".to_string()),
                TokenKind::As,
                TokenKind::Identifier("MPa".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_quantity_expression_with_scientific_notation() {
        let source = "value = (12.5e3 + 2) * mm^2";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier("value".to_string()),
                TokenKind::Equal,
                TokenKind::LeftParen,
                TokenKind::Number("12.5e3".to_string()),
                TokenKind::Plus,
                TokenKind::Number("2".to_string()),
                TokenKind::RightParen,
                TokenKind::Star,
                TokenKind::Identifier("mm".to_string()),
                TokenKind::Caret,
                TokenKind::Number("2".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_identifiers_numbers_and_operators() {
        let source = "alpha1 beta2 12 12.5 80e3 + - * / ^ ( )";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier("alpha1".to_string()),
                TokenKind::Identifier("beta2".to_string()),
                TokenKind::Number("12".to_string()),
                TokenKind::Number("12.5".to_string()),
                TokenKind::Number("80e3".to_string()),
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Caret,
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn ignores_comments() {
        let source = "# header\nforce = 12 kN # inline comment\n# trailing";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier("force".to_string()),
                TokenKind::Equal,
                TokenKind::Number("12".to_string()),
                TokenKind::Identifier("kN".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn ignores_full_line_and_inline_comments() {
        let source = "a = 1 # first\n# second\nb = 2";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier("a".to_string()),
                TokenKind::Equal,
                TokenKind::Number("1".to_string()),
                TokenKind::Identifier("b".to_string()),
                TokenKind::Equal,
                TokenKind::Number("2".to_string()),
                TokenKind::Eof,
            ]
        );
        assert_eq!(tokens[3].line, 3);
        assert_eq!(tokens[3].column, 1);
    }

    #[test]
    fn reports_invalid_characters_with_span() {
        let source = "force = 12 @";
        let mut lexer = Lexer::new(source);
        let error = lexer.tokenize().expect_err("lexer should fail");

        assert!(error.message.contains("Invalid character '@'"));
        assert_eq!(error.line, Some(1));
        assert_eq!(error.column, Some(12));
    }
}
