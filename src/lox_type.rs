use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::{
    environment::Environment,
    lox_error,
    statement::{Statement, StatementSignal},
    token::{Keyword, Token},
};

pub type LoxString = String;
pub type LoxNumber = f64;
pub type LoxBoolean = bool;

#[derive(Clone)]
pub struct LoxFunction {
    pub name: String,
    pub params: Vec<Token>,
    pub body: Statement,
    pub closure: Environment,
}

#[derive(Clone)]
pub struct LoxNativeFunction {
    pub arity: usize,
    pub body: Arc<dyn Fn(LoxFunctionArgs) -> LoxType + Send + Sync>,
}

#[derive(Clone)]
pub enum LoxType {
    String(LoxString),
    Number(LoxNumber),
    Boolean(LoxBoolean),
    Nil,
    Unknown,
    Function(Arc<Mutex<dyn LoxCallable>>),
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
}

impl std::fmt::Display for LoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Boolean(b) => write!(f, "{b}"),
            LoxType::Nil => write!(f, "nil"),
            LoxType::Number(n) => write!(f, "{n}"),
            LoxType::String(s) => write!(f, "{s}"),
            LoxType::Unknown => write!(f, "\0"),
            LoxType::Function(lf) => write!(f, "<lox fn>({})", lf.lock().unwrap().arity()),
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
pub type LoxCallableArgs<'a> = (LoxFunctionArgs, &'a mut Environment, usize);

pub trait LoxCallable: Send + Sync + Any {
    fn call(&mut self, args: LoxCallableArgs) -> LoxType;

    fn arity(&self) -> usize;
}

impl LoxCallable for LoxFunction {
    fn call(&mut self, (args, env, line): LoxCallableArgs) -> LoxType {
        let mut closure = self.closure.clone();
        closure.define(
            self.name.clone(),
            LoxType::Function(Arc::new(Mutex::new(self.clone()))),
        );
        let mut call_env = Environment::new(Some(closure), env.values.clone());

        self.params
            .iter()
            .enumerate()
            .for_each(|(i, param)| match param {
                Token::Keyword(k) => match &k.keyword {
                    Keyword::Identifier(param_name) => {
                        call_env.define(param_name.clone(), args[i].clone())
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            });

        let res = self.body.eval(&mut call_env);
        self.closure.reset(&call_env.enclosing.unwrap());

        if res.is_ok() {
            return LoxType::Nil;
        }

        match res.unwrap_err() {
            StatementSignal::Return(rv) => rv.unwrap_or(LoxType::Nil),
            _ => lox_error!(
                "[line {}] Error: Function terminated with an unexpected token.",
                line
            ),
        }
    }

    fn arity(&self) -> usize {
        return self.params.len();
    }
}

impl LoxCallable for LoxNativeFunction {
    fn call(&mut self, (args, _env, _line): LoxCallableArgs) -> LoxType {
        (self.body)(args)
    }

    fn arity(&self) -> usize {
        return self.arity;
    }
}

impl dyn LoxCallable {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
