use crate::{
    environment::global_env,
    lox_error,
    lox_type::{LoxCallable, LoxNumber, LoxString, LoxType},
    token::{Keyword, Token},
};

#[derive(Clone)]
pub struct AssignExpr {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Clone)]
pub struct GetExpr {
    pub object: Box<Expr>,
    pub name: Token,
}

#[derive(Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Clone)]
pub enum LiteralExprType {
    Identifier(Keyword),
    String(LoxString),
    Number(LoxNumber),
    EOF,
}

#[derive(Clone)]
pub struct LiteralExpr {
    pub value: LiteralExprType,
}

#[derive(Clone)]
pub struct LogicalExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone)]
pub struct SetExpr {
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Clone)]
pub struct SuperExpr {
    pub keyword: Token,
    pub method: Token,
}

#[derive(Clone)]
pub struct TernaryExpr {
    pub condition: Box<Expr>,
    pub trueish: Box<Expr>,
    pub falseish: Box<Expr>,
}

#[derive(Clone)]
pub struct ThisExpr {
    pub keyword: Token,
}

#[derive(Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone)]
pub struct VariableExpr {
    pub name: String,
}

#[derive(Clone)]
pub enum Expr {
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
    pub fn eval(&self) -> LoxType {
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
                        _ => lox_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::GreaterEqual(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Boolean(ln >= rn),
                        _ => lox_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::Less(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Boolean(ln < rn),
                        _ => lox_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::LessEqual(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Boolean(ln <= rn),
                        _ => lox_error!(
                            "[line {}] Error: Cannot compare NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::BangEqual(_) => LoxType::Boolean(left != right),
                    Token::EqualEqual(_) => LoxType::Boolean(left == right),
                    Token::Minus(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Number(ln - rn),
                        _ => lox_error!(
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
                        _ => lox_error!(
                            "[line {}] Error: Incompatible addition types",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::Slash(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Number(ln / rn),
                        _ => lox_error!(
                            "[line {}] Error: Cannot divide NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    Token::Star(_) => match (left, right) {
                        (LoxType::Number(ln), LoxType::Number(rn)) => LoxType::Number(ln * rn),
                        _ => lox_error!(
                            "[line {}] Error: Cannot multiply NaNs",
                            binary_expr.operator.line()
                        ),
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Call(call_expr) => {
                let callee = call_expr.callee.eval();

                let args = call_expr
                    .arguments
                    .iter()
                    .map(|carg| carg.eval())
                    .collect::<Vec<LoxType>>();

                match callee {
                    LoxType::Function(fun) => {
                        if args.len() != fun.arity() {
                            lox_error!(
                                "[line {}] Error: Expected {} arguments but got {}.",
                                call_expr.paren.line(),
                                fun.arity(),
                                args.len()
                            );
                        }

                        return fun.call((args, call_expr.paren.line()));
                    }
                    _ => lox_error!(
                        "[line {}] Error: Can only call functions and classes.",
                        call_expr.paren.line()
                    ),
                }
            }
            Expr::Get(get_expr) => LoxType::Unknown,
            Expr::Grouping(grouping_expr) => grouping_expr.expression.eval(),
            Expr::Literal(literal_expr) => match &literal_expr.value {
                LiteralExprType::Identifier(id) => match id {
                    Keyword::True => LoxType::Boolean(true),
                    Keyword::False => LoxType::Boolean(false),
                    Keyword::Nil => LoxType::Nil,
                    _ => LoxType::Unknown,
                },
                LiteralExprType::Number(num) => LoxType::Number(*num),
                LiteralExprType::String(str) => LoxType::String(str.clone()),
                LiteralExprType::EOF => LoxType::Unknown,
            },
            Expr::Logical(logical_expr) => {
                let left = logical_expr.left.eval();

                match &logical_expr.operator {
                    Token::Keyword(k) => match k.keyword {
                        Keyword::Or => {
                            if left.is_truthy() {
                                return left;
                            }
                        }
                        _ => {
                            if !left.is_truthy() {
                                return left;
                            }
                        }
                    },
                    _ => {
                        if !left.is_truthy() {
                            return left;
                        }
                    }
                }

                return logical_expr.right.eval();
            }
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
                            lox_error!("[line {}] Error: Cannot negate NaNs", unary_expr.operator)
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
