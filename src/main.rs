mod environment;
mod expression;
mod interpreter;
mod lox_type;
mod parser;
mod scanner;
mod statement;
mod token;

use std::fs;
use std::io;
use std::io::Write;
use std::process::exit;
use std::rc::Rc;
use std::usize;

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token::Token;

#[macro_export]
macro_rules! token_n {
    ($self:expr, $variant:ident) => {
        Token::$variant($self.get_token_value())
    };
}

#[macro_export]
macro_rules! lox_error {
    ($fmt:expr $(, $($arg:tt)+ )? ) => {{
        eprintln!($fmt $(, $($arg)+ )?);
        std::process::exit(1);
    }};
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct CompileErrors(Vec<CompileError>);

#[derive(Debug, Clone)]
pub struct CompileError {
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

pub fn error(line: usize, message: &String) -> CompileError {
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
