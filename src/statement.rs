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
pub enum Statement {
    Expression(Expr),
    Print(Expr),
    Var(VarStatement),
    Block(Vec<Statement>),
    If(IfStatement),
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
            Statement::If(is) => {
                if is.condition.eval().is_truthy() {
                    let _value = is.then_branch.eval();
                } else if let Some(else_branch) = &is.else_branch {
                    let _value = else_branch.eval();
                }
            }
        }
    }
}
