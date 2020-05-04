use crate::token::{Span, Token};
use crate::value::Value;

/// Contains either an expression or a statement node, tagged with a Span `span`
#[derive(Debug)]
pub struct AstNode {
    pub span: Span,
    expression: Option<Box<Expression>>,
    statement: Option<Box<Statement>>,
}

impl AstNode {
    /// Create and return a new AstNode with the given `statement` and `span`
    pub fn new_statement(statement: Statement, span: Span) -> Self {
        AstNode {
            span,
            expression: None,
            statement: Some(Box::new(statement)),
        }
    }
    /// Create and return a new AstNode with the given `expression` and `span`
    pub fn new_expression(expression: Expression, span: Span) -> Self {
        AstNode {
            span,
            expression: Some(Box::new(expression)),
            statement: None,
        }
    }
    /// Create and return a new AstNode by copying the given `node` and tagging it
    /// with the given `span`
    pub fn new_ast_node(node: AstNode, span: Span) -> Self {
        AstNode {
            span,
            expression: node.expression,
            statement: node.statement,
        }
    }
    /// Create and return a new AstNode that is neither an expression or statement,
    /// representing an empty/invalid AstNode
    pub fn none() -> Self {
        AstNode {
            span: Span::new(0, 0),
            expression: None,
            statement: None,
        }
    }
    /// Get a reference to this node's expression. Panics if there is no expression.
    pub fn expression(&self) -> &Expression {
        self.expression
            .as_ref()
            .expect("Failed to unwrap expression.")
    }
    /// Get a reference to this node's statement. Panics if there is no statement.
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
    Call {
        target: Box<AstNode>,
        arguments: Vec<AstNode>,
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
        condition: Box<AstNode>,
        if_block: Box<AstNode>,
        else_block: Option<Box<AstNode>>,
    },
    While {
        condition: Box<AstNode>,
        block: Box<AstNode>,
    },
    For {
        initializer: Option<Box<AstNode>>,
        condition: Option<Box<AstNode>>,
        update: Option<Box<AstNode>>,
        block: Box<AstNode>,
    },
    FunDeclaration {
        name: Token,
        parameters: Vec<Token>,
        body: Box<AstNode>,
    },
    Return {
        value: Option<Box<AstNode>>,
    },
}
