//! Parser for turning Forge tokens into an AST.

use crate::ast::{BinaryOp, Expr, Program, Statement, UnaryOp, UnitExpr};
use crate::error::{ForgeError, ForgeResult};
use crate::token::{Token, TokenKind};

/// Recursive-descent parser for Forge v0.1.
#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Creates a parser from a token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Parses tokens into a program AST.
    pub fn parse(&mut self) -> ForgeResult<Program> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> ForgeResult<Statement> {
        let line = self.current().line;
        let column = self.current().column;
        match self.current_kind() {
            TokenKind::Print => self.parse_print_statement(line, column),
            TokenKind::Identifier(_) => self.parse_assignment_statement(line, column),
            _ => Err(self.error_at_current("expected a statement start (`print` or identifier)")),
        }
    }

    fn parse_assignment_statement(&mut self, line: usize, column: usize) -> ForgeResult<Statement> {
        let name = self.expect_identifier("expected variable name at start of assignment")?;
        self.expect_simple(
            |kind| matches!(kind, TokenKind::Equal),
            "expected '=' after variable name in assignment",
        )?;
        let value = self.parse_expression()?;
        Ok(Statement::Assignment {
            name,
            value,
            line,
            column,
        })
    }

    fn parse_print_statement(&mut self, line: usize, column: usize) -> ForgeResult<Statement> {
        self.expect_simple(
            |kind| matches!(kind, TokenKind::Print),
            "expected 'print' keyword",
        )?;
        let value = self.parse_expression()?;
        let as_unit = if self.match_simple(|kind| matches!(kind, TokenKind::As)) {
            Some(self.parse_unit_expression()?)
        } else {
            None
        };

        Ok(Statement::Print {
            value,
            as_unit,
            line,
            column,
        })
    }

    fn parse_expression(&mut self) -> ForgeResult<Expr> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> ForgeResult<Expr> {
        let mut expression = self.parse_multiplication()?;
        loop {
            let op = if self.match_simple(|kind| matches!(kind, TokenKind::Plus)) {
                Some(BinaryOp::Add)
            } else if self.match_simple(|kind| matches!(kind, TokenKind::Minus)) {
                Some(BinaryOp::Subtract)
            } else {
                None
            };

            let Some(operator) = op else {
                break;
            };

            let right = self.parse_multiplication()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_multiplication(&mut self) -> ForgeResult<Expr> {
        let mut expression = self.parse_power()?;
        loop {
            let op = if self.match_simple(|kind| matches!(kind, TokenKind::Star)) {
                Some(BinaryOp::Multiply)
            } else if self.match_simple(|kind| matches!(kind, TokenKind::Slash)) {
                Some(BinaryOp::Divide)
            } else {
                None
            };

            let Some(operator) = op else {
                break;
            };

            let right = self.parse_power()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_power(&mut self) -> ForgeResult<Expr> {
        let left = self.parse_unary()?;
        if self.match_simple(|kind| matches!(kind, TokenKind::Caret)) {
            let right = self.parse_power()?;
            return Ok(Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Power,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ForgeResult<Expr> {
        if self.match_simple(|kind| matches!(kind, TokenKind::Minus)) {
            let expression = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expression: Box::new(expression),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> ForgeResult<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(literal) => {
                let value = literal.parse::<f64>().map_err(|_| {
                    ForgeError::with_span("Invalid numeric literal.", token.line, token.column)
                })?;

                if self.is_unit_expression_start_on_line(token.line) {
                    let unit = self.parse_unit_expression()?;
                    Ok(Expr::Quantity { value, unit })
                } else {
                    Ok(Expr::Number(value))
                }
            }
            TokenKind::Identifier(name) => Ok(Expr::Variable(name)),
            TokenKind::LeftParen => {
                let expression = self.parse_expression()?;
                self.expect_simple(
                    |kind| matches!(kind, TokenKind::RightParen),
                    "expected ')' after expression",
                )?;
                Ok(Expr::Group(Box::new(expression)))
            }
            _ => Err(ForgeError::with_span(
                format!(
                    "Invalid syntax: expected an expression, found {}.",
                    describe_token(&token.kind)
                ),
                token.line,
                token.column,
            )),
        }
    }

    fn parse_unit_expression(&mut self) -> ForgeResult<UnitExpr> {
        let mut expression = self.parse_unit_factor()?;
        loop {
            if self.match_simple(|kind| matches!(kind, TokenKind::Star)) {
                let right = self.parse_unit_factor()?;
                expression = UnitExpr::Multiply(Box::new(expression), Box::new(right));
                continue;
            }

            if self.match_simple(|kind| matches!(kind, TokenKind::Slash)) {
                let right = self.parse_unit_factor()?;
                expression = UnitExpr::Divide(Box::new(expression), Box::new(right));
                continue;
            }

            break;
        }

        Ok(expression)
    }

    fn parse_unit_factor(&mut self) -> ForgeResult<UnitExpr> {
        let base = self.expect_identifier("expected unit name in unit expression")?;
        if self.match_simple(|kind| matches!(kind, TokenKind::Caret)) {
            let exponent = self.expect_unit_exponent()?;
            Ok(UnitExpr::Power { base, exponent })
        } else {
            Ok(UnitExpr::Unit(base))
        }
    }

    fn expect_unit_exponent(&mut self) -> ForgeResult<u32> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(literal) => {
                if literal.chars().all(|ch| ch.is_ascii_digit()) {
                    literal.parse::<u32>().map_err(|_| {
                        ForgeError::with_span(
                            "Invalid unit exponent: value is out of supported range.",
                            token.line,
                            token.column,
                        )
                    })
                } else {
                    Err(ForgeError::with_span(
                        "Invalid syntax: expected an integer exponent after '^' in a unit expression.",
                        token.line,
                        token.column,
                    ))
                }
            }
            _ => Err(ForgeError::with_span(
                "Invalid syntax: expected an integer exponent after '^' in a unit expression.",
                token.line,
                token.column,
            )),
        }
    }

    fn is_unit_expression_start_on_line(&self, line: usize) -> bool {
        self.current().line == line && matches!(self.current_kind(), TokenKind::Identifier(_))
    }

    fn expect_identifier(&mut self, message: &str) -> ForgeResult<String> {
        let token = self.advance().clone();
        if let TokenKind::Identifier(name) = token.kind {
            Ok(name)
        } else {
            Err(ForgeError::with_span(
                format!(
                    "Invalid syntax: {message}. Found {}.",
                    describe_token(&token.kind)
                ),
                token.line,
                token.column,
            ))
        }
    }

    fn expect_simple<F>(&mut self, predicate: F, message: &str) -> ForgeResult<()>
    where
        F: FnOnce(&TokenKind) -> bool,
    {
        if predicate(self.current_kind()) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_at_current(message))
        }
    }

    fn match_simple<F>(&mut self, predicate: F) -> bool
    where
        F: FnOnce(&TokenKind) -> bool,
    {
        if predicate(self.current_kind()) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn current(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn current_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn advance(&mut self) -> &Token {
        let index = self.position;
        if !self.is_at_end() {
            self.position += 1;
        }
        &self.tokens[index]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    fn error_at_current(&self, message: &str) -> ForgeError {
        let token = self.current();
        ForgeError::with_span(
            format!(
                "Invalid syntax: {message}. Found {}.",
                describe_token(&token.kind)
            ),
            token.line,
            token.column,
        )
    }
}

fn describe_token(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier(name) => format!("identifier '{name}'"),
        TokenKind::Number(value) => format!("number '{value}'"),
        TokenKind::Equal => "'='".to_string(),
        TokenKind::Plus => "'+'".to_string(),
        TokenKind::Minus => "'-'".to_string(),
        TokenKind::Star => "'*'".to_string(),
        TokenKind::Slash => "'/'".to_string(),
        TokenKind::Caret => "'^'".to_string(),
        TokenKind::LeftParen => "'('".to_string(),
        TokenKind::RightParen => "')'".to_string(),
        TokenKind::Print => "'print'".to_string(),
        TokenKind::As => "'as'".to_string(),
        TokenKind::Eof => "end of input".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{BinaryOp, Expr, Statement, UnaryOp, UnitExpr};
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse_ok(source: &str) -> Vec<Statement> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");
        let mut parser = Parser::new(tokens);
        parser.parse().expect("parser should succeed").statements
    }

    fn parse_err(source: &str) -> String {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexer should succeed");
        let mut parser = Parser::new(tokens);
        parser.parse().expect_err("parser should fail").to_string()
    }

    #[test]
    fn parses_assignment_and_print_statements() {
        let statements = parse_ok("force = 12 kN\nprint force as MPa");
        assert_eq!(statements.len(), 2);

        assert_eq!(
            statements[0],
            Statement::Assignment {
                name: "force".to_string(),
                value: Expr::Quantity {
                    value: 12.0,
                    unit: UnitExpr::Unit("kN".to_string()),
                },
                line: 1,
                column: 1,
            }
        );

        assert_eq!(
            statements[1],
            Statement::Print {
                value: Expr::Variable("force".to_string()),
                as_unit: Some(UnitExpr::Unit("MPa".to_string())),
                line: 2,
                column: 1,
            }
        );
    }

    #[test]
    fn parses_operator_precedence() {
        let statements = parse_ok("result = -2^3*4+5");
        assert_eq!(statements.len(), 1);

        assert_eq!(
            statements[0],
            Statement::Assignment {
                name: "result".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Binary {
                            left: Box::new(Expr::Unary {
                                op: UnaryOp::Negate,
                                expression: Box::new(Expr::Number(2.0)),
                            }),
                            op: BinaryOp::Power,
                            right: Box::new(Expr::Number(3.0)),
                        }),
                        op: BinaryOp::Multiply,
                        right: Box::new(Expr::Number(4.0)),
                    }),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Number(5.0)),
                },
                line: 1,
                column: 1,
            }
        );
    }

    #[test]
    fn parses_parentheses_and_unit_expressions() {
        let statements = parse_ok("density = (force) / 2 kg/m^3\nprint density as kN*m");
        assert_eq!(statements.len(), 2);

        assert_eq!(
            statements[0],
            Statement::Assignment {
                name: "density".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Group(Box::new(Expr::Variable("force".to_string())))),
                    op: BinaryOp::Divide,
                    right: Box::new(Expr::Quantity {
                        value: 2.0,
                        unit: UnitExpr::Divide(
                            Box::new(UnitExpr::Unit("kg".to_string())),
                            Box::new(UnitExpr::Power {
                                base: "m".to_string(),
                                exponent: 3,
                            }),
                        ),
                    }),
                },
                line: 1,
                column: 1,
            }
        );

        assert_eq!(
            statements[1],
            Statement::Print {
                value: Expr::Variable("density".to_string()),
                as_unit: Some(UnitExpr::Multiply(
                    Box::new(UnitExpr::Unit("kN".to_string())),
                    Box::new(UnitExpr::Unit("m".to_string())),
                )),
                line: 2,
                column: 1,
            }
        );
    }

    #[test]
    fn parses_print_statement_without_conversion() {
        let statements = parse_ok("x = 1\nprint x");
        assert_eq!(statements.len(), 2);

        assert_eq!(
            statements[1],
            Statement::Print {
                value: Expr::Variable("x".to_string()),
                as_unit: None,
                line: 2,
                column: 1,
            }
        );
    }

    #[test]
    fn parses_assignments_with_multiple_unit_expression_forms() {
        let statements = parse_ok("a = 1 mm^2\nb = 1 kg/m^3\nc = 1 kN*m");
        assert_eq!(statements.len(), 3);

        assert_eq!(
            statements[0],
            Statement::Assignment {
                name: "a".to_string(),
                value: Expr::Quantity {
                    value: 1.0,
                    unit: UnitExpr::Power {
                        base: "mm".to_string(),
                        exponent: 2,
                    },
                },
                line: 1,
                column: 1,
            }
        );

        assert_eq!(
            statements[1],
            Statement::Assignment {
                name: "b".to_string(),
                value: Expr::Quantity {
                    value: 1.0,
                    unit: UnitExpr::Divide(
                        Box::new(UnitExpr::Unit("kg".to_string())),
                        Box::new(UnitExpr::Power {
                            base: "m".to_string(),
                            exponent: 3,
                        }),
                    ),
                },
                line: 2,
                column: 1,
            }
        );

        assert_eq!(
            statements[2],
            Statement::Assignment {
                name: "c".to_string(),
                value: Expr::Quantity {
                    value: 1.0,
                    unit: UnitExpr::Multiply(
                        Box::new(UnitExpr::Unit("kN".to_string())),
                        Box::new(UnitExpr::Unit("m".to_string())),
                    ),
                },
                line: 3,
                column: 1,
            }
        );
    }

    #[test]
    fn parses_sequential_numeric_assignments_across_lines() {
        let statements = parse_ok("a = 10\nb = 3\nprint a + b");
        assert_eq!(statements.len(), 3);

        assert_eq!(
            statements[0],
            Statement::Assignment {
                name: "a".to_string(),
                value: Expr::Number(10.0),
                line: 1,
                column: 1,
            }
        );
        assert_eq!(
            statements[1],
            Statement::Assignment {
                name: "b".to_string(),
                value: Expr::Number(3.0),
                line: 2,
                column: 1,
            }
        );
    }

    #[test]
    fn rejects_invalid_assignment_syntax() {
        let error = parse_err("force 12 kN");
        assert!(
            error.contains("Invalid syntax"),
            "unexpected error: {error}"
        );
        assert!(
            error.contains("expected '=' after variable name in assignment"),
            "unexpected error: {error}"
        );
        assert!(
            error.contains("line 1 column 7"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_missing_unit_after_as() {
        let error = parse_err("print stress as");
        assert!(
            error.contains("expected unit name in unit expression"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_missing_closing_parenthesis() {
        let error = parse_err("x = (1 + 2");
        assert!(
            error.contains("expected ')' after expression"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_non_integer_unit_exponent() {
        let error = parse_err("x = 1 mm^2.5");
        assert!(
            error.contains("expected an integer exponent after '^' in a unit expression"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_unexpected_statement_start() {
        let error = parse_err("+ 1");
        assert!(
            error.contains("Invalid syntax"),
            "unexpected error: {error}"
        );
        assert!(
            error.contains("expected a statement start"),
            "unexpected error: {error}"
        );
        assert!(
            error.contains("line 1 column 1"),
            "unexpected error: {error}"
        );
    }
}
