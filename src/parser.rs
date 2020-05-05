use crate::ast::{AstNode, Expression, Statement};
use crate::error::ReportableError;
use crate::scanner::Scanner;
use crate::token::{Kind, Span, Token};
use crate::value::Value;

#[derive(Debug)]
pub struct ParsingError {
    message: String,
    span: Span,
}

impl ReportableError for ParsingError {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        format!("Parsing Error - {}", self.message)
    }
}

#[derive(Debug)]
pub struct Parser {
    scanner: Scanner,
    current: Token,
    next: Token,
    panic_mode: bool,
    errors: Vec<ParsingError>,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut scanner = Scanner::new(&source);
        let current = scanner.next().unwrap();
        let next = scanner.next().unwrap();
        Parser {
            scanner,
            current,
            next,
            panic_mode: false,
            errors: vec![],
        }
    }

    /// Parse the source into a program - a list of declaratation `AstNode`s
    pub fn parse_program(&mut self) -> Result<Vec<AstNode>, Vec<ParsingError>> {
        let mut program = vec![];
        while self.current.kind != Kind::Eof {
            program.push(self.declaration());
        }

        if self.errors.is_empty() {
            Ok(program)
        } else {
            Err(self.errors.drain(0..).collect())
        }
    }

    fn declaration(&mut self) -> AstNode {
        match self.current.kind {
            Kind::Var => self.var_declaration(),
            Kind::Fun => {
                self.advance();
                self.function()
            }
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
            AstNode::new_statement(
                Statement::Declaration {
                    name,
                    operator: Some(operator),
                    initializer: Some(Box::new(initializer)),
                },
                span,
            )
        } else {
            let span = Span::merge(vec![&keyword.span, &name.span]);
            AstNode::new_statement(
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
                AstNode::new_ast_node(node, span)
            }
            Err(_) => AstNode::none(),
        }
    }

    fn parameter_list(&mut self) -> Result<Vec<Token>, ()> {
        let mut parameters = vec![];
        parameters.push(self.advance());
        while self.current.kind == Kind::Comma {
            self.advance();
            match self.eat(Kind::IdentifierLiteral, "Expected parameter name.") {
                Ok(id) => parameters.push(id),
                Err(_) => return Err(()),
            }
        }

        Ok(parameters)
    }

    fn function(&mut self) -> AstNode {
        let name = match self.eat(Kind::IdentifierLiteral, "Expected function name.") {
            Ok(t) => t,
            Err(()) => return AstNode::none(),
        };

        match self.eat(Kind::LeftParen, "Expected '(' after function declaration") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        let parameters = match self.current.kind {
            Kind::RightParen => vec![],
            Kind::IdentifierLiteral => match self.parameter_list() {
                Ok(list) => list,
                Err(()) => return AstNode::none(),
            },
            _ => {
                self.error_at_current("Expected parameter list or ')'.");
                self.synchronize();
                return AstNode::none();
            }
        };

        match self.eat(Kind::RightParen, "Expected ')' after formal parameter list") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        let body = self.block_statement();
        let span = Span::merge(vec![&name.span, &body.span]);

        AstNode::new_statement(
            Statement::FunDeclaration {
                name,
                parameters,
                body: Box::new(body),
            },
            span,
        )
    }

    fn statement(&mut self) -> AstNode {
        match self.current.kind {
            Kind::Print => self.print_statement(),
            Kind::LeftBrace => self.block_statement(),
            Kind::If => self.if_statement(),
            Kind::While => self.while_statement(),
            Kind::For => self.for_statement(),
            Kind::Return => self.return_statement(),
            _ => self.expression_statement(),
        }
    }

    fn expression_statement(&mut self) -> AstNode {
        let expression = self.expression();
        match self.eat(Kind::Semicolon, "Expected ';' after expression") {
            Ok(semi) => {
                let new_span = Span::merge(vec![&expression.span, &semi.span]);
                AstNode::new_statement(
                    Statement::Expression {
                        expression: Box::new(expression),
                    },
                    new_span,
                )
            }
            Err(_) => AstNode::none(),
        }
    }

    fn return_statement(&mut self) -> AstNode {
        let keyword = self.advance();

        let (value, span) = match self.current.kind {
            Kind::Semicolon => (None, keyword.span),
            _ => {
                let expr = self.expression();
                let span = Span::merge(vec![&keyword.span, &expr.span]);
                (Some(Box::new(expr)), span)
            }
        };

        match self.eat(Kind::Semicolon, "Expected ';' after return statement.") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        AstNode::new_statement(Statement::Return { value }, span)
    }

    fn for_statement(&mut self) -> AstNode {
        let keyword = self.advance();
        match self.eat(Kind::LeftParen, "Expected '(' after 'for.'") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        let initializer = match self.current.kind {
            Kind::Var => Some(Box::new(self.var_declaration())),
            Kind::Semicolon => {
                self.advance();
                None
            }
            _ => Some(Box::new(self.expression_statement())),
        };

        let condition = match self.current.kind {
            Kind::Semicolon => None,
            _ => Some(Box::new(self.expression())),
        };

        match self.eat(Kind::Semicolon, "Expected ';' after for condition.") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        let update = match self.current.kind {
            Kind::RightParen => None,
            _ => Some(Box::new(self.expression())),
        };

        match self.eat(Kind::RightParen, "Expected ')' before for block.") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        let block = self.statement();
        let span = Span::merge(vec![&keyword.span, &block.span]);

        AstNode::new_statement(
            Statement::For {
                initializer,
                condition,
                update,
                block: Box::new(block),
            },
            span,
        )
    }

    fn while_statement(&mut self) -> AstNode {
        let keyword = self.advance();
        match self.eat(Kind::LeftParen, "Expected '(' after 'while.'") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        let condition = self.expression();

        match self.eat(Kind::RightParen, "Expected ')' after while condition.") {
            Ok(_) => {}
            Err(_) => return AstNode::none(),
        }

        let block = self.statement();
        let span = Span::merge(vec![&keyword.span, &block.span]);

        AstNode::new_statement(
            Statement::While {
                condition: Box::new(condition),
                block: Box::new(block),
            },
            span,
        )
    }

    fn if_statement(&mut self) -> AstNode {
        let keyword = self.advance();
        match self.eat(Kind::LeftParen, "Expected '(' after 'if.'") {
            Ok(_) => {}
            Err(_) => {
                return AstNode::none();
            }
        }

        let condition = self.expression();

        match self.eat(Kind::RightParen, "Expected ')' after if condition.") {
            Ok(_) => {}
            Err(_) => {
                return AstNode::none();
            }
        }

        let if_block = self.statement();
        let mut span = Span::merge(vec![&keyword.span, &if_block.span]);

        let else_block = if let Kind::Else = self.current.kind {
            self.advance();
            let stmt = self.statement();
            span = Span::merge(vec![&span, &stmt.span]);
            Some(Box::new(stmt))
        } else {
            None
        };

        AstNode::new_statement(
            Statement::If {
                condition: Box::new(condition),
                if_block: Box::new(if_block),
                else_block,
            },
            span,
        )
    }

    fn block_statement(&mut self) -> AstNode {
        let lbrace = self.advance();

        let mut declarations = vec![];
        loop {
            match self.current.kind {
                Kind::RightBrace | Kind::Eof => break,
                _ => declarations.push(self.declaration()),
            }
        }

        match self.eat(Kind::RightBrace, "Expected '}' after block statement") {
            Ok(rbrace) => {
                let new_span = Span::merge(vec![&lbrace.span, &rbrace.span]);
                AstNode::new_statement(
                    Statement::Block {
                        declarations,
                        rbrace,
                    },
                    new_span,
                )
            }
            Err(_) => AstNode::none(),
        }
    }

    fn print_statement(&mut self) -> AstNode {
        let keyword = self.advance();
        let expression = self.expression();
        match self.eat(Kind::Semicolon, "Expected ';' after print statement") {
            Ok(semi) => {
                let new_span = Span::merge(vec![&keyword.span, &expression.span, &semi.span]);
                AstNode::new_statement(
                    Statement::Print {
                        keyword,
                        expression: Box::new(expression),
                    },
                    new_span,
                )
            }
            Err(_) => AstNode::none(),
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

            AstNode::new_expression(
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
        while let Kind::EqualEqual | Kind::BangEqual = self.current.kind {
            let operator = self.advance();
            let right = self.comparison();
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = AstNode::new_expression(
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

    fn comparison(&mut self) -> AstNode {
        let mut node = self.addition();
        while let Kind::Less | Kind::LessEqual | Kind::Greater | Kind::GreaterEqual =
            self.current.kind
        {
            let operator = self.advance();
            let right = self.addition();
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = AstNode::new_expression(
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

    fn addition(&mut self) -> AstNode {
        let mut node = self.multiplication();

        while self.current.kind == Kind::Plus || self.current.kind == Kind::Minus {
            let operator = self.advance();
            let right = self.multiplication();
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = AstNode::new_expression(
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

            node = AstNode::new_expression(
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

                AstNode::new_expression(
                    Expression::Unary {
                        operator,
                        expression: Box::new(expression),
                    },
                    new_span,
                )
            }
            _ => self.call(),
        }
    }

    fn argument_list(&mut self) -> Vec<AstNode> {
        let mut args = vec![];
        args.push(self.expression());
        while self.current.kind == Kind::Comma {
            self.advance();
            args.push(self.expression());
        }

        args
    }

    fn call(&mut self) -> AstNode {
        let primary = self.primary();

        if self.current.kind == Kind::LeftParen {
            self.advance();

            let arguments = match self.current.kind {
                Kind::RightParen => vec![],
                _ => self.argument_list(),
            };

            let new_span = match self.eat(Kind::RightParen, "Expected ')' after argument list.") {
                Ok(rparen) => Span::merge(vec![&primary.span, &rparen.span]),
                Err(()) => return AstNode::none(),
            };

            AstNode::new_expression(
                Expression::Call {
                    target: Box::new(primary),
                    arguments,
                },
                new_span,
            )
        } else {
            primary
        }
    }

    fn primary(&mut self) -> AstNode {
        match self.current.kind {
            Kind::LeftParen => {
                let lparen = self.advance();
                let expression = self.expression();
                match self.eat(Kind::RightParen, "Expected ')' after expression.") {
                    Ok(rparen) => {
                        let new_span =
                            Span::merge(vec![&lparen.span, &expression.span, &rparen.span]);
                        AstNode::new_ast_node(expression, new_span)
                    }
                    Err(()) => AstNode::none(),
                }
            }
            Kind::IdentifierLiteral => {
                let literal = self.advance();
                let span = literal.span;
                AstNode::new_expression(Expression::Variable { name: literal }, span)
            }
            Kind::NumberLiteral => self.number(),
            Kind::StringLiteral => self.string(),
            Kind::True => {
                let literal = self.advance();
                let span = literal.span;
                AstNode::new_expression(
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
                AstNode::new_expression(
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
                AstNode::new_expression(
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
                .unwrap_or_else(|_| panic!("Failed to parse '{}' as f64", literal.string)),
        );

        AstNode::new_expression(Expression::Constant { literal, value }, span)
    }

    fn string(&mut self) -> AstNode {
        let literal = self.advance();
        let span = literal.span;
        let value = Value::from(&literal.string[1..literal.string.len() - 1]);

        AstNode::new_expression(Expression::Constant { literal, value }, span)
    }

    fn advance(&mut self) -> Token {
        let previous = self.current.clone();
        self.current = self.next.clone();
        loop {
            self.next = self.scanner.next().unwrap();
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
            self.synchronize();
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

        self.errors.push(ParsingError {
            message: message.to_string(),
            span: *span,
        });
    }

    /// Consume tokens until current is '{', '}', or the token after a ';'
    fn synchronize(&mut self) {
        loop {
            match self.current.kind {
                Kind::Semicolon | Kind::Eof => {
                    self.advance();
                    break;
                }
                Kind::LeftBrace | Kind::RightBrace => {
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
