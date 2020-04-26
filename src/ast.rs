use crate::token::Token;
use crate::value::Value;

/// An expression is an AST Node that results in a Value
///  being produced at runtime.
#[derive(Debug)]
pub enum Expression {
    Unary {
        operator: Token,
        expression: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Constant {
        value: Value,
        literal: Token,
    },
    True {
        literal: Token,
    },
    False {
        literal: Token,
    },
    /// The None variant indicates a parsing failure. Any AST
    /// with a None node is invalid and should not be compiled.
    None,
}

/// A Statement is an AST Node that does not result in a
/// Value being produced at runtime.
#[derive(Debug)]
pub enum Statement {
    Expression {
        expression: Box<Expression>,
        semi: Token,
    },
    Print {
        keyword: Token,
        expression: Box<Expression>,
        semi: Token,
    },
    /// The None variant indicates a parsing failure. Any AST
    /// with a None node is invalid and should not be compiled.
    None,
}
