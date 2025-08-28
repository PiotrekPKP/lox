use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::iter::Peekable;
use std::ops::Deref;
use std::process::exit;
use std::rc::Rc;
use std::str::Chars;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::usize;

macro_rules! token {
    ($self:expr, $variant:ident) => {
        Token::$variant($self.get_token_value())
    };
}

macro_rules! parse_error {
    ($fmt:expr $(, $($arg:tt)+ )? ) => {{
        eprintln!($fmt $(, $($arg)+ )?);
        std::process::exit(1);
    }};
}

type LoxString = String;
type LoxNumber = f64;
type LoxBoolean = bool;

#[derive(Debug, Clone)]
enum LoxType {
    String(LoxString),
    Number(LoxNumber),
    Boolean(LoxBoolean),
    Nil,
    Unknown,
}

impl LoxType {
    fn is_truthy(&self) -> bool {
        match self {
            LoxType::Boolean(b) => *b,
            LoxType::Nil => false,
            LoxType::Number(n) => *n != 0.,
            _ => true,
        }
    }
}

impl std::fmt::Display for LoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Boolean(b) => write!(f, "{b}"),
            LoxType::Nil => write!(f, "nil"),
            LoxType::Number(n) => write!(f, "{n}"),
            LoxType::String(s) => write!(f, "{s}"),
            LoxType::Unknown => write!(f, "\0"),
        }
    }
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct CompileErrors(Vec<CompileError>);

#[derive(Debug, Clone)]
struct CompileError {
    line: usize,
    kind: String,
    message: String,
}

impl CompileError {
    fn new(line: usize, kind: String, message: String) -> Self {
        Self {
            line,
            kind,
            message,
        }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {}] Error{}: {}",
            self.line, self.kind, self.message
        )
    }
}

impl std::fmt::Display for CompileErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        self.0.iter().for_each(|err| {
            output.push_str(format!("{}\n", err).as_str());
        });

        write!(f, "{}", output)
    }
}

impl std::error::Error for CompileError {}
impl std::error::Error for CompileErrors {}

#[derive(Debug)]
struct Scanner<'a> {
    source: &'a String,
    chars: Peekable<Chars<'a>>,

    at_the_end: bool,
    errors: Rc<RefCell<Vec<CompileError>>>,

    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a String) -> Self {
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

        return Some(Token::Identifier(TokenValueIdentifier {
            lexeme: self.source[self.start..self.current].to_string(),
            line: self.line,
            identifier: Identifier::from(&self.source[self.start..self.current]),
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
            '(' => return Some(token!(self, LeftParen)),
            ')' => return Some(token!(self, RightParen)),
            '{' => return Some(token!(self, LeftBrace)),
            '}' => return Some(token!(self, RightBrace)),
            ',' => return Some(token!(self, Comma)),
            '.' => return Some(token!(self, Dot)),
            '-' => return Some(token!(self, Minus)),
            '+' => return Some(token!(self, Plus)),
            ';' => return Some(token!(self, Semicolon)),
            '*' => return Some(token!(self, Star)),
            '?' => return Some(token!(self, QuestionMark)),
            ':' => return Some(token!(self, Colon)),

            '!' => {
                if self.matching('=') {
                    return Some(token!(self, BangEqual));
                } else {
                    return Some(token!(self, Bang));
                }
            }
            '=' => {
                if self.matching('=') {
                    return Some(token!(self, EqualEqual));
                } else {
                    return Some(token!(self, Equal));
                }
            }
            '<' => {
                if self.matching('=') {
                    return Some(token!(self, LessEqual));
                } else {
                    return Some(token!(self, Less));
                }
            }
            '>' => {
                if self.matching('=') {
                    return Some(token!(self, GreaterEqual));
                } else {
                    return Some(token!(self, Greater));
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
                    return Some(token!(self, Slash));
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

#[derive(Debug, Clone)]
struct TokenValue {
    lexeme: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct TokenValueString {
    lexeme: String,
    line: usize,
    value: LoxString,
}

#[derive(Debug, Clone)]
struct TokenValueNumber {
    lexeme: String,
    line: usize,
    value: LoxNumber,
}

#[derive(Debug, Clone)]
struct TokenValueIdentifier {
    lexeme: String,
    line: usize,
    identifier: Identifier,
}

#[derive(Debug, Clone)]
struct TokenValueEof {
    line: usize,
}

#[derive(Debug, Clone)]
enum Identifier {
    And,
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

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        match s {
            "and" => Identifier::And,
            "class" => Identifier::Class,
            "else" => Identifier::Else,
            "false" => Identifier::False,
            "fun" => Identifier::Fun,
            "for" => Identifier::For,
            "if" => Identifier::If,
            "nil" => Identifier::Nil,
            "or" => Identifier::Or,
            "print" => Identifier::Print,
            "return" => Identifier::Return,
            "super" => Identifier::Super,
            "this" => Identifier::This,
            "true" => Identifier::True,
            "var" => Identifier::Var,
            "while" => Identifier::While,
            _ => Identifier::Identifier(s.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
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
    Identifier(TokenValueIdentifier),
    String(TokenValueString),
    Number(TokenValueNumber),

    Eof(TokenValueEof),
}

impl Token {
    fn line(&self) -> usize {
        match self {
            Token::Identifier(t) => t.line,
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
        }
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
            Token::Identifier(tv) => write!(f, "Identifier '{}'", tv.lexeme),
            Token::String(tv) => write!(f, "String \"{}\"", tv.value),
            Token::Number(tv) => write!(f, "Number {}", tv.value),

            // EOF
            Token::Eof(_) => write!(f, "EOF"),
        }
    }
}

#[derive(Debug)]
struct AssignExpr {
    name: String,
    value: Box<Expr>,
}

#[derive(Debug)]
struct BinaryExpr {
    left: Box<Expr>,
    operator: Token,
    right: Box<Expr>,
}

#[derive(Debug)]
struct CallExpr {
    callee: Box<Expr>,
    paren: Token,
    arguments: Vec<Expr>,
}

#[derive(Debug)]
struct GetExpr {
    object: Box<Expr>,
    name: Token,
}

#[derive(Debug)]
struct GroupingExpr {
    expression: Box<Expr>,
}

#[derive(Debug)]
enum LiteralExprType {
    Identifier(Identifier),
    String(LoxString),
    Number(LoxNumber),
    EOF,
}

#[derive(Debug)]
struct LiteralExpr {
    value: LiteralExprType,
}

#[derive(Debug)]
struct LogicalExpr {
    left: Box<Expr>,
    operator: Token,
    right: Box<Expr>,
}

#[derive(Debug)]
struct SetExpr {
    object: Box<Expr>,
    name: Token,
    value: Box<Expr>,
}

#[derive(Debug)]
struct SuperExpr {
    keyword: Token,
    method: Token,
}

#[derive(Debug)]
struct TernaryExpr {
    condition: Box<Expr>,
    trueish: Box<Expr>,
    falseish: Box<Expr>,
}

#[derive(Debug)]
struct ThisExpr {
    keyword: Token,
}

#[derive(Debug)]
struct UnaryExpr {
    operator: Token,
    right: Box<Expr>,
}

#[derive(Debug)]
struct VariableExpr {
    name: String,
}

#[derive(Debug)]
enum Expr {
    Assign(AssignExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Get(GetExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Logical(LogicalExpr),
    Set(SetExpr),
    Super(SuperExpr),
    Ternary(TernaryExpr),
    This(ThisExpr),
    Unary(UnaryExpr),
    Variable(VariableExpr),
}

impl Expr {
    fn eval(&self) -> LoxType {
        match self {
            Expr::Assign(assign_expr) => {
                let value = assign_expr.value.eval();
                let mut env = global_env().lock().unwrap();

                env.assign(assign_expr.name.clone(), value.clone());

                return value;
            }
            Expr::Binary(binary_expr) => {
                let left = binary_expr.left.eval();
                let right = binary_expr.right.eval();

                match &binary_expr.operator {
                    Token::Greater(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Boolean(ln > rn),
                        _ => parse_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::GreaterEqual(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Boolean(ln >= rn),
                        _ => parse_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::Less(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Boolean(ln < rn),
                        _ => parse_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::LessEqual(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Boolean(ln <= rn),
                        _ => parse_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::BangEqual(_) => LoxType::Boolean(left != right),
                    Token::EqualEqual(_) => LoxType::Boolean(left == right),
                    Token::Minus(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Number(ln - rn),
                        _ => parse_error!(
                            "[line {}] Error: Cannot subtract NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::Plus(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Number(ln + rn),
                        (LoxType::String(ls), LoxType::String(rs)) => LoxType::String(ls + &rs),
                        (LoxType::String(ls), LoxType::Number(rn)) => {
                            LoxType::String(ls + &rn.to_string())
                        }
                        (LoxType::Number(ln), LoxType::String(rs)) => {
                            LoxType::String(ln.to_string() + &rs)
                        }
                        _ => parse_error!(
                            "[line {}] Error: Incompatible addition types",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::Slash(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Number(ln / rn),
                        _ => parse_error!(
                            "[line {}] Error: Cannot divide NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::Star(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Number(ln * rn),
                        _ => parse_error!(
                            "[line {}] Error: Cannot multiply NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Call(call_expr) => LoxType::Unknown,
            Expr::Get(get_expr) => LoxType::Unknown,
            Expr::Grouping(grouping_expr) => grouping_expr.expression.eval(),
            Expr::Literal(literal_expr) => match &literal_expr.value {
                LiteralExprType::Identifier(id) => match id {
                    Identifier::True => LoxType::Boolean(true),
                    Identifier::False => LoxType::Boolean(false),
                    Identifier::Nil => LoxType::Nil,
                    _ => LoxType::Unknown,
                },
                LiteralExprType::Number(num) => LoxType::Number(*num),
                LiteralExprType::String(str) => LoxType::String(str.clone()),
                LiteralExprType::EOF => LoxType::Unknown,
            },
            Expr::Logical(logical_expr) => LoxType::Unknown,
            Expr::Set(set_expr) => LoxType::Unknown,
            Expr::Super(super_expr) => LoxType::Unknown,
            Expr::Ternary(ternary_expr) => {
                let condition = ternary_expr.condition.eval();
                let trueish = ternary_expr.trueish.eval();
                let falseish = ternary_expr.falseish.eval();

                if condition.is_truthy() {
                    return trueish;
                } else {
                    return falseish;
                }
            }
            Expr::This(this_expr) => LoxType::Unknown,
            Expr::Unary(unary_expr) => {
                let right = unary_expr.right.eval();

                match &unary_expr.operator {
                    Token::Bang(_) => LoxType::Boolean(!right.is_truthy()),
                    Token::Minus(_) => match right {
                        LoxType::Number(n) => LoxType::Number(-n),
                        _ => {
                            parse_error!("[line {}] Error: Cannot negate NaNs", unary_expr.operator)
                        }
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Variable(variable_expr) => {
                let env = global_env().lock().unwrap();
                return env.get(&variable_expr.name).clone();
            }
        }
    }
}

#[derive(Debug)]
struct VarStatement {
    name: String,
    initializer: Option<Expr>,
}

#[derive(Debug)]
enum Statement {
    Expression(Expr),
    Print(Expr),
    Var(VarStatement),
    Block(Vec<Statement>),
}

impl Statement {
    fn eval(&self) {
        match self {
            Statement::Expression(expr) => {
                let _value = expr.eval();
            }
            Statement::Print(expr) => {
                let value = expr.eval();
                println!("{}", value);
            }
            Statement::Var(vs) => {
                let mut value = LoxType::Nil;

                if let Some(expr) = &vs.initializer {
                    value = expr.eval();
                }

                let mut env = global_env().lock().unwrap();
                env.define(vs.name.clone(), value);
            }
            Statement::Block(block) => {
                let enclosed = {
                    let mut guard = global_env().lock().unwrap();
                    let mut enclosed = Environment {
                        enclosing: Some(Box::new(guard.clone())),
                        values: HashMap::new(),
                    };

                    std::mem::swap(&mut *guard, &mut enclosed);

                    enclosed
                };

                block.iter().for_each(|s| s.eval());

                {
                    let mut guard = global_env().lock().unwrap();
                    let mut tmp = enclosed;

                    std::mem::swap(&mut *guard, &mut tmp);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, LoxType>,
}

impl Environment {
    fn define(&mut self, name: String, value: LoxType) {
        self.values.insert(name, value);
    }

    fn get(&self, name: &String) -> &LoxType {
        if let Some(value) = self.values.get(name) {
            return value;
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }

        parse_error!("Undefined variable '{}'.", name);
    }

    fn assign(&mut self, name: String, value: LoxType) {
        if let Some(_) = self.values.get(&name) {
            self.values.insert(name.clone(), value);
            return;
        }

        if let Some(ref mut enclosing) = self.enclosing {
            enclosing.assign(name.clone(), value);
            return;
        }

        parse_error!("Undefined variable '{}'", name);
    }
}

static GLOBAL_ENV: OnceLock<Mutex<Environment>> = OnceLock::new();

fn global_env() -> &'static Mutex<Environment> {
    GLOBAL_ENV.get_or_init(|| {
        Mutex::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        })
    })
}

struct Interpreter {}

impl Interpreter {
    fn new() -> Self {
        Self {}
    }

    fn interpret(&mut self, statements: Vec<Statement>) {
        statements.iter().for_each(|s| s.eval());
    }
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn previous(&self) -> Token {
        return self.tokens[self.current - 1].clone();
    }

    fn expression(&mut self) -> Expr {
        return self.assignment();
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.ternary();

        if let Some(token) = self.tokens.get(self.current).cloned() {
            match token {
                Token::Equal(_) => {
                    self.current += 1;

                    let value = self.assignment();

                    match expr {
                        Expr::Variable(v) => {
                            return Expr::Assign(AssignExpr {
                                name: v.name,
                                value: Box::new(value),
                            });
                        }
                        _ => parse_error!(
                            "[line {}] Error: Invalid assignment target.",
                            token.line()
                        ),
                    }
                }
                _ => {}
            }
        }

        return expr;
    }

    fn ternary(&mut self) -> Expr {
        let mut expr = self.equality();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::QuestionMark(_) => {
                        self.current += 1;

                        let trueish = self.ternary();

                        if let Some(token_n) = self.tokens.get(self.current) {
                            match token_n {
                                Token::Colon(_) => {
                                    self.current += 1;
                                }
                                _ => parse_error!(
                                    "[line {}] Error: Missing ':' in ternary expression.",
                                    token_n.line()
                                ),
                            }
                        }

                        let falseish = self.ternary();

                        expr = Expr::Ternary(TernaryExpr {
                            condition: Box::new(expr),
                            trueish: Box::new(trueish),
                            falseish: Box::new(falseish),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::BangEqual(_) | Token::EqualEqual(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.comparison();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::Greater(_)
                    | Token::GreaterEqual(_)
                    | Token::Less(_)
                    | Token::LessEqual(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.term();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::Minus(_) | Token::Plus(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.factor();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::Slash(_) | Token::Star(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.unary();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn unary(&mut self) -> Expr {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Bang(_) | Token::Minus(_) => {
                    self.current += 1;

                    let operator = self.previous();
                    let right = self.unary();
                    return Expr::Unary(UnaryExpr {
                        operator,
                        right: Box::new(right),
                    });
                }
                _ => {}
            }
        }

        return self.primary();
    }

    fn primary(&mut self) -> Expr {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Identifier(id) => match &id.identifier {
                    Identifier::False | Identifier::True | Identifier::Nil => {
                        self.current += 1;

                        return Expr::Literal(LiteralExpr {
                            value: LiteralExprType::Identifier(id.identifier.clone()),
                        });
                    }
                    Identifier::Identifier(name) => {
                        self.current += 1;

                        return Expr::Variable(VariableExpr { name: name.clone() });
                    }
                    _ => {
                        parse_error!(
                            "[line {}] Error: Unexpected identifier '{:?}' encountered.",
                            id.line,
                            id.identifier
                        );
                    }
                },
                Token::Number(num) => {
                    self.current += 1;

                    return Expr::Literal(LiteralExpr {
                        value: LiteralExprType::Number(num.value),
                    });
                }
                Token::String(str) => {
                    self.current += 1;

                    return Expr::Literal(LiteralExpr {
                        value: LiteralExprType::String(str.value.clone()),
                    });
                }
                Token::LeftParen(_) => {
                    self.current += 1;

                    let expr = self.expression();

                    if let Some(token_n) = self.tokens.get(self.current) {
                        match token_n {
                            Token::RightParen(_) => {
                                self.current += 1;
                            }
                            _ => parse_error!("[line {}] Error: Missing ')'.", token_n.line()),
                        }
                    }

                    return Expr::Grouping(GroupingExpr {
                        expression: Box::new(expr),
                    });
                }
                Token::Eof(_) => {
                    return Expr::Literal(LiteralExpr {
                        value: LiteralExprType::EOF,
                    });
                }
                _ => {
                    parse_error!("Unexpected token {} encountered.", token);
                }
            }
        }

        parse_error!("Empty file cannot be parsed.");
    }

    fn declaration(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Identifier(id) => match id.identifier {
                    Identifier::Var => {
                        self.current += 1;
                        return self.var_declaration();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        return self.statement();
    }

    fn var_declaration(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current).cloned() {
            match token {
                Token::Identifier(ref id) => match &id.identifier {
                    Identifier::Identifier(name) => {
                        self.current += 1;

                        let initializer =
                            if let Some(Token::Equal(_)) = self.tokens.get(self.current) {
                                self.current += 1;

                                Some(self.expression())
                            } else {
                                None
                            };

                        if let Some(token_n) = self.tokens.get(self.current) {
                            match token_n {
                                Token::Semicolon(_) => {
                                    self.current += 1;
                                }
                                _ => parse_error!("[line {}] Error: Missing ';'.", token_n.line()),
                            }
                        }

                        return Statement::Var(VarStatement {
                            name: name.clone(),
                            initializer,
                        });
                    }
                    _ => parse_error!(
                        "[line {}] Error: The name of your variable cannot be a keyword.",
                        token.line()
                    ),
                },
                _ => parse_error!(
                    "[line {}] Error: Provide a name for your variable.",
                    token.line()
                ),
            }
        }

        parse_error!(
            "[line {}] Error: Provide a name for your variable.",
            self.tokens.last().unwrap().line()
        );
    }

    fn statement(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Identifier(id) => match id.identifier {
                    Identifier::Print => {
                        self.current += 1;
                        return self.print_statement();
                    }
                    _ => {}
                },
                Token::LeftBrace(_) => {
                    self.current += 1;

                    return Statement::Block(self.block());
                }
                _ => {}
            }
        }

        return self.expression_statement();
    }

    fn block(&mut self) -> Vec<Statement> {
        let mut statements = vec![];

        while let Some(token) = self.tokens.get(self.current) {
            if let Token::RightBrace(_) = token {
                break;
            }

            statements.push(self.declaration());
        }

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::RightBrace(_) => {
                    self.current += 1;
                }
                _ => parse_error!("[line {}] Error: Missing '}}'.", token.line()),
            }
        }

        return statements;
    }

    fn print_statement(&mut self) -> Statement {
        let value = self.expression();

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Semicolon(_) => {
                    self.current += 1;
                }
                _ => parse_error!("[line {}] Error: Missing ';'.", token.line()),
            }
        }

        return Statement::Print(value);
    }

    fn expression_statement(&mut self) -> Statement {
        let expr = self.expression();

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Semicolon(_) => {
                    self.current += 1;
                }
                Token::Eof(_) => {
                    self.current += 1;
                }
                _ => parse_error!("[line {}] Error: Missing ';'.", token.line()),
            }
        }

        return Statement::Expression(expr);
    }

    fn parse(&mut self) -> Vec<Statement> {
        let mut statements = vec![];

        while self.current < self.tokens.len() {
            statements.push(self.declaration());
        }

        return statements;
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();

    if args.len() == 1 {
        if let Err(err) = run_prompt() {
            eprintln!("{}", err);
            exit(1);
        }
    } else if args.len() == 2 {
        if let Err(err) = run_file(&args.nth(1).unwrap()) {
            eprintln!("{}", err);
            exit(1);
        }
    } else {
        println!("Usage: lox [file]");
        exit(1);
    }

    Ok(())
}

fn error(line: usize, message: &String) -> CompileError {
    CompileError::new(line, "".to_string(), String::from(message))
}

fn run_prompt() -> Result<()> {
    let mut input;

    loop {
        print!("[lox] > ");

        input = String::new();

        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;

        input = input.trim_end().to_string();

        if input.is_empty() {
            break;
        }

        run(&input)?;
    }

    Ok(())
}

fn run_file(path: &String) -> Result<()> {
    let file_string = fs::read_to_string(path)?;
    run(&file_string)?;

    Ok(())
}

fn run(source: &String) -> Result<()> {
    let scanner = Scanner::new(source);

    let errs_rc = scanner.errors.clone();
    let tokens = scanner.collect::<Vec<Token>>();

    match Rc::try_unwrap(errs_rc) {
        Ok(cell) => {
            let errs = cell.into_inner();
            if !errs.is_empty() {
                return Err(Box::new(CompileErrors(errs)));
            }
        }
        Err(_) => {}
    }

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(statements);

    return Ok(());
}
