use std::cmp;

/// An indivisible bit of source code tagged with a `Kind` and a `Span`
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}

/// A logical classification of a `Token`
#[derive(Debug, PartialEq, Clone)]
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

    IdentifierLiteral(String),
    StringLiteral(String),
    NumberLiteral(f64),

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
    Error(String),
}

/// A region of source code with a start and an end
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Create and return a new span from a start and end index
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
    /// Create and return a new span that minimally covers all of the spans in `spans`
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
