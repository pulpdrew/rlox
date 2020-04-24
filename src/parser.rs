use crate::ast::{Expression, Statement};
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
}

#[allow(dead_code)]
impl Parser {
    pub fn new(source: String) -> Self {
        let mut scanner = Scanner::new(source);
        let current = scanner.next();
        let next = scanner.next();
        Parser {
            scanner,
            current,
            next,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn parse_program(&mut self) -> Statement {
        let program = self.statement();

        match self.eat(Kind::Eof, "Expected end of file.") {
            Ok(_) => {}
            Err(_) => self.synchronize(),
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
            Err(_) => Statement::None,
        }
    }

    fn expression(&mut self) -> Expression {
        self.addition()
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
        if self.current.kind == Kind::Minus {
            let operator = self.advance();
            let expression = self.expression();
            Expression::Unary {
                operator: operator,
                expression: Box::new(expression),
            }
        } else {
            self.number()
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
                self.error_at_current(format!("Unknown character {}", self.next.string).as_str());
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

    fn error_at_current(&mut self, message: &str) {
        let line = self.next.line;
        self.error_at(line, message);
    }

    fn error(&mut self, message: &str) {
        let line = self.current.line;
        self.error_at(line, message);
    }

    fn error_at(&mut self, line: usize, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;

        eprintln!("[line {}] Error: {}", line, message);
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
    }
}
