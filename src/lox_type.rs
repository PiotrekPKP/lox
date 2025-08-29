use std::fmt::Debug;

use crate::{
    lox_error,
    statement::{Statement, StatementSignal},
};

pub type LoxString = String;
pub type LoxNumber = f64;
pub type LoxBoolean = bool;

#[derive(Clone)]
pub struct LoxFunction {
    pub arity: usize,
    pub body: Statement,
}

impl Debug for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoxFunction")
            .field("arity", &self.arity)
            .field("body", &self.body)
            .finish()
    }
}

#[derive(Clone)]
pub enum LoxType {
    String(LoxString),
    Number(LoxNumber),
    Boolean(LoxBoolean),
    Nil,
    Unknown,
    Function(LoxFunction),
}

impl LoxType {
    pub fn is_truthy(&self) -> bool {
        match self {
            LoxType::Boolean(b) => *b,
            LoxType::Nil => false,
            LoxType::Number(n) => *n != 0.,
            _ => true,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxType::Function(lf) => lf.arity,
            _ => 0,
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
            LoxType::Function(lf) => write!(f, "<lox fn>({})", lf.arity),
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

pub type LoxFunctionArgs = Vec<LoxType>;
pub type LoxCallableArgs = (LoxFunctionArgs, usize);

pub trait LoxCallable: Debug + Clone {
    fn call(&self, args: LoxCallableArgs) -> LoxType;
}

impl LoxCallable for LoxType {
    fn call(&self, (args, line): LoxCallableArgs) -> LoxType {
        match self {
            LoxType::Function(lf) => {
                let res = lf.body.eval();

                if res.is_ok() {
                    return LoxType::Nil;
                }

                return match res.unwrap_err() {
                    StatementSignal::Return(rv) => rv,
                    _ => lox_error!(
                        "[line {}] Error: Function terminated with an unexpected token.",
                        line
                    ),
                };
            }
            _ => lox_error!(
                "[line {}] Error: Can only call functions and classes.",
                line
            ),
        }
    }
}

impl<F: Debug + Clone> LoxCallable for F
where
    F: Fn(LoxFunctionArgs) -> LoxType,
{
    fn call(&self, (args, _): LoxCallableArgs) -> LoxType {
        self(args)
    }
}

impl Debug for LoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Boolean(arg0) => f.debug_tuple("Boolean").field(arg0).finish(),
            Self::Nil => write!(f, "Nil"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Function(arg0) => f.debug_tuple("Function").field(arg0).finish(),
        }
    }
}
