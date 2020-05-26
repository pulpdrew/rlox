use crate::token::Kind;
use crate::token::{Span, Token};
use std::str::Chars;

/// A Scanner is an iterator over source code that returns
/// the `Token`s in the source code, in order.
#[derive(Debug)]
pub struct Scanner<'a> {
    /// An iterator over the underlying source code
    characters: Chars<'a>,

    /// The source string that that makes up the `Token` currently being built
    current: String,

    /// The index in the underlying source string at which the current token begins
    current_start_index: usize,

    /// The length of the underlying source string
    source_len: usize,
}

impl Iterator for Scanner<'_> {
    type Item = Token;

    /// Return the next `Token` in the source.
    /// If there is no next token, return Token::Eof
    fn next(&mut self) -> Option<Token> {
        if self.peek(0) == None {
            return Some(self.make_token(Kind::Eof));
        }

        self.consume_whitespace();
        if let Some(ch) = self.advance() {
            Some(match ch {
                '{' => self.make_token(Kind::LeftBrace),
                '}' => self.make_token(Kind::RightBrace),
                '(' => self.make_token(Kind::LeftParen),
                ')' => self.make_token(Kind::RightParen),
                ',' => self.make_token(Kind::Comma),
                '.' => self.make_token(Kind::Dot),
                '-' => self.make_token(Kind::Minus),
                '+' => self.make_token(Kind::Plus),
                '/' => self.make_token(Kind::Slash),
                '*' => self.make_token(Kind::Star),
                ';' => self.make_token(Kind::Semicolon),

                '!' if self.peek(0) == Some('=') => {
                    self.advance();
                    self.make_token(Kind::BangEqual)
                }
                '!' => self.make_token(Kind::Bang),
                '=' if self.peek(0) == Some('=') => {
                    self.advance();
                    self.make_token(Kind::EqualEqual)
                }
                '=' => self.make_token(Kind::Equal),
                '>' if self.peek(0) == Some('=') => {
                    self.advance();
                    self.make_token(Kind::GreaterEqual)
                }
                '>' => self.make_token(Kind::Greater),
                '<' if self.peek(0) == Some('=') => {
                    self.advance();
                    self.make_token(Kind::LessEqual)
                }
                '<' => self.make_token(Kind::Less),

                'a'..='z' | 'A'..='Z' | '_' => self.identifier_literal(),
                '0'..='9' => self.number_literal(),
                '"' => self.string_literal(),

                _ => self.make_error_token("unrecognized character"),
            })
        } else {
            Some(self.make_token(Kind::Eof))
        }
    }
}

impl<'a> Scanner<'a> {
    /// Create and return a new Scanner that reads tokens from the given `source`
    pub fn new(source: &'a str) -> Self {
        Scanner {
            characters: source.chars(),
            current: String::new(),
            current_start_index: 0,
            source_len: source.len(),
        }
    }

    /// Get the length of the underlying source string
    pub fn source_len(&self) -> usize {
        self.source_len
    }

    /// Consume a single `char` from `self.characters` and append it to `self.current`
    fn advance(&mut self) -> Option<char> {
        self.characters.next().map(|ch| {
            self.current.push(ch);
            ch
        })
    }

    /// Peek at the character `count` characters from the beginning of `self.characters`.
    fn peek(&self, count: usize) -> Option<char> {
        self.characters.as_str().chars().nth(count)
    }

    /// Consumes an identifier or keyword and makes a Token.
    fn identifier_literal(&mut self) -> Token {
        while is_digit(self.peek(0)) || is_alpha_or_under(self.peek(0)) {
            self.advance();
        }

        match self.current.as_str() {
            "and" => self.make_token(Kind::And),
            "class" => self.make_token(Kind::Class),
            "else" => self.make_token(Kind::Else),

            "false" => self.make_token(Kind::False),
            "for" => self.make_token(Kind::For),
            "fun" => self.make_token(Kind::Fun),

            "if" => self.make_token(Kind::If),
            "nil" => self.make_token(Kind::Nil),
            "or" => self.make_token(Kind::Or),
            "print" => self.make_token(Kind::Print),
            "return" => self.make_token(Kind::Return),
            "super" => self.make_token(Kind::Super),

            "this" => self.make_token(Kind::This),
            "true" => self.make_token(Kind::True),

            "var" => self.make_token(Kind::Var),
            "while" => self.make_token(Kind::While),

            _ => {
                let (source, span) = self.take_current();
                Token {
                    span,
                    kind: Kind::IdentifierLiteral(source),
                }
            }
        }
    }

    /// Consumes a number literal and makes a Token
    fn number_literal(&mut self) -> Token {
        while is_digit(self.peek(0)) {
            self.advance();
        }

        if self.peek(0) == Some('.') && is_digit(self.peek(1)) {
            self.advance();
            self.advance();

            while is_digit(self.peek(0)) {
                self.advance();
            }
        }

        let (source, span) = self.take_current();
        Token {
            span,
            kind: Kind::NumberLiteral(source.parse().unwrap()),
        }
    }

    /// Consumes a string literal and makes a Token
    fn string_literal(&mut self) -> Token {
        while let Some(ch) = self.advance() {
            if ch == '"' {
                break;
            }
        }

        if !self.current.ends_with('"') {
            return self.make_error_token("unclosed string literal");
        }

        let (source, span) = self.take_current();
        Token {
            span,
            kind: Kind::StringLiteral(source[1..(source.len() - 1)].to_string()),
        }
    }

    /// Consume `self.current` to produce a `Token` with the given kind
    fn make_token(&mut self, kind: Kind) -> Token {
        let (_, mut span) = self.take_current();
        if kind == Kind::Eof {
            span = Span::new(span.start, span.start + 1)
        }
        Token { kind, span }
    }

    /// Consume `self.current` to produce a `Token` with `Kind::Error` with the given `message`
    fn make_error_token(&mut self, message: &str) -> Token {
        let (source, span) = self.take_current();
        Token {
            span,
            kind: Kind::Error {
                message: message.to_string(),
                source,
            },
        }
    }

    /// Return the current String and its Span and prepare
    /// `current_start_index` and `current` for the next `Token`
    fn take_current(&mut self) -> (String, Span) {
        let mut content = String::new();
        std::mem::swap(&mut content, &mut self.current);

        let span = Span::new(
            self.current_start_index,
            self.current_start_index + content.len(),
        );
        self.current_start_index += content.len();

        (content, span)
    }

    /// Advances past any whitespace or comments.
    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek(0) {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                    continue;
                }
                '/' if self.peek(1) == Some('/') => {
                    while self.advance() != Some('\n') {}
                    continue;
                }
                _ => break,
            }
        }
        self.take_current();
    }
}

fn is_digit(ch: Option<char>) -> bool {
    if let Some(_c @ '0'..='9') = ch {
        true
    } else {
        false
    }
}

fn is_alpha_or_under(ch: Option<char>) -> bool {
    if let Some(c) = ch {
        match c {
            'a'..='z' | 'A'..='Z' | '_' => true,
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner;
    use crate::token::{Kind, Span};

    #[test]
    fn number_literals() {
        single_token_test(String::from("123"), Kind::NumberLiteral(123f64));
        single_token_test(String::from("123.1"), Kind::NumberLiteral(123.1f64));
        single_token_test(String::from("123.456"), Kind::NumberLiteral(123.456f64));
        single_token_test(String::from("0.456"), Kind::NumberLiteral(0.456f64));
        single_token_test(String::from("0.0"), Kind::NumberLiteral(0f64));
    }

    #[test]
    fn string_literals() {
        single_token_test(String::from("\"\""), Kind::StringLiteral("".to_string()));
        single_token_test(
            String::from("\"a string literal\""),
            Kind::StringLiteral("a string literal".to_string()),
        );
    }

    #[test]
    fn identifier_literals() {
        single_token_test(String::from("x"), Kind::IdentifierLiteral("x".to_string()));
        single_token_test(
            String::from("While"),
            Kind::IdentifierLiteral("While".to_string()),
        );
        single_token_test(String::from("_"), Kind::IdentifierLiteral("_".to_string()));
        single_token_test(
            String::from("_1"),
            Kind::IdentifierLiteral("_1".to_string()),
        );
        single_token_test(
            String::from("_abc123"),
            Kind::IdentifierLiteral("_abc123".to_string()),
        );
    }

    #[test]
    fn keywords() {
        single_token_test(String::from("and"), Kind::And);
        single_token_test(String::from("or"), Kind::Or);
        single_token_test(String::from("class"), Kind::Class);
        single_token_test(String::from("fun"), Kind::Fun);
        single_token_test(String::from("var"), Kind::Var);
        single_token_test(String::from("if"), Kind::If);
        single_token_test(String::from("else"), Kind::Else);
        single_token_test(String::from("while"), Kind::While);
        single_token_test(String::from("for"), Kind::For);
        single_token_test(String::from("true"), Kind::True);
        single_token_test(String::from("false"), Kind::False);
        single_token_test(String::from("nil"), Kind::Nil);
        single_token_test(String::from("print"), Kind::Print);
        single_token_test(String::from("return"), Kind::Return);
        single_token_test(String::from("super"), Kind::Super);
        single_token_test(String::from("this"), Kind::This);
    }

    #[test]
    fn other_tokens() {
        single_token_test(String::from("{"), Kind::LeftBrace);
        single_token_test(String::from("}"), Kind::RightBrace);
        single_token_test(String::from("("), Kind::LeftParen);
        single_token_test(String::from(")"), Kind::RightParen);
        single_token_test(String::from(","), Kind::Comma);
        single_token_test(String::from("."), Kind::Dot);
        single_token_test(String::from("-"), Kind::Minus);
        single_token_test(String::from("+"), Kind::Plus);
        single_token_test(String::from("*"), Kind::Star);
        single_token_test(String::from("/"), Kind::Slash);
        single_token_test(String::from(";"), Kind::Semicolon);
        single_token_test(String::from("!"), Kind::Bang);
        single_token_test(String::from("!="), Kind::BangEqual);
        single_token_test(String::from("="), Kind::Equal);
        single_token_test(String::from("=="), Kind::EqualEqual);
        single_token_test(String::from(">"), Kind::Greater);
        single_token_test(String::from(">="), Kind::GreaterEqual);
        single_token_test(String::from("<"), Kind::Less);
        single_token_test(String::from("<="), Kind::LessEqual);
    }

    #[test]
    fn whitespace_and_comments() {
        let source = "
            // This is a comment
            while (true) // another comment
                print \"hey   \"
        ";

        let mut scanner = scanner::Scanner::new(&source);
        assert_eq!(scanner.next().unwrap().kind, Kind::While);
        assert_eq!(scanner.next().unwrap().kind, Kind::LeftParen);
        assert_eq!(scanner.next().unwrap().kind, Kind::True);
        assert_eq!(scanner.next().unwrap().kind, Kind::RightParen);
        assert_eq!(scanner.next().unwrap().kind, Kind::Print);
        assert_eq!(
            scanner.next().unwrap().kind,
            Kind::StringLiteral("hey   ".to_string())
        );
    }

    #[test]
    fn empty_file() {
        let mut scanner = scanner::Scanner::new("");
        assert_eq!(scanner.next().unwrap().kind, Kind::Eof);
    }

    #[test]
    fn spans() {
        let source = "
long_id // This is a comment

\"string\"
        "
        .trim();

        let mut scanner = scanner::Scanner::new(&source);
        assert_eq!(scanner.next().unwrap().span, Span::new(0, 7));
        assert_eq!(scanner.next().unwrap().span, Span::new(30, 38));
        assert_eq!(scanner.next().unwrap().span, Span::new(38, 39));
        assert_eq!(scanner.next().unwrap().span, Span::new(38, 39));
    }

    fn single_token_test(source: String, expected_kind: Kind) {
        let mut scanner = scanner::Scanner::new(&source);
        let token = scanner.next();

        assert_eq!(token.as_ref().unwrap().kind, expected_kind);
        assert_eq!(scanner.next().unwrap().kind, Kind::Eof, "Expected Eof.");
    }
}
