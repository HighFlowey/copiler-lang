use crate::frontend::lexer::{Literal, Token};
use crate::util::cursor::Cursor;

#[derive(Debug, Clone)]
pub enum Statement {
    VariableDeclaration {
        ident: Expression,
        value: Expression,
    },
    ClassConstructor {
        ident: Expression,
        body: Expression,
    },
    If {
        expr: Expression,
        body: Box<Statement>,
    },
    Scope(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    FunctionCall {
        ident: Box<Expression>,
        args: Vec<Expression>,
    },
    Function {
        params: Vec<Expression>,
        stmt: Box<Statement>,
    },
    Parameter {
        ident: Box<Expression>,
        expected_type: Box<Option<Expression>>,
    },
    ClassBody {
        properties: Vec<Statement>,
    },
    Identifier(String),
    Indexing(Box<Expression>, Box<Expression>),
    String(String),
    Bool(bool),
    Char(char),
    Number(f32),
}

pub struct Parser {
    cursor: Cursor<Token>,
}

impl Parser {
    fn parse_expression(&mut self) -> Option<Expression> {
        if let Some(token) = self.cursor.peek(None) {
            let token = match token {
                Token::Identifier(identifier) => {
                    let identifier = Expression::Identifier(identifier);

                    if matches!(self.cursor.peek(Some(2)), Some(Token::Dot)) {
                        self.cursor.eat();
                        self.cursor.eat();

                        let indexed = self.parse_expression().unwrap();

                        match indexed.clone() {
                            Expression::FunctionCall { ident, args, .. } => {
                                return Some(Expression::FunctionCall {
                                    ident: Box::new(Expression::Indexing(
                                        Box::new(identifier),
                                        ident,
                                    )),
                                    args,
                                });
                            }
                            _ => {
                                return Some(Expression::Indexing(
                                    Box::new(identifier),
                                    Box::new(indexed),
                                ));
                            }
                        }
                    } else if matches!(self.cursor.peek(Some(2)), Some(Token::LParen)) {
                        self.cursor.eat();
                        self.cursor.eat();

                        let mut args = vec![];

                        while self
                            .cursor
                            .peek_iff(None, |token| !matches!(token, Token::RParen))
                            .is_some()
                        {
                            args.push(self.parse_expression().unwrap());

                            if !matches!(self.cursor.eat(), Some(Token::Comma)) {
                                break;
                            }
                        }

                        Some(Expression::FunctionCall {
                            ident: Box::new(identifier),
                            args,
                        })
                    } else {
                        Some(identifier)
                    }
                }
                Token::Literal(literal) => match literal {
                    Literal::Char(char) => Some(Expression::Char(char)),
                    Literal::Identifier(ident) => Some(Expression::Identifier(ident)),
                    Literal::Number(number) => Some(Expression::Number(number)),
                    Literal::String(string) => Some(Expression::String(string)),
                    Literal::Bool(bool) => Some(Expression::Bool(bool)),
                },
                _ => None,
            };

            if token.is_some() {
                self.cursor.eat();

                token
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse_parameter(&mut self) -> Option<Expression> {
        let ident = self.parse_expression();

        ident.as_ref()?;

        let expected_type = if self
            .cursor
            .eat_iff(|token| matches!(token, Token::Colon))
            .is_some()
        {
            self.parse_expression()
        } else {
            None
        };

        Some(Expression::Parameter {
            ident: Box::new(ident.unwrap()),
            expected_type: Box::new(expected_type),
        })
    }

    fn parse_class_body(&mut self) -> Expression {
        let mut properties = vec![];

        self.cursor
            .eat_iff(|token| matches!(token, Token::LScope))
            .unwrap();

        while self
            .cursor
            .eat_iff(|token| !matches!(token, Token::RScope))
            .is_some()
        {
            let property = self.parse_variable();

            if let Some(property) = property {
                properties.push(property);
            }
        }

        self.cursor.eat();

        Expression::ClassBody { properties }
    }

    fn parse_if_statement(&mut self) -> Statement {
        let expr = self.parse_expression().unwrap();

        self.cursor
            .eat_iff(|token| matches!(token, Token::LScope))
            .unwrap();

        let body = self.parse_scope();

        Statement::If {
            expr,
            body: Box::new(body),
        }
    }

    fn parse_class(&mut self) -> Statement {
        let ident = self.parse_expression();

        Statement::ClassConstructor {
            ident: ident.expect("Expected class identifier"),
            body: self.parse_class_body(),
        }
    }

    fn parse_variable(&mut self) -> Option<Statement> {
        let ident = self.parse_expression();
        let ident = ident.as_ref()?.clone();

        self.cursor.eat_iff(|token| matches!(token, Token::Equal))?;

        if matches!(self.cursor.peek(None), Some(Token::LParen)) {
            let mut params: Vec<Expression> = vec![];
            self.cursor.eat(); // (

            while let Some(expr) = self.parse_parameter() {
                params.push(expr);

                if matches!(self.cursor.peek(None), Some(Token::Comma)) {
                    self.cursor.eat();
                } else {
                    break;
                }
            }

            self.cursor
                .eat_iff(|token| matches!(token, Token::RParen))
                .unwrap();

            self.cursor
                .eat_iff(|token| matches!(token, Token::LScope))
                .unwrap();

            let scope = self.parse_scope();

            Some(Statement::VariableDeclaration {
                ident,
                value: Expression::Function {
                    params,
                    stmt: Box::new(scope),
                },
            })
        } else {
            let value = self.parse_expression();
            let value = value.as_ref()?.clone();

            Some(Statement::VariableDeclaration { ident, value })
        }
    }

    fn parse_fn_call(&mut self) -> Statement {
        let ident = self.parse_expression().unwrap();

        Statement::VariableDeclaration {
            ident: Expression::Identifier(String::from("_")),
            value: ident,
        }
    }

    pub fn parse_statement(&mut self) -> Option<Statement> {
        if let Some(token) = self.cursor.peek(None) {
            match token {
                Token::Identifier(identifier) => match identifier.as_str() {
                    "let" => {
                        self.cursor.eat();
                        Some(self.parse_variable().unwrap())
                    }
                    "class" => {
                        self.cursor.eat();
                        Some(self.parse_class())
                    }
                    "if" => {
                        self.cursor.eat();
                        Some(self.parse_if_statement())
                    }
                    _ => Some(self.parse_fn_call()),
                },
                Token::LScope => {
                    self.cursor.eat();
                    Some(self.parse_scope())
                }
                Token::RScope => None,
                _ => todo!("{token:?}"),
            }
        } else {
            None
        }
    }

    pub fn parse_scope(&mut self) -> Statement {
        let mut stack: Vec<Statement> = Vec::new();

        while let Some(expr) = self.parse_statement() {
            stack.push(expr);
        }

        Statement::Scope(stack)
    }

    pub fn load(&mut self) -> Statement {
        self.parse_scope()
    }

    pub fn new(tokens: &[Token]) -> Self {
        Self {
            cursor: Cursor::new(tokens.to_owned()),
        }
    }
}
