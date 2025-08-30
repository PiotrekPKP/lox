use crate::{
    expression::{
        AssignExpr, BinaryExpr, CallExpr, Expr, GroupingExpr, LiteralExpr, LiteralExprType,
        LogicalExpr, TernaryExpr, UnaryExpr, VariableExpr,
    },
    lox_error,
    statement::{FunctionStatement, IfStatement, Statement, VarStatement, WhileStatement},
    token::{Keyword, Token},
};

macro_rules! consume {
    ($self:ident, $($token_type:ident)|+, $msg:expr) => {{
        if let Some(token) = $self.tokens.get($self.current) {
            match token {
                $(Token::$token_type(_))|+ => {
                    $self.current += 1;
                    token
                },
                _ => lox_error!(concat!("[line {}] ", $msg), token.line() - 1),
            }
        } else {
            lox_error!(concat!("Internal error: ", $msg));
        }
    }};

    ($self:ident, Keyword, Identifier, $msg:expr) => {{
        if let Some(token) = $self.tokens.get($self.current) {
            match token {
                Token::Keyword(inner) => match &inner.keyword {
                    Keyword::Identifier(_) => {
                        $self.current += 1;
                        token
                    },
                    _ => lox_error!(concat!("[line {}] ", $msg), token.line() - 1),
                },
                _ => lox_error!(concat!("[line {}] ", $msg), token.line() - 1),
            }
        } else {
            lox_error!(concat!("Internal error: ", $msg));
        }
    }};

    ($self:ident, Keyword, $inner:ident, $msg:expr) => {{
        if let Some(token) = $self.tokens.get($self.current) {
            match token {
                Token::Keyword(inner) => match &inner.keyword {
                    Keyword::$inner => {
                        $self.current += 1;
                        token
                    },
                    _ => lox_error!(concat!("[line {}] ", $msg), token.line() - 1),
                },
                _ => lox_error!(concat!("[line {}] ", $msg), token.line() - 1),
            }
        } else {
            lox_error!(concat!("Internal error: ", $msg));
        }
    }};
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

    fn expression(&mut self) -> Expr {
        return self.assignment();
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if let Some(token) = match_token!(self, Equal) {
            let line = token.line();
            let value = self.assignment();

            match expr {
                Expr::Variable(v) => {
                    return Expr::Assign(AssignExpr {
                        name: v.name,
                        value: Box::new(value),
                    });
                }
                _ => {
                    lox_error!("[line {}] Error: Invalid assignment target.", line)
                }
            }
        }

        return expr;
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while let Some(op) = match_token!(self, Keyword, Or) {
            let operator = op.clone();
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

        while let Some(op) = match_token!(self, Keyword, And) {
            let operator = op.clone();
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

        while let Some(op) = match_token!(self, BangEqual | EqualEqual) {
            let operator = op.clone();
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

        while let Some(op) = match_token!(self, Greater | GreaterEqual | Less | LessEqual) {
            let operator = op.clone();
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

        while let Some(op) = match_token!(self, Minus | Plus) {
            let operator = op.clone();
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

        while let Some(op) = match_token!(self, Slash | Star) {
            let operator = op.clone();
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
        if let Some(op) = match_token!(self, Bang | Minus) {
            let operator = op.clone();
            let right = self.unary();

            return Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            });
        }

        return self.call();
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        loop {
            if let Some(_) = match_token!(self, LeftParen) {
                expr = self.finish_call(expr);
            } else {
                break;
            }
        }

        return expr;
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut args = vec![];

        if let Some(token) = self.tokens.get(self.current) {
            let line = token.line();

            match token {
                Token::RightParen(_) => {}
                _ => loop {
                    if args.len() >= 255 {
                        lox_error!(
                            "[line {}] Error: Cannot have more than 255 arguments.",
                            line
                        );
                    }

                    args.push(self.expression());

                    if match_token!(self, Comma).is_none() {
                        break;
                    }
                },
            }
        }

        let paren = consume!(self, RightParen, "Error: Expect ')' after arguments.");

        return Expr::Call(CallExpr {
            arguments: args,
            paren: paren.clone(),
            callee: Box::new(callee),
        });
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
                            "[line {}] Error: Unexpected identifier '{}' encountered.",
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
        if let Some(_) = match_token!(self, Keyword, Fun) {
            return self.function("function");
        }

        if let Some(_) = match_token!(self, Keyword, Var) {
            return self.var_declaration();
        }

        return self.statement();
    }

    fn function(&mut self, _kind: &str) -> Statement {
        let name = consume!(self, Keyword, Identifier, "Error: Expected function name.");
        consume!(self, LeftParen, "Error: Expected '(' after function name.");

        let mut parameters = vec![];
        if let Some(token) = self.tokens.get(self.current) {
            let line = token.line();

            match token {
                Token::RightParen(_) => {}
                _ => loop {
                    if parameters.len() > 255 {
                        lox_error!(
                            "[line {}] Error: Can't have more than 255 parameters.",
                            line
                        );
                    }

                    parameters.push(
                        consume!(self, Keyword, Identifier, "Error: Expected parameter name.")
                            .clone(),
                    );

                    if match_token!(self, Comma).is_none() {
                        break;
                    }
                },
            }
        }

        consume!(self, RightParen, "Error: Expected ')' after parameters.");
        consume!(
            self,
            LeftBrace,
            "Error: Expected '{{' before function body."
        );

        let name = match name {
            Token::Keyword(k) => match &k.keyword {
                Keyword::Identifier(n) => n,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        return Statement::Function(FunctionStatement {
            name: name.clone(),
            params: parameters,
            body: self.block(),
        });
    }

    fn var_declaration(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current) {
            let line = token.line();

            match token {
                Token::Keyword(k) => match &k.keyword {
                    Keyword::Identifier(name) => {
                        let n = name.clone();

                        self.current += 1;

                        let initializer = if let Some(_) = match_token!(self, Equal) {
                            Some(self.expression())
                        } else {
                            None
                        };

                        consume!(self, Semicolon, "Error: Missing ';'.");

                        return Statement::Var(VarStatement {
                            name: n,
                            initializer,
                        });
                    }
                    _ => lox_error!(
                        "[line {}] Error: The name of your variable cannot be a keyword.",
                        line
                    ),
                },
                _ => lox_error!("[line {}] Error: Provide a name for your variable.", line),
            }
        }

        lox_error!(
            "[line {}] Error: Provide a name for your variable.",
            self.tokens.last().unwrap().line()
        );
    }

    fn statement(&mut self) -> Statement {
        if let Some(_) = match_token!(self, Keyword, For) {
            return self.for_statement();
        }

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

    fn for_statement(&mut self) -> Statement {
        consume!(self, LeftParen, "Error: Expect '(' after 'for'.");

        let initializer;
        if let Some(_) = match_token!(self, Semicolon) {
            initializer = None;
        } else if let Some(_) = match_token!(self, Keyword, Var) {
            initializer = Some(self.var_declaration());
        } else {
            initializer = Some(self.expression_statement());
        }

        let mut condition = None;
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Semicolon(_) => {}
                _ => condition = Some(self.expression()),
            }
        }
        consume!(self, Semicolon, "Error: Expect ';' after loop condition.");

        let mut increment = None;
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::RightParen(_) => {}
                _ => increment = Some(self.expression()),
            }
        }
        consume!(self, RightParen, "Error: Expect ')' after for clauses.");

        let mut body = self.statement();

        if let Some(incr) = increment {
            body = Statement::Block(vec![body, Statement::Expression(incr)]);
        }

        body = Statement::While(WhileStatement {
            condition: condition.unwrap_or(Expr::Literal(LiteralExpr {
                value: LiteralExprType::Identifier(Keyword::True),
            })),
            body: Box::new(body),
            in_for_loop: true,
        });

        if let Some(init) = initializer {
            body = Statement::Block(vec![init, body]);
        }

        return body;
    }

    fn while_statement(&mut self) -> Statement {
        consume!(self, LeftParen, "Error: Expected '(' after 'while'.");

        let condition = self.expression();

        consume!(self, RightParen, "Error: Expected ')' after condition.");

        let body = self.statement();

        return Statement::While(WhileStatement {
            body: Box::new(body),
            condition,
            in_for_loop: false,
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
