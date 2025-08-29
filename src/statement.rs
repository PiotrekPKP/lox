use std::collections::HashMap;

use crate::{
    environment::{Environment, global_env},
    expression::Expr,
    lox_type::LoxType,
};

#[derive(Debug)]
pub struct VarStatement {
    pub name: String,
    pub initializer: Option<Expr>,
}

#[derive(Debug)]
pub struct IfStatement {
    pub condition: Expr,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

#[derive(Debug)]
pub struct WhileStatement {
    pub condition: Expr,
    pub body: Box<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expr),
    Print(Expr),
    Var(VarStatement),
    Block(Vec<Statement>),
    If(IfStatement),
    While(WhileStatement),
}

impl Statement {
    pub fn eval(&self) {
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
                {
                    let mut guard = global_env().lock().unwrap();

                    let prev = std::mem::replace(
                        &mut *guard,
                        Environment {
                            enclosing: None,
                            values: HashMap::new(),
                        },
                    );

                    let new_env = Environment {
                        values: HashMap::new(),
                        enclosing: Some(Box::new(prev)),
                    };

                    *guard = new_env;
                }

                block.iter().for_each(|s| s.eval());

                {
                    let mut guard = global_env().lock().unwrap();
                    if let Some(enclosing_box) = guard.enclosing.take() {
                        *guard = *enclosing_box;
                    }
                }
            }
            Statement::If(is) => {
                if is.condition.eval().is_truthy() {
                    let _value = is.then_branch.eval();
                } else if let Some(else_branch) = &is.else_branch {
                    let _value = else_branch.eval();
                }
            }
            Statement::While(ws) => {
                while ws.condition.eval().is_truthy() {
                    let _value = ws.body.eval();
                }
            }
        }
    }
}
