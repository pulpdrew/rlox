use crate::token::{Span, Token};
use crate::value::Value;

/// Contains either an expression or a statement node, tagged with a Span `span`
#[derive(Debug)]
pub struct SpannedAstNode {
    pub span: Span,
    pub node: Option<AstNode>,
}

impl SpannedAstNode {
    /// Create and return a new AstNode by copying the given `node` and tagging it
    /// with the given `span`
    pub fn new(node: AstNode, span: Span) -> Self {
        SpannedAstNode {
            span,
            node: Some(node),
        }
    }

    pub fn respan(node: SpannedAstNode, span: Span) -> Self {
        SpannedAstNode {
            node: node.node,
            span,
        }
    }

    /// Create and return a new AstNode that is neither an expression or statement,
    /// representing an empty/invalid AstNode
    pub fn empty() -> Self {
        SpannedAstNode {
            span: Span::new(0, 0),
            node: None,
        }
    }
}

/// An expression is an AST Node that results in a Value
/// being produced at runtime.
#[derive(Debug)]
pub enum AstNode {
    Unary {
        operator: Token,
        expression: Box<SpannedAstNode>,
    },
    Binary {
        left: Box<SpannedAstNode>,
        operator: Token,
        right: Box<SpannedAstNode>,
    },
    Assignment {
        lvalue: Box<SpannedAstNode>,
        rvalue: Box<SpannedAstNode>,
    },
    Variable {
        name: String,
    },
    Constant {
        value: Value,
    },
    Call {
        target: Box<SpannedAstNode>,
        arguments: Vec<SpannedAstNode>,
    },
    ExpressionStmt {
        expression: Box<SpannedAstNode>,
    },
    Print {
        expression: Box<SpannedAstNode>,
    },
    Declaration {
        name: String,
        initializer: Option<Box<SpannedAstNode>>,
    },
    Block {
        declarations: Vec<SpannedAstNode>,
        rbrace: Token,
    },
    If {
        condition: Box<SpannedAstNode>,
        if_block: Box<SpannedAstNode>,
        else_block: Option<Box<SpannedAstNode>>,
    },
    While {
        condition: Box<SpannedAstNode>,
        block: Box<SpannedAstNode>,
    },
    For {
        initializer: Option<Box<SpannedAstNode>>,
        condition: Option<Box<SpannedAstNode>>,
        update: Option<Box<SpannedAstNode>>,
        block: Box<SpannedAstNode>,
    },
    FunDeclaration {
        name: String,
        parameters: Vec<Token>,
        body: Box<SpannedAstNode>,
    },
    Return {
        value: Option<Box<SpannedAstNode>>,
    },
}
