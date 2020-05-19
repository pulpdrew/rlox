use std::cmp;
use std::fmt;

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
    Error { message: String, source: String },
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::LeftBrace => write!(f, "{{"),
            Kind::RightBrace => write!(f, "}}"),
            Kind::LeftParen => write!(f, "("),
            Kind::RightParen => write!(f, ")"),
            Kind::Comma => write!(f, ","),
            Kind::Dot => write!(f, "."),
            Kind::Minus => write!(f, "-"),
            Kind::Plus => write!(f, "+"),
            Kind::Slash => write!(f, "/"),
            Kind::Star => write!(f, "*"),
            Kind::Semicolon => write!(f, ";"),
            Kind::Bang => write!(f, "!"),
            Kind::BangEqual => write!(f, "!="),
            Kind::Equal => write!(f, "="),
            Kind::EqualEqual => write!(f, "=="),
            Kind::Greater => write!(f, ">"),
            Kind::GreaterEqual => write!(f, ">="),
            Kind::Less => write!(f, "<"),
            Kind::LessEqual => write!(f, "<="),
            Kind::IdentifierLiteral(id) => write!(f, "{}", id),
            Kind::StringLiteral(s) => write!(f, "{}", s),
            Kind::NumberLiteral(n) => write!(f, "{}", n),
            Kind::And => write!(f, "and"),
            Kind::Or => write!(f, "or"),
            Kind::Class => write!(f, "class"),
            Kind::Fun => write!(f, "fun"),
            Kind::Var => write!(f, "var"),
            Kind::If => write!(f, "if"),
            Kind::Else => write!(f, "else"),
            Kind::While => write!(f, "while"),
            Kind::For => write!(f, "for"),
            Kind::True => write!(f, "true"),
            Kind::False => write!(f, "false"),
            Kind::Nil => write!(f, "nil"),
            Kind::Print => write!(f, "print"),
            Kind::Return => write!(f, "return"),
            Kind::Super => write!(f, "super"),
            Kind::This => write!(f, "this"),
            Kind::Eof => write!(f, "end of file"),
            Kind::Error { message, .. } => write!(f, "{}", message),
        }
    }
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
