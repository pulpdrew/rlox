use crate::ast::{AstNode, SpannedAstNode};
use crate::error::ParsingError;
use crate::scanner::Scanner;
use crate::token::{Kind, Span, Token};
use crate::value::Value;

#[derive(Debug)]
pub struct Parser {
    scanner: Scanner,
    current: Token,
    next: Token,
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
        }
    }

    /// Parse the source into a program - a list of declaratation `AstNode`s
    pub fn parse_program(&mut self) -> Result<Vec<SpannedAstNode>, Vec<ParsingError>> {
        let mut program = vec![];
        let mut errors = vec![];

        while self.current.kind != Kind::Eof {
            match self.declaration() {
                Ok(decl) => program.push(decl),
                Err(err) => {
                    self.synchronize();
                    errors.push(err);
                }
            }
        }

        if errors.is_empty() {
            Ok(program)
        } else {
            Err(errors.drain(0..).collect())
        }
    }

    fn declaration(&mut self) -> Result<SpannedAstNode, ParsingError> {
        match self.current.kind {
            Kind::Var => self.var_declaration(),
            Kind::Class => self.class_declaration(),
            Kind::Fun => {
                self.advance();
                self.function()
            }
            _ => self.statement(),
        }
    }

    fn id_token(&mut self) -> Result<(String, Span), ParsingError> {
        let Token { kind, span } = self.advance();
        if let Kind::IdentifierLiteral(id) = kind {
            Ok((id, span))
        } else {
            Err(ParsingError {
                message: "Expected identifier.".to_string(),
                span,
            })
        }
    }

    fn var_declaration(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        let (name, _) = self.id_token()?;

        let initializer = if self.current.kind == Kind::Equal {
            self.advance();
            let initializer = self.expression()?;
            Some(Box::new(initializer))
        } else {
            None
        };

        let semi = self.eat(Kind::Semicolon, "Expected ';' after declaration.")?;
        let span = Span::merge(vec![&keyword.span, &semi.span]);
        Ok(SpannedAstNode::new(
            AstNode::VarDeclaration { name, initializer },
            span,
        ))
    }

    fn class_declaration(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.eat(Kind::Class, "Expected 'class' keyword")?;
        let (name, _) = self.id_token()?;

        let superclass = if let Kind::Less = self.current.kind {
            self.advance();
            let (superclass_name, superclass_span) = self.id_token()?;
            if superclass_name == name {
                return Err(ParsingError {
                    message: format!("Class {} cannot inherit from itself", superclass_name),
                    span: superclass_span,
                });
            }
            Some(superclass_name)
        } else {
            None
        };

        self.eat(Kind::LeftBrace, "Expected '{' after class declaration")?;

        let mut methods = vec![];
        while self.current.kind != Kind::RightBrace && self.current.kind != Kind::Eof {
            let (method_name, method_name_span) = self.id_token()?;
            self.eat(Kind::LeftParen, "Expected '(' after method name")?;

            let parameters = match self.current.kind {
                Kind::RightParen => vec![],
                Kind::IdentifierLiteral(_) => self.parameter_list()?,
                _ => {
                    return Err(ParsingError {
                        message: "Expected parameter list or ')'.".to_string(),
                        span: self.current.span,
                    })
                }
            };
            self.eat(Kind::RightParen, "Expected ')' after formal parameter list")?;
            let body = self.block_statement()?;

            let span = Span::merge(vec![&method_name_span, &body.span]);

            methods.push(SpannedAstNode::new(
                AstNode::FunDeclaration {
                    name: method_name,
                    parameters,
                    body: Box::new(body),
                },
                span,
            ));
        }

        let end_brace = self.eat(Kind::RightBrace, "Expected '}' after class body")?;
        let span = Span::merge(vec![&keyword.span, &end_brace.span]);
        Ok(SpannedAstNode::new(
            AstNode::ClassDeclaration {
                name,
                methods,
                superclass,
            },
            span,
        ))
    }

    fn parameter_list(&mut self) -> Result<Vec<Token>, ParsingError> {
        let mut parameters = vec![];
        parameters.push(self.advance());
        while self.current.kind == Kind::Comma {
            self.advance();
            let param_name = self.advance();
            if let Kind::IdentifierLiteral(_) = param_name.kind {
                parameters.push(param_name);
            } else {
                return Err(ParsingError {
                    message: "Expected parameter name.".to_string(),
                    span: param_name.span,
                });
            }
        }

        Ok(parameters)
    }

    fn function(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let (name, name_span) = self.id_token()?;
        self.eat(Kind::LeftParen, "Expected '(' after function name")?;

        let parameters = match self.current.kind {
            Kind::RightParen => vec![],
            Kind::IdentifierLiteral(_) => self.parameter_list()?,
            _ => {
                return Err(ParsingError {
                    message: "Expected parameter list or ')'.".to_string(),
                    span: self.current.span,
                })
            }
        };

        self.eat(Kind::RightParen, "Expected ')' after formal parameter list")?;
        let body = self.block_statement()?;
        let span = Span::merge(vec![&name_span, &body.span]);

        Ok(SpannedAstNode::new(
            AstNode::FunDeclaration {
                name,
                parameters,
                body: Box::new(body),
            },
            span,
        ))
    }

    fn statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
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

    fn expression_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let expression = self.expression()?;
        let semi = self.eat(Kind::Semicolon, "Expected ';' after expression")?;
        let new_span = Span::merge(vec![&expression.span, &semi.span]);
        Ok(SpannedAstNode::new(
            AstNode::ExpressionStmt {
                expression: Box::new(expression),
            },
            new_span,
        ))
    }

    fn return_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();

        let (value, span) = match self.current.kind {
            Kind::Semicolon => (None, keyword.span),
            _ => {
                let expr = self.expression()?;
                let span = Span::merge(vec![&keyword.span, &expr.span]);
                (Some(Box::new(expr)), span)
            }
        };

        self.eat(Kind::Semicolon, "Expected ';' after return statement.")?;
        Ok(SpannedAstNode::new(AstNode::Return { value }, span))
    }

    fn for_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        self.eat(Kind::LeftParen, "Expected '(' after 'for.'")?;

        let initializer = match self.current.kind {
            Kind::Var => Some(Box::new(self.var_declaration()?)),
            Kind::Semicolon => {
                self.advance();
                None
            }
            _ => Some(Box::new(self.expression_statement()?)),
        };

        let condition = match self.current.kind {
            Kind::Semicolon => None,
            _ => Some(Box::new(self.expression()?)),
        };

        self.eat(Kind::Semicolon, "Expected ';' after for condition.")?;

        let update = match self.current.kind {
            Kind::RightParen => None,
            _ => Some(Box::new(self.expression()?)),
        };

        self.eat(Kind::RightParen, "Expected ')' before for block.")?;

        let block = self.statement()?;
        let span = Span::merge(vec![&keyword.span, &block.span]);

        Ok(SpannedAstNode::new(
            AstNode::For {
                initializer,
                condition,
                update,
                block: Box::new(block),
            },
            span,
        ))
    }

    fn while_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        self.eat(Kind::LeftParen, "Expected '(' after 'while.'")?;

        let condition = self.expression()?;
        self.eat(Kind::RightParen, "Expected ')' after while condition.")?;

        let block = self.statement()?;
        let span = Span::merge(vec![&keyword.span, &block.span]);

        Ok(SpannedAstNode::new(
            AstNode::While {
                condition: Box::new(condition),
                block: Box::new(block),
            },
            span,
        ))
    }

    fn if_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        self.eat(Kind::LeftParen, "Expected '(' after 'if.'")?;
        let condition = self.expression()?;

        self.eat(Kind::RightParen, "Expected ')' after if condition.")?;

        let if_block = self.statement()?;
        let mut span = Span::merge(vec![&keyword.span, &if_block.span]);

        let else_block = if let Kind::Else = self.current.kind {
            self.advance();
            let stmt = self.statement()?;
            span = Span::merge(vec![&span, &stmt.span]);
            Some(Box::new(stmt))
        } else {
            None
        };

        Ok(SpannedAstNode::new(
            AstNode::If {
                condition: Box::new(condition),
                if_block: Box::new(if_block),
                else_block,
            },
            span,
        ))
    }

    fn block_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let lbrace = self.advance();

        let mut declarations = vec![];
        loop {
            match self.current.kind {
                Kind::RightBrace | Kind::Eof => break,
                _ => declarations.push(self.declaration()?),
            }
        }

        let rbrace = self.eat(Kind::RightBrace, "Expected '}' after block statement")?;
        let new_span = Span::merge(vec![&lbrace.span, &rbrace.span]);
        Ok(SpannedAstNode::new(
            AstNode::Block { declarations },
            new_span,
        ))
    }

    fn print_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        let expression = self.expression()?;
        let semi = self.eat(Kind::Semicolon, "Expected ';' after print statement")?;
        let new_span = Span::merge(vec![&keyword.span, &expression.span, &semi.span]);
        Ok(SpannedAstNode::new(
            AstNode::Print {
                expression: Box::new(expression),
            },
            new_span,
        ))
    }

    fn expression(&mut self) -> Result<SpannedAstNode, ParsingError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let node = self.logic_or()?;

        if self.current.kind == Kind::Equal {
            self.advance();
            let rvalue = self.assignment()?;
            let new_span = Span::merge(vec![&node.span, &rvalue.span]);

            Ok(SpannedAstNode::new(
                AstNode::Assignment {
                    lvalue: Box::new(node),
                    rvalue: Box::new(rvalue),
                },
                new_span,
            ))
        } else {
            Ok(node)
        }
    }

    fn logic_or(&mut self) -> Result<SpannedAstNode, ParsingError> {
        self.logic_and()
    }

    fn logic_and(&mut self) -> Result<SpannedAstNode, ParsingError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.comparison()?;
        while let Kind::EqualEqual | Kind::BangEqual = self.current.kind {
            let operator = self.advance();
            let right = self.comparison()?;
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = SpannedAstNode::new(
                AstNode::Binary {
                    left: Box::new(node),
                    operator,
                    right: Box::new(right),
                },
                new_span,
            );
        }
        Ok(node)
    }

    fn comparison(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.addition()?;
        while let Kind::Less | Kind::LessEqual | Kind::Greater | Kind::GreaterEqual =
            self.current.kind
        {
            let operator = self.advance();
            let right = self.addition()?;
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = SpannedAstNode::new(
                AstNode::Binary {
                    left: Box::new(node),
                    operator,
                    right: Box::new(right),
                },
                new_span,
            );
        }
        Ok(node)
    }

    fn addition(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.multiplication()?;

        while self.current.kind == Kind::Plus || self.current.kind == Kind::Minus {
            let operator = self.advance();
            let right = self.multiplication()?;
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = SpannedAstNode::new(
                AstNode::Binary {
                    left: Box::new(node),
                    operator,
                    right: Box::new(right),
                },
                new_span,
            );
        }

        Ok(node)
    }

    fn multiplication(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.unary()?;

        while self.current.kind == Kind::Star || self.current.kind == Kind::Slash {
            let operator = self.advance();
            let right = self.unary()?;
            let new_span = Span::merge(vec![&node.span, &operator.span, &right.span]);

            node = SpannedAstNode::new(
                AstNode::Binary {
                    left: Box::new(node),
                    operator,
                    right: Box::new(right),
                },
                new_span,
            );
        }

        Ok(node)
    }

    fn unary(&mut self) -> Result<SpannedAstNode, ParsingError> {
        match self.current.kind {
            Kind::Minus | Kind::Bang => {
                let operator = self.advance();
                let expression = self.unary()?;
                let new_span = Span::new(expression.span.start - 1, expression.span.end);

                Ok(SpannedAstNode::new(
                    AstNode::Unary {
                        operator,
                        expression: Box::new(expression),
                    },
                    new_span,
                ))
            }
            _ => self.call(),
        }
    }

    fn argument_list(&mut self) -> Result<Vec<SpannedAstNode>, ParsingError> {
        let mut args = vec![];
        args.push(self.expression()?);
        while self.current.kind == Kind::Comma {
            self.advance();
            args.push(self.expression()?);
        }

        Ok(args)
    }

    fn call(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.primary()?;

        loop {
            match self.current.kind {
                Kind::LeftParen => {
                    self.advance();

                    let arguments = match self.current.kind {
                        Kind::RightParen => vec![],
                        _ => self.argument_list()?,
                    };

                    let rparen = self.eat(Kind::RightParen, "Expected ')' after argument list.")?;

                    let new_span = Span::merge(vec![&node.span, &rparen.span]);

                    node = SpannedAstNode::new(
                        AstNode::Invokation {
                            target: Box::new(node),
                            arguments,
                        },
                        new_span,
                    )
                }
                Kind::Dot => {
                    self.advance();
                    let (field_name, field_span) = self.id_token()?;
                    node = SpannedAstNode::new(
                        AstNode::FieldAccess {
                            target: Box::new(node),
                            name: field_name,
                        },
                        field_span,
                    )
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn primary(&mut self) -> Result<SpannedAstNode, ParsingError> {
        match self.current.clone().kind {
            Kind::LeftParen => {
                let lparen = self.advance();
                let expression = self.expression()?;
                let rparen = self.eat(Kind::RightParen, "Expected ')' after expression.")?;
                let new_span = Span::merge(vec![&lparen.span, &expression.span, &rparen.span]);
                Ok(SpannedAstNode::respan(expression, new_span))
            }
            Kind::IdentifierLiteral(name) => Ok(SpannedAstNode::new(
                AstNode::Variable { name },
                self.advance().span,
            )),
            Kind::NumberLiteral(_) => self.number(),
            Kind::StringLiteral(_) => self.string(),
            Kind::True => Ok(SpannedAstNode::new(
                AstNode::Constant {
                    value: Value::Bool(true),
                },
                self.advance().span,
            )),
            Kind::False => Ok(SpannedAstNode::new(
                AstNode::Constant {
                    value: Value::Bool(false),
                },
                self.advance().span,
            )),
            Kind::Nil => {
                let literal = self.advance();
                let span = literal.span;
                Ok(SpannedAstNode::new(
                    AstNode::Constant { value: Value::Nil },
                    span,
                ))
            }
            Kind::This => {
                let literal = self.advance();
                let span = literal.span;
                Ok(SpannedAstNode::new(
                    AstNode::Variable {
                        name: "this".to_string(),
                    },
                    span,
                ))
            }
            Kind::Super => {
                let keyword_span = self.advance().span;
                self.eat(Kind::Dot, "Expected '.' after 'super'.")?;
                let (name, name_span) = self.id_token()?;
                Ok(SpannedAstNode::new(
                    AstNode::SuperAccess { name },
                    Span::merge(vec![&keyword_span, &name_span]),
                ))
            }
            _ => Err(ParsingError {
                span: self.current.span,
                message: "Expected primary expression.".to_string(),
            }),
        }
    }

    fn number(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let Token { kind, span } = self.advance();

        if let Kind::NumberLiteral(n) = kind {
            Ok(SpannedAstNode::new(
                AstNode::Constant {
                    value: Value::from(n),
                },
                span,
            ))
        } else {
            Err(ParsingError {
                span,
                message: "Expected a NumberLiteral.".to_string(),
            })
        }
    }

    fn string(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let Token { kind, span } = self.advance();
        if let Kind::StringLiteral(s) = kind {
            Ok(SpannedAstNode::new(
                AstNode::Constant {
                    value: Value::from(s),
                },
                span,
            ))
        } else {
            Err(ParsingError {
                span,
                message: "Expected a StringLiteral.".to_string(),
            })
        }
    }

    fn advance(&mut self) -> Token {
        let previous = self.current.clone();
        self.current = self.next.clone();
        self.next = self.scanner.next().unwrap();
        previous
    }
    fn eat(&mut self, kind: Kind, message: &str) -> Result<Token, ParsingError> {
        if self.current.kind == kind {
            Ok(self.advance())
        } else {
            Err(ParsingError {
                message: message.to_string(),
                span: self.current.span,
            })
        }
    }

    /// Consume tokens until current is '{', '}', or the token after a ';'
    fn synchronize(&mut self) {
        self.advance();
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
    }
}
