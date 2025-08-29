use crate::{
    expression::{
        AssignExpr, BinaryExpr, Expr, GroupingExpr, LiteralExpr, LiteralExprType, LogicalExpr,
        TernaryExpr, UnaryExpr, VariableExpr,
    },
    parse_error,
    statement::{IfStatement, Statement, VarStatement},
    token::{Keyword, Token},
};

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

        if let Some(token) = self.tokens.get(self.current).cloned() {
            match token {
                Token::Equal(_) => {
                    self.current += 1;

                    let value = self.assignment();

                    match expr {
                        Expr::Variable(v) => {
                            return Expr::Assign(AssignExpr {
                                name: v.name,
                                value: Box::new(value),
                            });
                        }
                        _ => parse_error!(
                            "[line {}] Error: Invalid assignment target.",
                            token.line()
                        ),
                    }
                }
                _ => {}
            }
        }

        return expr;
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Keyword(k) => match k.keyword {
                    Keyword::Or => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.and();

                        expr = Expr::Logical(LogicalExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                _ => break,
            }
        }

        return expr;
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.ternary();

        while let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Keyword(k) => match k.keyword {
                    Keyword::And => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.ternary();

                        expr = Expr::Logical(LogicalExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                _ => break,
            }
        }

        return expr;
    }

    fn ternary(&mut self) -> Expr {
        let mut expr = self.equality();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::QuestionMark(_) => {
                        self.current += 1;

                        let trueish = self.ternary();

                        if let Some(token_n) = self.tokens.get(self.current) {
                            match token_n {
                                Token::Colon(_) => {
                                    self.current += 1;
                                }
                                _ => parse_error!(
                                    "[line {}] Error: Missing ':' in ternary expression.",
                                    token_n.line()
                                ),
                            }
                        }

                        let falseish = self.ternary();

                        expr = Expr::Ternary(TernaryExpr {
                            condition: Box::new(expr),
                            trueish: Box::new(trueish),
                            falseish: Box::new(falseish),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::BangEqual(_) | Token::EqualEqual(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.comparison();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::Greater(_)
                    | Token::GreaterEqual(_)
                    | Token::Less(_)
                    | Token::LessEqual(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.term();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::Minus(_) | Token::Plus(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.factor();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        loop {
            match self.tokens.get(self.current) {
                Some(token) => match token {
                    Token::Slash(_) | Token::Star(_) => {
                        self.current += 1;

                        let operator = self.previous();
                        let right = self.unary();
                        expr = Expr::Binary(BinaryExpr {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(right),
                        })
                    }
                    _ => break,
                },
                None => break,
            }
        }

        return expr;
    }

    fn unary(&mut self) -> Expr {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Bang(_) | Token::Minus(_) => {
                    self.current += 1;

                    let operator = self.previous();
                    let right = self.unary();
                    return Expr::Unary(UnaryExpr {
                        operator,
                        right: Box::new(right),
                    });
                }
                _ => {}
            }
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
                        parse_error!(
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

                    if let Some(token_n) = self.tokens.get(self.current) {
                        match token_n {
                            Token::RightParen(_) => {
                                self.current += 1;
                            }
                            _ => parse_error!("[line {}] Error: Missing ')'.", token_n.line()),
                        }
                    }

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
                    parse_error!("Unexpected token {} encountered.", token);
                }
            }
        }

        parse_error!("Empty file cannot be parsed.");
    }

    fn declaration(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Keyword(id) => match id.keyword {
                    Keyword::Var => {
                        self.current += 1;
                        return self.var_declaration();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        return self.statement();
    }

    fn var_declaration(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current).cloned() {
            match token {
                Token::Keyword(ref id) => match &id.keyword {
                    Keyword::Identifier(name) => {
                        self.current += 1;

                        let initializer =
                            if let Some(Token::Equal(_)) = self.tokens.get(self.current) {
                                self.current += 1;

                                Some(self.expression())
                            } else {
                                None
                            };

                        if let Some(token_n) = self.tokens.get(self.current) {
                            match token_n {
                                Token::Semicolon(_) => {
                                    self.current += 1;
                                }
                                _ => parse_error!("[line {}] Error: Missing ';'.", token_n.line()),
                            }
                        }

                        return Statement::Var(VarStatement {
                            name: name.clone(),
                            initializer,
                        });
                    }
                    _ => parse_error!(
                        "[line {}] Error: The name of your variable cannot be a keyword.",
                        token.line()
                    ),
                },
                _ => parse_error!(
                    "[line {}] Error: Provide a name for your variable.",
                    token.line()
                ),
            }
        }

        parse_error!(
            "[line {}] Error: Provide a name for your variable.",
            self.tokens.last().unwrap().line()
        );
    }

    fn statement(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Keyword(id) => match id.keyword {
                    Keyword::Print => {
                        self.current += 1;
                        return self.print_statement();
                    }
                    Keyword::If => {
                        self.current += 1;
                        return self.if_statement();
                    }
                    _ => {}
                },
                Token::LeftBrace(_) => {
                    self.current += 1;

                    return Statement::Block(self.block());
                }
                _ => {}
            }
        }

        return self.expression_statement();
    }

    fn block(&mut self) -> Vec<Statement> {
        let mut statements = vec![];

        while let Some(token) = self.tokens.get(self.current) {
            if let Token::RightBrace(_) = token {
                break;
            }

            statements.push(self.declaration());
        }

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::RightBrace(_) => {
                    self.current += 1;
                }
                _ => parse_error!("[line {}] Error: Missing '}}'.", token.line()),
            }
        }

        return statements;
    }

    fn if_statement(&mut self) -> Statement {
        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::LeftParen(_) => {
                    self.current += 1;
                }
                _ => parse_error!("[line {}] Error: Expected '(' after 'if'.", token.line()),
            }
        }

        let condition = self.expression();

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::RightParen(_) => {
                    self.current += 1;
                }
                _ => parse_error!(
                    "[line {}] Error: Expected ')' after if condition.",
                    token.line()
                ),
            }
        }

        let then_branch = self.statement();
        let mut else_branch = None;

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Keyword(k) => match k.keyword {
                    Keyword::Else => {
                        self.current += 1;
                        else_branch = Some(self.statement());
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        return Statement::If(IfStatement {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        });
    }

    fn print_statement(&mut self) -> Statement {
        let value = self.expression();

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Semicolon(_) => {
                    self.current += 1;
                }
                _ => parse_error!("[line {}] Error: Missing ';'.", token.line()),
            }
        }

        return Statement::Print(value);
    }

    fn expression_statement(&mut self) -> Statement {
        let expr = self.expression();

        if let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Semicolon(_) => {
                    self.current += 1;
                }
                Token::Eof(_) => {
                    self.current += 1;
                }
                _ => parse_error!("[line {}] Error: Missing ';'.", token.line()),
            }
        }

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
