use crate::token::Token;
use crate::value::Value;

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
    None,
}

#[derive(Debug)]
pub enum Statement {
    Expression {
        expression: Box<Expression>,
        semi: Token,
    },
    None,
}
