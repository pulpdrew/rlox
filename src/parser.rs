use crate::ast::{AstNode, SpannedAstNode};
use crate::error::ParsingError;
use crate::scanner::Scanner;
use crate::token::{Kind, Span, Token};
use crate::value::Value;
use std::iter::Peekable;

#[derive(Debug)]
pub struct Parser<'a> {
    scanner: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let scanner = Scanner::new(&source).peekable();
        Parser { scanner }
    }

    /// Parse the source into a program - a list of declaration `AstNode`s
    pub fn parse_program(&mut self) -> Result<Vec<SpannedAstNode>, Vec<ParsingError>> {
        let mut program = vec![];
        let mut errors = vec![];

        while self.has_next() {
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
        match self.next().kind {
            Kind::Var => self.var_declaration(),
            Kind::Class => self.class_declaration(),
            Kind::Fun => {
                self.advance();
                self.function_declaration()
            }
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        let (name, _) = self.id_token()?;

        let initializer = if self.next().kind == Kind::Equal {
            self.advance();
            let initializer = self.expression()?;
            Some(Box::new(initializer))
        } else {
            None
        };

        let semi = self.eat(Kind::Semicolon)?;
        let span = Span::merge(vec![&keyword.span, &semi.span]);
        Ok(SpannedAstNode::new(
            AstNode::VarDeclaration { name, initializer },
            span,
        ))
    }

    fn class_declaration(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.eat(Kind::Class)?;
        let (name, _) = self.id_token()?;

        let superclass = if let Kind::Less = self.next().kind {
            self.advance();
            let (superclass_name, superclass_span) = self.id_token()?;
            if superclass_name == name {
                return Err(ParsingError::SelfInheritance {
                    span: superclass_span,
                });
            }
            Some(superclass_name)
        } else {
            None
        };

        // Parse the class body (methods)
        self.eat(Kind::LeftBrace)?;
        let mut methods = vec![];
        while self.next().kind != Kind::RightBrace {
            methods.push(self.function_declaration()?);
        }
        let end_brace = self.eat(Kind::RightBrace)?;

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

    fn function_declaration(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let (name, name_span) = self.id_token()?;
        self.eat(Kind::LeftParen)?;

        let parameters = match self.next().kind {
            Kind::RightParen => vec![],
            Kind::IdentifierLiteral(_) => self.parameter_list()?,
            _ => {
                return Err(ParsingError::UnexpectedToken {
                    expected: "parameter list or ')'.".to_string(),
                    actual: self.advance(),
                })
            }
        };

        self.eat(Kind::RightParen)?;
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
        match self.next().kind {
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
        let semi = self.eat(Kind::Semicolon)?;
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

        let (value, span) = match self.next().kind {
            Kind::Semicolon => (None, keyword.span),
            _ => {
                let expr = self.expression()?;
                let span = Span::merge(vec![&keyword.span, &expr.span]);
                (Some(Box::new(expr)), span)
            }
        };

        self.eat(Kind::Semicolon)?;
        Ok(SpannedAstNode::new(AstNode::Return { value }, span))
    }

    fn for_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        self.eat(Kind::LeftParen)?;

        let initializer = match self.next().kind {
            Kind::Var => Some(Box::new(self.var_declaration()?)),
            Kind::Semicolon => {
                self.advance();
                None
            }
            _ => Some(Box::new(self.expression_statement()?)),
        };

        let condition = match self.next().kind {
            Kind::Semicolon => None,
            _ => Some(Box::new(self.expression()?)),
        };

        self.eat(Kind::Semicolon)?;

        let update = match self.next().kind {
            Kind::RightParen => None,
            _ => Some(Box::new(self.expression()?)),
        };

        self.eat(Kind::RightParen)?;

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
        self.eat(Kind::LeftParen)?;

        let condition = self.expression()?;
        self.eat(Kind::RightParen)?;

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
        self.eat(Kind::LeftParen)?;
        let condition = self.expression()?;

        self.eat(Kind::RightParen)?;

        let if_block = self.statement()?;
        let mut span = Span::merge(vec![&keyword.span, &if_block.span]);

        let else_block = if let Kind::Else = self.next().kind {
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
            match self.next().kind {
                Kind::RightBrace => break,
                _ => declarations.push(self.declaration()?),
            }
        }

        let rbrace = self.eat(Kind::RightBrace)?;
        let new_span = Span::merge(vec![&lbrace.span, &rbrace.span]);
        Ok(SpannedAstNode::new(
            AstNode::Block { declarations },
            new_span,
        ))
    }

    fn print_statement(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let keyword = self.advance();
        let expression = self.expression()?;
        let semi = self.eat(Kind::Semicolon)?;
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
        let node = self.or()?;

        if self.next().kind == Kind::Equal {
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

    fn or(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.and()?;
        while self.next().kind == Kind::Or {
            self.advance();
            let right = self.and()?;
            let new_span = Span::merge(vec![&node.span, &right.span]);

            node = SpannedAstNode::new(
                AstNode::Or {
                    left: Box::new(node),
                    right: Box::new(right),
                },
                new_span,
            );
        }

        Ok(node)
    }

    fn and(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.equality()?;
        while self.next().kind == Kind::And {
            self.advance();
            let right = self.equality()?;
            let new_span = Span::merge(vec![&node.span, &right.span]);

            node = SpannedAstNode::new(
                AstNode::And {
                    left: Box::new(node),
                    right: Box::new(right),
                },
                new_span,
            );
        }

        Ok(node)
    }

    fn equality(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.comparison()?;
        while let Kind::EqualEqual | Kind::BangEqual = self.next().kind {
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
            self.next().kind
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

        while self.next().kind == Kind::Plus || self.next().kind == Kind::Minus {
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

        while self.next().kind == Kind::Star || self.next().kind == Kind::Slash {
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
        match self.next().kind {
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
        while self.next().kind == Kind::Comma {
            self.advance();
            args.push(self.expression()?);
        }

        Ok(args)
    }

    fn call(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let mut node = self.primary()?;

        loop {
            match self.next().kind {
                Kind::LeftParen => {
                    self.advance();

                    let arguments = match self.next().kind {
                        Kind::RightParen => vec![],
                        _ => self.argument_list()?,
                    };

                    let rparen = self.eat(Kind::RightParen)?;

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
        match &self.next().kind {
            Kind::LeftParen => {
                let lparen = self.advance();
                let expression = self.expression()?;
                let rparen = self.eat(Kind::RightParen)?;
                let new_span = Span::merge(vec![&lparen.span, &rparen.span]);
                Ok(SpannedAstNode::respan(expression, new_span))
            }
            Kind::IdentifierLiteral(name) => Ok(SpannedAstNode::new(
                AstNode::Variable {
                    name: name.to_string(),
                },
                self.advance().span,
            )),
            Kind::NumberLiteral(_) => self.number_literal(),
            Kind::StringLiteral(_) => self.string_literal(),
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
            Kind::This => Ok(SpannedAstNode::new(
                AstNode::Variable {
                    name: "this".to_string(),
                },
                self.advance().span,
            )),
            Kind::Super => {
                let keyword_span = self.advance().span;
                self.eat(Kind::Dot)?;
                let (name, name_span) = self.id_token()?;
                Ok(SpannedAstNode::new(
                    AstNode::SuperAccess { name },
                    Span::merge(vec![&keyword_span, &name_span]),
                ))
            }
            _ => Err(ParsingError::UnexpectedToken {
                expected: "primary expression".to_string(),
                actual: self.advance(),
            }),
        }
    }

    fn number_literal(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let token = self.advance();

        if let Kind::NumberLiteral(n) = token.kind {
            Ok(SpannedAstNode::new(
                AstNode::Constant {
                    value: Value::from(n),
                },
                token.span,
            ))
        } else {
            Err(ParsingError::UnexpectedToken {
                expected: "number".to_string(),
                actual: token,
            })
        }
    }

    fn string_literal(&mut self) -> Result<SpannedAstNode, ParsingError> {
        let token = self.advance();
        if let Kind::StringLiteral(s) = token.kind {
            Ok(SpannedAstNode::new(
                AstNode::Constant {
                    value: Value::from(s),
                },
                token.span,
            ))
        } else {
            Err(ParsingError::UnexpectedToken {
                expected: "string".to_string(),
                actual: token,
            })
        }
    }

    /// Parse a parameter list and return a vector of the `Token`s that represent the paremeter names
    fn parameter_list(&mut self) -> Result<Vec<Token>, ParsingError> {
        let mut parameters = vec![];
        parameters.push(self.advance());
        while self.next().kind == Kind::Comma {
            self.advance();
            let param_name = self.advance();
            if let Kind::IdentifierLiteral(_) = param_name.kind {
                parameters.push(param_name);
            } else {
                return Err(ParsingError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    actual: param_name,
                });
            }
        }

        Ok(parameters)
    }

    /// Parse an identifier literal and return it, destructured
    fn id_token(&mut self) -> Result<(String, Span), ParsingError> {
        let token = self.advance();
        if let Kind::IdentifierLiteral(id) = token.kind {
            Ok((id, token.span))
        } else {
            Err(ParsingError::UnexpectedToken {
                expected: "identifier".to_string(),
                actual: token,
            })
        }
    }

    fn has_next(&mut self) -> bool {
        self.scanner.peek().is_some()
    }

    /// Get a reference to the next `Token` that will be returned by `advance`
    fn next(&mut self) -> &Token {
        self.scanner.peek().unwrap()
    }

    /// Return the next `Token` and advance `self.scanner` to the next `Token`
    fn advance(&mut self) -> Token {
        self.scanner.next().unwrap()
    }

    /// Advance if the current `Token` matches `kind`. Otherwise, return an error
    fn eat(&mut self, kind: Kind) -> Result<Token, ParsingError> {
        if self.next().kind == kind {
            Ok(self.advance())
        } else {
            Err(ParsingError::UnexpectedToken {
                expected: format!("'{}'", kind),
                actual: self.advance(),
            })
        }
    }

    /// Consume tokens until current is '{', '}', or the token after a ';'
    fn synchronize(&mut self) {
        loop {
            match self.next().kind {
                Kind::Semicolon => {
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
