use std::{cell::RefCell, iter::Peekable, rc::Rc, str::Chars};

use crate::{
    CompileError, error,
    lox_type::LoxNumber,
    token::{
        Keyword, Token, TokenValue, TokenValueEof, TokenValueKeyword, TokenValueNumber,
        TokenValueString,
    },
    token_n,
};

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a String,
    chars: Peekable<Chars<'a>>,

    at_the_end: bool,
    pub errors: Rc<RefCell<Vec<CompileError>>>,

    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Self {
            source: source,
            chars: source.chars().peekable(),
            at_the_end: source.is_empty(),
            errors: Rc::new(RefCell::new(vec![])),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&mut self) -> bool {
        let at_the_end = self.current >= self.source.len();

        if at_the_end {
            self.at_the_end = true;
        }

        return at_the_end;
    }

    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        return self.is_alpha(c) || c.is_digit(10);
    }

    fn peek(&mut self) -> char {
        return *self.chars.peek().unwrap_or(&'\0');
    }

    fn peek_next(&mut self) -> char {
        let mut it = self.source[self.current..].chars();
        it.next();

        return it.next().unwrap_or('\0');
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        return self.chars.next().unwrap();
    }

    fn number(&mut self) -> Option<Token> {
        while self.peek().is_digit(10) {
            let _ = self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            let _ = self.advance();

            while self.peek().is_digit(10) {
                let _ = self.advance();
            }
        }

        return Some(Token::Number(TokenValueNumber {
            lexeme: self.source[self.start..self.current].to_string(),
            line: self.line,
            value: self.source[self.start..self.current]
                .parse::<LoxNumber>()
                .unwrap(),
        }));
    }

    fn string(&mut self) -> Option<Token> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            let _ = self.advance();
        }

        if self.is_at_end() {
            self.errors.borrow_mut().push(error(
                self.line,
                &"Unterminated string literal.".to_string(),
            ));

            return None;
        }

        let _ = self.advance();

        return Some(Token::String(TokenValueString {
            lexeme: self.source[self.start..self.current].to_string(),
            line: self.line,
            value: self.source[self.start + 1..self.current - 1].to_string(),
        }));
    }

    fn identifier(&mut self) -> Option<Token> {
        let mut c = self.peek();

        while self.is_alphanumeric(c) {
            let _ = self.advance();
            c = self.peek();
        }

        return Some(Token::Keyword(TokenValueKeyword {
            lexeme: self.source[self.start..self.current].to_string(),
            line: self.line,
            keyword: Keyword::from(&self.source[self.start..self.current]),
        }));
    }

    fn get_token_value(&self) -> TokenValue {
        TokenValue {
            lexeme: self.source[self.start..self.current].to_string(),
            line: self.line,
        }
    }

    fn matching(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if *self.chars.peek().unwrap() != expected {
            return false;
        }

        let _ = self.advance();
        return true;
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.start = self.current;

        if self.at_the_end {
            return None;
        }

        if self.is_at_end() {
            self.at_the_end = true;
            return Some(Token::Eof(TokenValueEof { line: self.line }));
        }

        let c = self.advance();

        match c {
            '(' => return Some(token_n!(self, LeftParen)),
            ')' => return Some(token_n!(self, RightParen)),
            '{' => return Some(token_n!(self, LeftBrace)),
            '}' => return Some(token_n!(self, RightBrace)),
            ',' => return Some(token_n!(self, Comma)),
            '.' => return Some(token_n!(self, Dot)),
            '-' => return Some(token_n!(self, Minus)),
            '+' => return Some(token_n!(self, Plus)),
            ';' => return Some(token_n!(self, Semicolon)),
            '*' => return Some(token_n!(self, Star)),
            '?' => return Some(token_n!(self, QuestionMark)),
            ':' => return Some(token_n!(self, Colon)),

            '!' => {
                if self.matching('=') {
                    return Some(token_n!(self, BangEqual));
                } else {
                    return Some(token_n!(self, Bang));
                }
            }
            '=' => {
                if self.matching('=') {
                    return Some(token_n!(self, EqualEqual));
                } else {
                    return Some(token_n!(self, Equal));
                }
            }
            '<' => {
                if self.matching('=') {
                    return Some(token_n!(self, LessEqual));
                } else {
                    return Some(token_n!(self, Less));
                }
            }
            '>' => {
                if self.matching('=') {
                    return Some(token_n!(self, GreaterEqual));
                } else {
                    return Some(token_n!(self, Greater));
                }
            }

            '/' => {
                if self.matching('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        let _ = self.advance();
                    }

                    return self.next();
                } else if self.matching('*') {
                    while self.peek() != '*' && self.peek_next() != '/' && !self.is_at_end() {
                        let _ = self.advance();
                    }

                    let _ = self.advance();
                    let _ = self.advance();

                    return self.next();
                } else {
                    return Some(token_n!(self, Slash));
                }
            }

            ' ' | '\r' | '\t' => return self.next(),
            '\n' => {
                self.line += 1;
                return self.next();
            }

            '"' => {
                if let Some(string_token) = self.string() {
                    return Some(string_token);
                }

                return self.next();
            }

            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0' => {
                if let Some(number_token) = self.number() {
                    return Some(number_token);
                }

                return self.next();
            }

            _ => {
                if self.is_alpha(c) {
                    if let Some(identifier_token) = self.identifier() {
                        return Some(identifier_token);
                    }

                    return self.next();
                } else {
                    self.errors
                        .borrow_mut()
                        .push(error(self.line, &format!("Unexpected character '{}'.", c)));

                    return self.next();
                }
            }
        }
    }
}
