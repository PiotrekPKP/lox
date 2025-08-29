use crate::{
    expression::{
        AssignExpr, BinaryExpr, Expr, GroupingExpr, LiteralExpr, LiteralExprType, LogicalExpr,
        TernaryExpr, UnaryExpr, VariableExpr,
    },
    lox_error,
    statement::{IfStatement, Statement, VarStatement, WhileStatement},
    token::{Keyword, Token},
};

macro_rules! consume {
    ($self:ident, $($token_type:ident)|+, $msg:expr) => {
        if let Some(token) = $self.tokens.get($self.current) {
            match token {
                $(Token::$token_type(_))|+ => {
                    $self.current += 1;
                },
                _ => lox_error!(concat!("[line {}] ", $msg), token.line()),
            }
        }
    };
}

macro_rules! match_token {
    ($self:ident, $($token_type:ident)|+) => {{
        if let Some(token) = $self.tokens.get($self.current) {
            if matches!(token, $(Token::$token_type(_))|+) {
                $self.current += 1;
                Some(token)
            } else {
                None
            }
        } else {
            None
        }
    }};

    ($self:ident, $token_type:ident, $($inner_enum:ident)|+) => {{
        if let Some(token) = $self.tokens.get($self.current) {
            match token {
                Token::$token_type(inner) => {
                    let matched = matches!(inner.keyword, $(Keyword::$inner_enum)|+);
                    if matched {
                        $self.current += 1;
                        Some(token)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }};
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn previous(&self) -> Token {
        return self.tokens[self.current - 1].clone();
    }

    fn expression(&mut self) -> Expr {
        return self.assignment();
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if let Some(token) = match_token!(self, Equal).cloned() {
            let value = self.assignment();

            match expr {
                Expr::Variable(v) => {
                    return Expr::Assign(AssignExpr {
                        name: v.name,
                        value: Box::new(value),
                    });
                }
                _ => {
                    lox_error!("[line {}] Error: Invalid assignment target.", token.line())
                }
            }
        }

        return expr;
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while let Some(_) = match_token!(self, Keyword, Or) {
            let operator = self.previous();
            let right = self.and();

            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.ternary();

        while let Some(_) = match_token!(self, Keyword, And) {
            let operator = self.previous();
            let right = self.ternary();

            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn ternary(&mut self) -> Expr {
        let mut expr = self.equality();

        while let Some(_) = match_token!(self, QuestionMark) {
            let trueish = self.ternary();
            consume!(self, Colon, "Error: Missing ':' in ternary expression.");
            let falseish = self.ternary();

            expr = Expr::Ternary(TernaryExpr {
                condition: Box::new(expr),
                trueish: Box::new(trueish),
                falseish: Box::new(falseish),
            })
        }

        return expr;
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while let Some(_) = match_token!(self, BangEqual | EqualEqual) {
            let operator = self.previous();
            let right = self.comparison();

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while let Some(_) = match_token!(self, Greater | GreaterEqual | Less | LessEqual) {
            let operator = self.previous();
            let right = self.term();

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while let Some(_) = match_token!(self, Minus | Plus) {
            let operator = self.previous();
            let right = self.factor();

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while let Some(_) = match_token!(self, Slash | Star) {
            let operator = self.previous();
            let right = self.unary();

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn unary(&mut self) -> Expr {
        if let Some(_) = match_token!(self, Bang | Minus) {
            let operator = self.previous();
            let right = self.unary();

            return Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            });
        }

        return self.primary();
    }

    fn primary(&mut self) -> Expr {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Keyword(id) => match &id.keyword {
                    Keyword::False | Keyword::True | Keyword::Nil => {
                        self.current += 1;

                        return Expr::Literal(LiteralExpr {
                            value: LiteralExprType::Identifier(id.keyword.clone()),
                        });
                    }
                    Keyword::Identifier(name) => {
                        self.current += 1;

                        return Expr::Variable(VariableExpr { name: name.clone() });
                    }
                    _ => {
                        lox_error!(
                            "[line {}] Error: Unexpected identifier '{:?}' encountered.",
                            id.line,
                            id.keyword
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

                    consume!(self, RightParen, "Error: Missing ')'.");

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
                    lox_error!("Unexpected token {} encountered.", token);
                }
            }
        }

        lox_error!("Empty file cannot be parsed.");
    }

    fn declaration(&mut self) -> Statement {
        if let Some(_) = match_token!(self, Keyword, Var) {
            return self.var_declaration();
        }

        return self.statement();
    }

    fn var_declaration(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current).cloned() {
            match token {
                Token::Keyword(ref k) => match &k.keyword {
                    Keyword::Identifier(name) => {
                        self.current += 1;

                        let initializer =
                            if let Some(Token::Equal(_)) = self.tokens.get(self.current) {
                                self.current += 1;

                                Some(self.expression())
                            } else {
                                None
                            };

                        consume!(self, Semicolon, "Error: Missing ';'.");

                        return Statement::Var(VarStatement {
                            name: name.clone(),
                            initializer,
                        });
                    }
                    _ => lox_error!(
                        "[line {}] Error: The name of your variable cannot be a keyword.",
                        token.line()
                    ),
                },
                _ => lox_error!(
                    "[line {}] Error: Provide a name for your variable.",
                    token.line()
                ),
            }
        }

        lox_error!(
            "[line {}] Error: Provide a name for your variable.",
            self.tokens.last().unwrap().line()
        );
    }

    fn statement(&mut self) -> Statement {
        if let Some(_) = match_token!(self, Keyword, Print) {
            return self.print_statement();
        }

        if let Some(_) = match_token!(self, Keyword, While) {
            return self.while_statement();
        }

        if let Some(_) = match_token!(self, Keyword, If) {
            return self.if_statement();
        }

        if let Some(_) = match_token!(self, Keyword, Break) {
            return self.break_statement();
        }

        if let Some(_) = match_token!(self, Keyword, Continue) {
            return self.continue_statement();
        }
        if let Some(_) = match_token!(self, LeftBrace) {
            return Statement::Block(self.block());
        }

        return self.expression_statement();
    }

    fn block(&mut self) -> Vec<Statement> {
        let mut statements = vec![];

        while let Some(token) = self.tokens.get(self.current) {
            if let Token::RightBrace(_) | Token::Eof(_) = token {
                break;
            }

            statements.push(self.declaration());
        }

        consume!(self, RightBrace, "Error: Missing '}}'.");

        return statements;
    }

    fn while_statement(&mut self) -> Statement {
        consume!(self, LeftParen, "Error: Expected '(' after 'while'.");

        let condition = self.expression();

        consume!(self, RightParen, "Error: Expected ')' after condition.");

        let body = self.statement();

        return Statement::While(WhileStatement {
            body: Box::new(body),
            condition,
        });
    }

    fn if_statement(&mut self) -> Statement {
        consume!(self, LeftParen, "Error: Expected '(' after 'if'.");

        let condition = self.expression();

        consume!(self, RightParen, "Error: Expected ')' after if condition.");

        let then_branch = self.statement();
        let else_branch = if let Some(_) = match_token!(self, Keyword, Else) {
            Some(self.statement())
        } else {
            None
        };

        return Statement::If(IfStatement {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        });
    }

    fn print_statement(&mut self) -> Statement {
        let value = self.expression();

        consume!(self, Semicolon, "Error: Missing ';'.");

        return Statement::Print(value);
    }

    fn break_statement(&mut self) -> Statement {
        consume!(self, Semicolon, "Error: Missing ';'.");

        return Statement::Break;
    }

    fn continue_statement(&mut self) -> Statement {
        consume!(self, Semicolon, "Error: Missing ';'.");

        return Statement::Continue;
    }

    fn expression_statement(&mut self) -> Statement {
        let expr = self.expression();

        consume!(self, Semicolon | Eof, "Error: Missing ';'.");

        return Statement::Expression(expr);
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements = vec![];

        while self.current < self.tokens.len() {
            statements.push(self.declaration());
        }

        return statements;
    }
}
