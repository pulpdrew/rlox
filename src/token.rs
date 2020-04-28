use std::cmp;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
    pub string: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Kind {
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,
    Semicolon,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    IdentifierLiteral,
    StringLiteral,
    NumberLiteral,

    And,
    Or,
    Class,
    Fun,
    Var,
    If,
    Else,
    While,
    For,
    True,
    False,
    Nil,
    Print,
    Return,
    Super,
    This,

    Eof,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
    pub fn merge(spans: Vec<&Span>) -> Self {
        let mut start = 0;
        let mut end = 0;
        for span in spans {
            start = cmp::min(start, span.start);
            end = cmp::max(end, span.end);
        }
        Span::new(start, end)
    }
}
