use crate::ast::{AstNode, Expression, Statement};
use crate::error::ErrorHandler;
use crate::object::Obj;
use crate::scanner::Scanner;
use crate::token::{Kind, Span, Token};
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

    pub fn parse_program(&mut self) -> Vec<AstNode> {
        let mut program = vec![];
        while self.current.kind != Kind::Eof {
            program.push(self.declaration());
        }

        program
    }

    fn declaration(&mut self) -> AstNode {
        match self.current.kind {
            Kind::Var => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> AstNode {
        let keyword = self.advance();

        let name = match self.eat(Kind::IdentifierLiteral, "Expected identifier after 'var'.") {
            Ok(t) => t,
            Err(()) => return AstNode::none(),
        };

        let node = if self.current.kind == Kind::Equal {
            let operator = self.advance();
            let initializer = self.expression();
            let span = Span::merge(vec![
                &keyword.span,
                &name.span,
                &operator.span,
                &initializer.span,
            ]);
            AstNode::from_statement(
                Statement::Declaration {
                    name,
                    operator: Some(operator),
                    initializer: Some(initializer),
                },
                span,
            )
        } else {
            let span = Span::merge(vec![&keyword.span, &name.span]);
            AstNode::from_statement(
                Statement::Declaration {
                    name,
                    operator: None,
                    initializer: None,
                },
                span,
            )
        };

        match self.eat(Kind::Semicolon, "Expected ';' after initializer.") {
            Ok(semi) => {
                let span = Span::merge(vec![&node.span, &semi.span]);
                AstNode::from_ast_node(node, span)
            }
            Err(_) => {
                self.synchronize();
                AstNode::none()
            }
        }
    }

    fn statement(&mut self) -> AstNode {
        let statement = match self.current.kind {
            Kind::Print => self.print_statement(),
            _ => {
                let expression = self.expression();
                match self.eat(Kind::Semicolon, "Expected ';' after expression") {
                    Ok(semi) => {
                        let new_span = Span::merge(vec![&expression.span, &semi.span]);
                        AstNode::from_statement(
                            Statement::Expression {
                                expression: Box::new(expression),
                            },
                            new_span,
                        )
                    }
                    Err(_) => {
                        self.synchronize();
                        AstNode::none()
                    }
                }
            }
        };

        statement
    }

    fn print_statement(&mut self) -> AstNode {
        let keyword = self.advance();
        let expression = self.expression();
        match self.eat(Kind::Semicolon, "Expected ';' after print expression") {
            Ok(semi) => {
                let new_span = Span::merge(vec![&keyword.span, &expression.span, &semi.span]);
                AstNode::from_statement(
                    Statement::Print {
                        keyword,
                        expression: Box::new(expression),
                    },
                    new_span,
                )
            }
            Err(_) => {
                self.synchronize();
                AstNode::none()
            }
        }
    }

    fn expression(&mut self) -> AstNode {
        self.assignment()
    }

    fn assignment(&mut self) -> AstNode {
        let node = self.logic_or();

        if self.current.kind == Kind::Equal {
            let operator = self.advance();
            let rvalue = self.assignment();
            let new_span = Span::merge(vec![&node.span, &operator.span, &rvalue.span]);

            AstNode::from_expression(
                Expression::Assignment {
                    lvalue: Box::new(node),
                    operator,
                    rvalue: Box::new(rvalue),
                },
                new_span,
            )
        } else {
            node
        }
    }

    fn logic_or(&mut self) -> AstNode {
        self.logic_and()
    }

    fn logic_and(&mut self) -> AstNode {
        self.equality()
    }

    fn equality(&mut self) -> AstNode {
        let mut node = self.comparison();
        loop {
            match self.current.kind {
                Kind::EqualEqual | Kind::BangEqual => {
                    let operator = self.advance();
                    let right = self.comparison();
                    let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

                    node = AstNode::from_expression(
                        Expression::Binary {
                            left: Box::new(node),
                            operator,
                            right: Box::new(right),
                        },
                        new_span,
                    );
                }
                _ => break,
            }
        }
        node
    }

    fn comparison(&mut self) -> AstNode {
        let mut node = self.addition();
        loop {
            match self.current.kind {
                Kind::Less | Kind::LessEqual | Kind::Greater | Kind::GreaterEqual => {
                    let operator = self.advance();
                    let right = self.addition();
                    let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

                    node = AstNode::from_expression(
                        Expression::Binary {
                            left: Box::new(node),
                            operator,
                            right: Box::new(right),
                        },
                        new_span,
                    );
                }
                _ => break,
            }
        }
        node
    }

    fn addition(&mut self) -> AstNode {
        let mut node = self.multiplication();

        while self.current.kind == Kind::Plus || self.current.kind == Kind::Minus {
            let operator = self.advance();
            let right = self.multiplication();
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = AstNode::from_expression(
                Expression::Binary {
                    left: Box::new(node),
                    operator,
                    right: Box::new(right),
                },
                new_span,
            );
        }

        node
    }

    fn multiplication(&mut self) -> AstNode {
        let mut node = self.unary();

        while self.current.kind == Kind::Star || self.current.kind == Kind::Slash {
            let operator = self.advance();
            let right = self.unary();
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = AstNode::from_expression(
                Expression::Binary {
                    left: Box::new(node),
                    operator,
                    right: Box::new(right),
                },
                new_span,
            );
        }

        node
    }

    fn unary(&mut self) -> AstNode {
        match self.current.kind {
            Kind::Minus | Kind::Bang => {
                let operator = self.advance();
                let expression = self.unary();
                let new_span = Span::new(expression.span.start - 1, expression.span.end);

                AstNode::from_expression(
                    Expression::Unary {
                        operator: operator,
                        expression: Box::new(expression),
                    },
                    new_span,
                )
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> AstNode {
        self.primary()
    }

    fn primary(&mut self) -> AstNode {
        match self.current.kind {
            Kind::LeftParen => {
                self.advance();
                let expression = self.expression();
                match self.eat(Kind::RightParen, "Expected ')' after expression.") {
                    Ok(_) => {
                        let new_span =
                            Span::new(expression.span.start - 1, expression.span.end + 1);
                        AstNode::from_ast_node(expression, new_span)
                    }
                    Err(()) => AstNode::none(),
                }
            }
            Kind::IdentifierLiteral => {
                let literal = self.advance();
                let span = literal.span;
                AstNode::from_expression(Expression::Variable { name: literal }, span)
            }
            Kind::NumberLiteral => self.number(),
            Kind::StringLiteral => self.string(),
            Kind::True => {
                let literal = self.advance();
                let span = literal.span;
                AstNode::from_expression(
                    Expression::Constant {
                        value: Value::Bool(true),
                        literal,
                    },
                    span,
                )
            }
            Kind::False => {
                let literal = self.advance();
                let span = literal.span;
                AstNode::from_expression(
                    Expression::Constant {
                        value: Value::Bool(false),
                        literal,
                    },
                    span,
                )
            }
            Kind::Nil => {
                let literal = self.advance();
                let span = literal.span;
                AstNode::from_expression(
                    Expression::Constant {
                        value: Value::Nil,
                        literal,
                    },
                    span,
                )
            }
            _ => {
                self.error_at_current("Expected primary expression.");
                AstNode::none()
            }
        }
    }

    fn number(&mut self) -> AstNode {
        let literal = self.advance();
        let span = literal.span;

        let value = Value::Number(
            literal
                .string
                .parse()
                .expect(format!("Failed to parse '{}' as f64", literal.string).as_str()),
        );

        AstNode::from_expression(Expression::Constant { literal, value }, span)
    }

    fn string(&mut self) -> AstNode {
        let literal = self.advance();
        let span = literal.span;
        let value = Value::Obj(Obj::from(&literal.string[1..literal.string.len() - 1]));

        AstNode::from_expression(Expression::Constant { literal, value }, span)
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
        let span = self.next.span;
        self.error_at(&span, message);
    }

    fn error_at_current(&mut self, message: &str) {
        let span = self.current.span;
        self.error_at(&span, message);
    }

    fn error_at(&mut self, span: &Span, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;

        self.handler.error(span, message);
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
