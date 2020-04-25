use crate::ast::{Expression, Statement};
use crate::error::ErrorHandler;
use crate::scanner::Scanner;
use crate::token::{Kind, Token};
use crate::value::Value;

#[derive(Debug)]
pub struct Parser {
    scanner: Scanner,
    current: Token,
    next: Token,
    pub had_error: bool,
    pub panic_mode: bool,
    handler: ErrorHandler,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(source: String, handler: ErrorHandler) -> Self {
        let mut scanner = Scanner::new(source);
        let current = scanner.next();
        let next = scanner.next();
        Parser {
            scanner,
            current,
            next,
            had_error: false,
            panic_mode: false,
            handler,
        }
    }

    pub fn parse_program(&mut self) -> Vec<Statement> {
        let mut program = vec![];
        while self.current.kind != Kind::Eof {
            program.push(self.statement());
        }

        program
    }

    fn statement(&mut self) -> Statement {
        let expression = self.expression();

        match self.eat(Kind::Semicolon, "Expected ';' after expression") {
            Ok(semi) => Statement::Expression {
                expression: Box::new(expression),
                semi,
            },
            Err(_) => {
                self.synchronize();
                Statement::None
            }
        }
    }

    fn expression(&mut self) -> Expression {
        self.equality()
    }

    fn equality(&mut self) -> Expression {
        let mut expr = self.comparison();
        loop {
            match self.current.kind {
                Kind::EqualEqual | Kind::BangEqual => {
                    let operator = self.advance();
                    let right = self.comparison();
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }
        expr
    }

    fn comparison(&mut self) -> Expression {
        let mut expr = self.addition();
        loop {
            match self.current.kind {
                Kind::Less | Kind::LessEqual | Kind::Greater | Kind::GreaterEqual => {
                    let operator = self.advance();
                    let right = self.addition();
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }
        expr
    }

    fn addition(&mut self) -> Expression {
        let mut expr = self.multiplication();

        while self.current.kind == Kind::Plus || self.current.kind == Kind::Minus {
            let operator = self.advance();
            let right = self.multiplication();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn multiplication(&mut self) -> Expression {
        let mut expr = self.unary();

        while self.current.kind == Kind::Star || self.current.kind == Kind::Slash {
            let operator = self.advance();
            let right = self.unary();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn unary(&mut self) -> Expression {
        match self.current.kind {
            Kind::Minus | Kind::Bang => {
                let operator = self.advance();
                let expression = self.unary();
                Expression::Unary {
                    operator: operator,
                    expression: Box::new(expression),
                }
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> Expression {
        self.primary()
    }

    fn primary(&mut self) -> Expression {
        match self.current.kind {
            Kind::LeftParen => {
                self.advance();
                let expression = self.expression();
                match self.eat(Kind::RightParen, "Expected ')' after expression.") {
                    Ok(_) => expression,
                    Err(()) => Expression::None,
                }
            }
            Kind::True => Expression::True {
                literal: self.advance(),
            },
            Kind::False => Expression::False {
                literal: self.advance(),
            },
            Kind::NumberLiteral => self.number(),
            _ => {
                self.error_at_current("Expected primary expression.");
                Expression::None
            }
        }
    }

    fn number(&mut self) -> Expression {
        let literal = self.advance();

        let value = Value::Number(
            literal
                .string
                .parse()
                .expect(format!("Failed to parse '{}' as f64", literal.string).as_str()),
        );
        Expression::Constant { value, literal }
    }

    fn advance(&mut self) -> Token {
        let previous = self.current.clone();
        self.current = self.next.clone();
        loop {
            self.next = self.scanner.next();
            if let Kind::Error = self.next.kind {
                self.error_at_next(format!("Unknown character {}", self.next.string).as_str());
            } else {
                break;
            }
        }
        previous
    }
    fn eat(&mut self, kind: Kind, message: &str) -> Result<Token, ()> {
        if self.current.kind == kind {
            Ok(self.advance())
        } else {
            self.error_at_current(message);
            Err(())
        }
    }

    fn error_at_next(&mut self, message: &str) {
        let index_in_source = self.next.index_in_source;
        self.error_at(index_in_source, message);
    }

    fn error_at_current(&mut self, message: &str) {
        let index_in_source = self.current.index_in_source;
        self.error_at(index_in_source, message);
    }

    fn error_at(&mut self, index_in_source: usize, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;

        self.handler.error(index_in_source, message);
    }

    fn synchronize(&mut self) {
        loop {
            match self.current.kind {
                Kind::Semicolon | Kind::Eof => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.panic_mode = false;
    }
}
