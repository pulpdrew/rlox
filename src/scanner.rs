use crate::token::Kind;
use crate::token::{Span, Token};
use std::collections::VecDeque;

/// A Scanner is an iterator over source code that returns
/// the `Token`s in the source code, in order.
#[derive(Debug)]
pub struct Scanner {
    characters: VecDeque<char>,
    line: usize,
    index: usize,
}

impl Iterator for Scanner {
    type Item = Token;

    /// Return the next `Token` in the source code
    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();

        Some(match self.peek(0) {
            Some(ch) => match ch {
                '{' => self.make_token(Kind::LeftBrace, 1),
                '}' => self.make_token(Kind::RightBrace, 1),
                '(' => self.make_token(Kind::LeftParen, 1),
                ')' => self.make_token(Kind::RightParen, 1),
                ',' => self.make_token(Kind::Comma, 1),
                '.' => self.make_token(Kind::Dot, 1),
                '-' => self.make_token(Kind::Minus, 1),
                '+' => self.make_token(Kind::Plus, 1),
                '/' => self.make_token(Kind::Slash, 1),
                '*' => self.make_token(Kind::Star, 1),
                ';' => self.make_token(Kind::Semicolon, 1),

                '!' if self.peek(1) == Some(&'=') => self.make_token(Kind::BangEqual, 2),
                '!' => self.make_token(Kind::Bang, 1),
                '=' if self.peek(1) == Some(&'=') => self.make_token(Kind::EqualEqual, 2),
                '=' => self.make_token(Kind::Equal, 1),
                '>' if self.peek(1) == Some(&'=') => self.make_token(Kind::GreaterEqual, 2),
                '>' => self.make_token(Kind::Greater, 1),
                '<' if self.peek(1) == Some(&'=') => self.make_token(Kind::LessEqual, 2),
                '<' => self.make_token(Kind::Less, 1),

                'a'..='z' | 'A'..='Z' | '_' => self.identifier_literal(),
                '0'..='9' => self.number_literal(),
                '"' => self.string_literal(),

                _ => self.make_error_token(1),
            },
            None => self.make_token(Kind::Eof, 0),
        })
    }
}

impl Scanner {
    /// Create and return a new Scanner that operators on the given `source`
    pub fn new(source: &str) -> Self {
        Scanner {
            characters: source.to_string().chars().collect(),
            line: 1,
            index: 0,
        }
    }

    /// Consume and return the character from the front of `self.characters`.
    fn advance(&mut self) -> Option<char> {
        self.index += 1;
        self.characters.pop_front()
    }

    /// Peek at the character `count` characters from the beginning of `self.characters`.
    fn peek(&self, count: usize) -> Option<&char> {
        self.characters.get(count)
    }

    /// Consumes an identifier or keyword and makes a Token.
    fn identifier_literal(&mut self) -> Token {
        let mut length = 1;
        while is_digit(self.peek(length)) || is_alpha_or_under(self.peek(length)) {
            length += 1
        }

        match self
            .peek(0)
            .expect("Expected a character at the beginning of an identifier")
        {
            'a' if length == 3 && self.starts_with("and") => self.make_token(Kind::And, 3),
            'c' if length == 5 && self.starts_with("class") => self.make_token(Kind::Class, 5),
            'e' if length == 4 && self.starts_with("else") => self.make_token(Kind::Else, 4),

            'f' if length == 5 && self.starts_with("false") => self.make_token(Kind::False, 5),
            'f' if length == 3 && self.starts_with("for") => self.make_token(Kind::For, 3),
            'f' if length == 3 && self.starts_with("fun") => self.make_token(Kind::Fun, 3),

            'i' if length == 2 && self.starts_with("if") => self.make_token(Kind::If, 2),
            'n' if length == 3 && self.starts_with("nil") => self.make_token(Kind::Nil, 3),
            'o' if length == 2 && self.starts_with("or") => self.make_token(Kind::Or, 2),
            'p' if length == 5 && self.starts_with("print") => self.make_token(Kind::Print, 5),
            'r' if length == 6 && self.starts_with("return") => self.make_token(Kind::Return, 6),
            's' if length == 5 && self.starts_with("super") => self.make_token(Kind::Super, 5),

            't' if length == 4 && self.starts_with("this") => self.make_token(Kind::This, 4),
            't' if length == 4 && self.starts_with("true") => self.make_token(Kind::True, 4),

            'v' if length == 3 && self.starts_with("var") => self.make_token(Kind::Var, 3),
            'w' if length == 5 && self.starts_with("while") => self.make_token(Kind::While, 5),

            _ => Token {
                span: self.make_span(length),
                kind: Kind::IdentifierLiteral(self.read_front(length)),
            },
        }
    }

    /// Consumes a number literal and makes a Token
    fn number_literal(&mut self) -> Token {
        let mut length = 1;
        while is_digit(self.peek(length)) {
            length += 1
        }

        if self.peek(length) == Some(&'.') && is_digit(self.peek(length + 1)) {
            length += 2;
            while is_digit(self.peek(length)) {
                length += 1
            }
        }

        Token {
            span: self.make_span(length),
            kind: Kind::NumberLiteral(self.read_front(length).parse().unwrap()),
        }
    }

    /// Consumes a string literal and makes a Token
    fn string_literal(&mut self) -> Token {
        let mut length = 1;
        while self.peek(length) != Some(&'"') && length <= self.characters.len() {
            length += 1
        }

        if length >= self.characters.len() {
            self.make_error_token(self.characters.len() - 1)
        } else {
            Token {
                span: self.make_span(length + 1),
                kind: Kind::StringLiteral(self.read_front(length + 1)[1..length].to_string()),
            }
        }
    }

    /// Makes a token of the given `kind` out of the first `count` characters in `self.characters`.
    fn make_token(&mut self, kind: Kind, length: usize) -> Token {
        let span = match kind {
            Kind::Eof => self.make_span(1),
            _ => self.make_span(length),
        };
        self.read_front(length);
        Token { kind, span }
    }

    /// Makes an error token out the next `length` characters
    fn make_error_token(&mut self, length: usize) -> Token {
        Token {
            span: self.make_span(length),
            kind: Kind::Error(self.read_front(length)),
        }
    }

    /// Makes a span beginning at the current location in source and continuing for `length` characters
    fn make_span(&self, length: usize) -> Span {
        Span::new(self.index, self.index + length)
    }

    /// Indicates whether thte prefix of `self.characters` matches the given prefix.
    fn starts_with(&self, prefix: &str) -> bool {
        for (i, ch) in prefix.char_indices() {
            if self.peek(i) != Some(&ch) {
                return false;
            }
        }
        true
    }

    /// Advances `count` times, yielding a String from the consumed characters
    fn read_front(&mut self, count: usize) -> String {
        let mut front = String::new();

        for _ in 0..count {
            if let Some(ch) = self.advance() {
                front.push(ch);
            }
        }

        front
    }

    /// Advances past any whitespace or comments.
    fn consume_whitespace(&mut self) {
        loop {
            match self.peek(0) {
                Some(c) => match c {
                    ' ' | '\t' | '\r' => {
                        self.advance();
                        continue;
                    }
                    '\n' => {
                        self.advance();
                        self.line += 1;
                        continue;
                    }
                    '/' if self.peek(1) == Some(&'/') => {
                        while self.advance() != Some('\n') {}
                        continue;
                    }
                    _ => return,
                },
                None => return,
            };
        }
    }
}

fn is_digit(ch: Option<&char>) -> bool {
    if let Some(c) = ch {
        *c >= '0' && *c <= '9'
    } else {
        false
    }
}

fn is_alpha_or_under(ch: Option<&char>) -> bool {
    if let Some(c) = ch {
        match *c {
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
    }

    fn single_token_test(source: String, expected_kind: Kind) {
        let mut scanner = scanner::Scanner::new(&source);
        let token = scanner.next();

        assert_eq!(token.as_ref().unwrap().kind, expected_kind);
        assert_eq!(scanner.next().unwrap().kind, Kind::Eof, "Expected Eof.");
    }
}
