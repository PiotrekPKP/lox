use std::fmt;

use crate::lox_type::{LoxNumber, LoxString};

#[derive(Clone)]
pub struct TokenValue {
    pub lexeme: String,
    pub line: usize,
}

#[derive(Clone)]
pub struct TokenValueString {
    pub lexeme: String,
    pub line: usize,
    pub value: LoxString,
}

#[derive(Clone)]
pub struct TokenValueNumber {
    pub lexeme: String,
    pub line: usize,
    pub value: LoxNumber,
}

#[derive(Clone)]
pub struct TokenValueKeyword {
    pub lexeme: String,
    pub line: usize,
    pub keyword: Keyword,
}

#[derive(Clone)]
pub struct TokenValueEof {
    pub line: usize,
}

#[derive(Clone)]
pub enum Keyword {
    And,
    Break,
    Continue,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Identifier(String),
}

impl From<&str> for Keyword {
    fn from(s: &str) -> Self {
        match s {
            "and" => Keyword::And,
            "break" => Keyword::Break,
            "continue" => Keyword::Continue,
            "class" => Keyword::Class,
            "else" => Keyword::Else,
            "false" => Keyword::False,
            "fun" => Keyword::Fun,
            "for" => Keyword::For,
            "if" => Keyword::If,
            "nil" => Keyword::Nil,
            "or" => Keyword::Or,
            "print" => Keyword::Print,
            "return" => Keyword::Return,
            "super" => Keyword::Super,
            "this" => Keyword::This,
            "true" => Keyword::True,
            "var" => Keyword::Var,
            "while" => Keyword::While,
            _ => Keyword::Identifier(s.to_string()),
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keyword::And => write!(f, "and"),
            Keyword::Break => write!(f, "break"),
            Keyword::Continue => write!(f, "continue"),
            Keyword::Class => write!(f, "class"),
            Keyword::Else => write!(f, "else"),
            Keyword::False => write!(f, "false"),
            Keyword::Fun => write!(f, "fun"),
            Keyword::For => write!(f, "for"),
            Keyword::If => write!(f, "if"),
            Keyword::Nil => write!(f, "nil"),
            Keyword::Or => write!(f, "or"),
            Keyword::Print => write!(f, "print"),
            Keyword::Return => write!(f, "return"),
            Keyword::Super => write!(f, "super"),
            Keyword::This => write!(f, "this"),
            Keyword::True => write!(f, "true"),
            Keyword::Var => write!(f, "var"),
            Keyword::While => write!(f, "while"),
            Keyword::Identifier(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Clone)]
pub enum Token {
    // Single character tokens.
    LeftParen(TokenValue),
    RightParen(TokenValue),
    LeftBrace(TokenValue),
    RightBrace(TokenValue),
    Comma(TokenValue),
    Dot(TokenValue),
    Minus(TokenValue),
    Plus(TokenValue),
    Semicolon(TokenValue),
    Slash(TokenValue),
    Star(TokenValue),
    QuestionMark(TokenValue),
    Colon(TokenValue),

    // One or two character tokens.
    Bang(TokenValue),
    BangEqual(TokenValue),
    Equal(TokenValue),
    EqualEqual(TokenValue),
    Greater(TokenValue),
    GreaterEqual(TokenValue),
    Less(TokenValue),
    LessEqual(TokenValue),

    // Literals.
    Keyword(TokenValueKeyword),
    String(TokenValueString),
    Number(TokenValueNumber),

    Eof(TokenValueEof),
}

impl Token {
    pub fn line(&self) -> usize {
        let l = match self {
            Token::Keyword(t) => t.line,
            Token::String(t) => t.line,
            Token::Number(t) => t.line,
            Token::Eof(t) => t.line,
            Token::LeftParen(t)
            | Token::RightParen(t)
            | Token::LeftBrace(t)
            | Token::RightBrace(t)
            | Token::Comma(t)
            | Token::Dot(t)
            | Token::Minus(t)
            | Token::Plus(t)
            | Token::Semicolon(t)
            | Token::Slash(t)
            | Token::Star(t)
            | Token::QuestionMark(t)
            | Token::Colon(t)
            | Token::Bang(t)
            | Token::BangEqual(t)
            | Token::Equal(t)
            | Token::EqualEqual(t)
            | Token::Greater(t)
            | Token::GreaterEqual(t)
            | Token::Less(t)
            | Token::LessEqual(t) => t.line,
        };

        return l;
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Single character tokens
            Token::LeftParen(tv) => write!(f, "LeftParen '{}'", tv.lexeme),
            Token::RightParen(tv) => write!(f, "RightParen '{}'", tv.lexeme),
            Token::LeftBrace(tv) => write!(f, "LeftBrace '{}'", tv.lexeme),
            Token::RightBrace(tv) => write!(f, "RightBrace '{}'", tv.lexeme),
            Token::Comma(tv) => write!(f, "Comma '{}'", tv.lexeme),
            Token::Dot(tv) => write!(f, "Dot '{}'", tv.lexeme),
            Token::Minus(tv) => write!(f, "Minus '{}'", tv.lexeme),
            Token::Plus(tv) => write!(f, "Plus '{}'", tv.lexeme),
            Token::Semicolon(tv) => write!(f, "Semicolon '{}'", tv.lexeme),
            Token::Slash(tv) => write!(f, "Slash '{}'", tv.lexeme),
            Token::Star(tv) => write!(f, "Star '{}'", tv.lexeme),
            Token::QuestionMark(tv) => write!(f, "QuestionMark '{}'", tv.lexeme),
            Token::Colon(tv) => write!(f, "Colon '{}'", tv.lexeme),

            // One or two character tokens
            Token::Bang(tv) => write!(f, "Bang '{}'", tv.lexeme),
            Token::BangEqual(tv) => write!(f, "BangEqual '{}'", tv.lexeme),
            Token::Equal(tv) => write!(f, "Equal '{}'", tv.lexeme),
            Token::EqualEqual(tv) => write!(f, "EqualEqual '{}'", tv.lexeme),
            Token::Greater(tv) => write!(f, "Greater '{}'", tv.lexeme),
            Token::GreaterEqual(tv) => write!(f, "GreaterEqual '{}'", tv.lexeme),
            Token::Less(tv) => write!(f, "Less '{}'", tv.lexeme),
            Token::LessEqual(tv) => write!(f, "LessEqual '{}'", tv.lexeme),

            // Literals
            Token::Keyword(tv) => write!(f, "Identifier '{}'", tv.lexeme),
            Token::String(tv) => write!(f, "String \"{}\"", tv.value),
            Token::Number(tv) => write!(f, "Number {}", tv.value),

            // EOF
            Token::Eof(_) => write!(f, "EOF"),
        }
    }
}
