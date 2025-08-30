use std::{collections::HashMap, sync::Arc};

use crate::{
    environment::Environment,
    expression::Expr,
    lox_type::{LoxFunction, LoxType},
    token::Token,
};

#[derive(Clone)]
pub struct FunctionStatement {
    pub name: String,
    pub params: Vec<Token>,
    pub body: Box<Statement>,
}

#[derive(Clone)]
pub struct VarStatement {
    pub name: String,
    pub initializer: Option<Expr>,
}

#[derive(Clone)]
pub struct IfStatement {
    pub condition: Expr,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

#[derive(Clone)]
pub struct WhileStatement {
    pub condition: Expr,
    pub body: Box<Statement>,
    pub in_for_loop: bool,
}

#[derive(Clone)]
pub struct ReturnStatement {
    pub keyword: Token,
    pub value: Option<Expr>,
}

#[derive(Clone)]
pub enum Statement {
    Expression(Expr),
    Print(Expr),
    Var(VarStatement),
    Block(Vec<Statement>),
    If(IfStatement),
    While(WhileStatement),
    Function(FunctionStatement),
    Break,
    Continue,
    Return(ReturnStatement),
}

pub enum StatementSignal {
    Break,
    Continue,
    Return(Option<LoxType>),
}

impl Statement {
    pub fn eval(&self, env: &mut Environment) -> Result<(), StatementSignal> {
        return match self {
            Statement::Expression(expr) => {
                let _value = expr.eval(env);

                Ok(())
            }
            Statement::Print(expr) => {
                let value = expr.eval(env);
                println!("{}", value);

                Ok(())
            }
            Statement::Function(fs) => {
                let lox_fn = LoxType::Function(Arc::new(LoxFunction {
                    params: fs.params.clone(),
                    body: *fs.body.clone(),
                }));

                env.define(fs.name.clone(), lox_fn);

                Ok(())
            }
            Statement::Var(vs) => {
                let mut value = LoxType::Nil;

                if let Some(expr) = &vs.initializer {
                    value = expr.eval(env);
                }

                env.define(vs.name.clone(), value);

                Ok(())
            }
            Statement::Block(block) => {
                let mut block_env = Environment::new(Some(env.clone()), HashMap::new());

                for stmt in block {
                    let res = stmt.eval(&mut block_env);

                    if res.is_err() {
                        env.reset(&block_env.enclosing.unwrap());
                        return res;
                    }
                }

                env.reset(&block_env.enclosing.unwrap());
                Ok(())
            }
            Statement::If(is) => {
                if is.condition.eval(env).is_truthy() {
                    is.then_branch.eval(env)?;
                } else if let Some(else_branch) = &is.else_branch {
                    else_branch.eval(env)?;
                }

                Ok(())
            }
            Statement::While(ws) => {
                while ws.condition.eval(env).is_truthy() {
                    let res = ws.body.eval(env);

                    if let Err(ss) = res {
                        match ss {
                            StatementSignal::Break => break,
                            StatementSignal::Continue => {
                                if ws.in_for_loop {
                                    let Statement::Block(ref loop_block) = *ws.body else {
                                        continue;
                                    };

                                    let _ = loop_block.last().unwrap().eval(env);
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
            Statement::Return(rs) => Err(StatementSignal::Return(
                rs.value.as_ref().map(|r| r.eval(env)),
            )),
        };
    }
}
