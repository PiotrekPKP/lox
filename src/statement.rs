use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crate::{
    environment::{Environment, global_env},
    expression::Expr,
    lox_type::LoxType,
};

#[derive(Debug, Clone)]
pub struct VarStatement {
    pub name: String,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expr,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expr,
    pub body: Box<Statement>,
    pub in_for_loop: bool,
}

#[derive(Clone)]
pub enum Statement {
    Expression(Expr),
    Print(Expr),
    Var(VarStatement),
    Block(Vec<Statement>),
    If(IfStatement),
    While(WhileStatement),
    Break,
    Continue,
    NativeFn(Arc<dyn Fn() -> LoxType + Send + Sync>),
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expression(arg0) => f.debug_tuple("Expression").field(arg0).finish(),
            Self::Print(arg0) => f.debug_tuple("Print").field(arg0).finish(),
            Self::Var(arg0) => f.debug_tuple("Var").field(arg0).finish(),
            Self::Block(arg0) => f.debug_tuple("Block").field(arg0).finish(),
            Self::If(arg0) => f.debug_tuple("If").field(arg0).finish(),
            Self::While(arg0) => f.debug_tuple("While").field(arg0).finish(),
            Self::Break => write!(f, "Break"),
            Self::Continue => write!(f, "Continue"),
            Self::NativeFn(_) => f.debug_tuple("NativeFn").finish(),
        }
    }
}

#[derive(Debug)]
pub enum StatementSignal {
    Break,
    Continue,
    Return(LoxType),
}

impl Statement {
    pub fn eval(&self) -> Result<(), StatementSignal> {
        return match self {
            Statement::Expression(expr) => {
                let _value = expr.eval();

                Ok(())
            }
            Statement::Print(expr) => {
                let value = expr.eval();
                println!("{}", value);

                Ok(())
            }
            Statement::Var(vs) => {
                let mut value = LoxType::Nil;

                if let Some(expr) = &vs.initializer {
                    value = expr.eval();
                }

                let mut env = global_env().lock().unwrap();
                env.define(vs.name.clone(), value);

                Ok(())
            }
            Statement::Block(block) => {
                let mut guard = global_env().lock().unwrap();
                let prev = std::mem::replace(&mut *guard, Environment::new());
                let new_env = Environment {
                    values: HashMap::new(),
                    enclosing: Some(Box::new(prev)),
                };
                *guard = new_env;
                drop(guard);

                for stmt in block {
                    let res = stmt.eval();

                    if res.is_err() {
                        let mut guard = global_env().lock().unwrap();
                        if let Some(enclosing_box) = guard.enclosing.take() {
                            *guard = *enclosing_box;
                        }

                        return res;
                    }
                }

                let mut guard = global_env().lock().unwrap();
                if let Some(enclosing_box) = guard.enclosing.take() {
                    *guard = *enclosing_box;
                }

                Ok(())
            }
            Statement::If(is) => {
                if is.condition.eval().is_truthy() {
                    is.then_branch.eval()?;
                } else if let Some(else_branch) = &is.else_branch {
                    else_branch.eval()?;
                }

                Ok(())
            }
            Statement::While(ws) => {
                while ws.condition.eval().is_truthy() {
                    let res = ws.body.eval();

                    if let Err(ss) = res {
                        match ss {
                            StatementSignal::Break => break,
                            StatementSignal::Continue => {
                                if ws.in_for_loop {
                                    let Statement::Block(ref loop_block) = *ws.body else {
                                        continue;
                                    };

                                    let _ = loop_block.last().unwrap().eval();
                                }

                                continue;
                            }
                            StatementSignal::Return(_) => return Err(ss),
                        }
                    }
                }

                Ok(())
            }
            Statement::Break => Err(StatementSignal::Break),
            Statement::Continue => Err(StatementSignal::Continue),
            Statement::NativeFn(closure) => {
                let fn_res = closure.call(());
                return Err(StatementSignal::Return(fn_res));
            }
        };
    }
}
