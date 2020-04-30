use crate::token::{Span, Token};
use crate::value::Value;

#[derive(Debug)]
pub struct AstNode {
    pub span: Span,
    expression: Option<Box<Expression>>,
    statement: Option<Box<Statement>>,
}

impl AstNode {
    pub fn new_statement(statement: Statement, span: Span) -> Self {
        AstNode {
            span,
            expression: None,
            statement: Some(Box::new(statement)),
        }
    }
    pub fn new_expression(expression: Expression, span: Span) -> Self {
        AstNode {
            span,
            expression: Some(Box::new(expression)),
            statement: None,
        }
    }
    pub fn new_ast_node(node: AstNode, span: Span) -> Self {
        AstNode {
            span,
            expression: node.expression,
            statement: node.statement,
        }
    }
    pub fn none() -> Self {
        AstNode {
            span: Span::new(0, 0),
            expression: None,
            statement: None,
        }
    }
    pub fn expression(&self) -> &Expression {
        self.expression
            .as_ref()
            .expect("Failed to unwrap expression.")
    }
    pub fn statement(&self) -> &Statement {
        self.statement
            .as_ref()
            .expect("Failed to unwrap statement.")
    }
}

/// An expression is an AST Node that results in a Value
/// being produced at runtime.
#[derive(Debug)]
pub enum Expression {
    Unary {
        operator: Token,
        expression: Box<AstNode>,
    },
    Binary {
        left: Box<AstNode>,
        operator: Token,
        right: Box<AstNode>,
    },
    Assignment {
        lvalue: Box<AstNode>,
        operator: Token,
        rvalue: Box<AstNode>,
    },
    Variable {
        name: Token,
    },
    Constant {
        value: Value,
        literal: Token,
    },
}

/// A Statement is an AST Node that does not result in a
/// Value being produced at runtime.
#[derive(Debug)]
pub enum Statement {
    Expression {
        expression: Box<AstNode>,
    },
    Print {
        keyword: Token,
        expression: Box<AstNode>,
    },
    Declaration {
        name: Token,
        operator: Option<Token>,
        initializer: Option<Box<AstNode>>,
    },
    Block {
        declarations: Vec<AstNode>,
        rbrace: Token,
    },
    If {
        keyword: Token,
        condition: Box<AstNode>,
        if_block: Box<AstNode>,
        else_block: Option<Box<AstNode>>,
    },
}
