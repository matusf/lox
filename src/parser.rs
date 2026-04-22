use std::{fmt::Display, iter::Peekable, vec};

use crate::tokenizer::{Token, TokenType};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Parser error")]
pub struct Error {
    // line: usize,
    message: String,
}

#[derive(Debug)]
pub enum Literal<'a> {
    Bool(bool),
    Nil,
    Number(f64),
    String(&'a str),
    Identifier(&'a str),
}

impl<'a> Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Bool(b) => write!(f, "{:?}", b),
            Literal::Nil => write!(f, "nil"),
            Literal::Number(num) => {
                if num.trunc() == *num {
                    write!(f, "{num}.0")
                } else {
                    write!(f, "{num}")
                }
            }
            Literal::String(s) => write!(f, "{}", s.trim_matches('"')),
            Literal::Identifier(ident) => write!(f, "{ident}"),
        }
    }
}

impl<'a> From<Token<'a>> for Literal<'a> {
    fn from(token: Token<'a>) -> Self {
        match token.typ {
            TokenType::True => Self::Bool(true),
            TokenType::False => Self::Bool(false),
            TokenType::Nil => Self::Nil,
            TokenType::Number => {
                let num: f64 = token.lexeme.parse().expect("parse num");
                Self::Number(num)
            }
            TokenType::String => Self::String(token.lexeme),
            TokenType::Identifier => Self::Identifier(token.lexeme),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum BinOp {
    BangEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Sub,
    Add,
    Mul,
    Div,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::BangEqual => write!(f, "!="),
            BinOp::EqualEqual => write!(f, "=="),
            BinOp::Greater => write!(f, ">"),
            BinOp::GreaterEqual => write!(f, ">="),
            BinOp::Less => write!(f, "<"),
            BinOp::LessEqual => write!(f, "<="),
            BinOp::Sub => write!(f, "-"),
            BinOp::Add => write!(f, "+"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
        }
    }
}

impl<'a> From<Token<'a>> for BinOp {
    fn from(token: Token<'a>) -> Self {
        match token.typ {
            TokenType::Minus => Self::Sub,
            TokenType::Plus => Self::Add,
            TokenType::Slash => Self::Div,
            TokenType::Star => Self::Mul,
            TokenType::BangEqual => Self::BangEqual,
            TokenType::EqualEqual => Self::EqualEqual,
            TokenType::Greater => Self::Greater,
            TokenType::GreaterEqual => Self::GreaterEqual,
            TokenType::Less => Self::Less,
            TokenType::LessEqual => Self::LessEqual,
            // TokenType::And => todo!(),
            // TokenType::Or => todo!(),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    Bang,
    Minus,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Bang => write!(f, "!"),
            UnaryOp::Minus => write!(f, "-"),
        }
    }
}

impl<'a> From<Token<'a>> for UnaryOp {
    fn from(token: Token<'a>) -> Self {
        match token.typ {
            TokenType::Bang => UnaryOp::Bang,
            TokenType::Minus => UnaryOp::Minus,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
    Literal(Literal<'a>),
    Group(Box<Expr<'a>>),
    BinOp(BinOp, Box<Expr<'a>>, Box<Expr<'a>>),
    UnaryOp(UnaryOp, Box<Expr<'a>>),
}

impl<'a> Display for Expr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{literal}"),
            Expr::Group(expr) => write!(f, "(group {expr})"),
            Expr::BinOp(bin_op, lhs, rhs) => write!(f, "({bin_op} {lhs} {rhs})"),
            Expr::UnaryOp(unary_op, expr) => write!(f, "({unary_op} {expr})"),
        }
    }
}

pub struct Parser<'a> {
    source: &'a str,
    tokens: Peekable<vec::IntoIter<Token<'a>>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Vec<Token<'a>>) -> Self {
        Self {
            source,
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr<'a>, Error> {
        self.parse_expression()
    }

    // expression → equality ;
    fn parse_expression(&mut self) -> Result<Expr<'a>, Error> {
        self.parse_equality()
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn parse_equality(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_comparison()?;

        while self.peek_match(|t| matches!(t, TokenType::BangEqual | TokenType::EqualEqual)) {
            let op = self
                .expect_match(|t| matches!(t, TokenType::BangEqual | TokenType::EqualEqual))?
                .into();
            let rhs = self.parse_comparison()?;
            expr = Expr::BinOp(op, Box::new(expr), Box::new(rhs));
        }

        Ok(expr)
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn parse_comparison(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_term()?;
        while self.peek_match(|t| {
            matches!(
                t,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        }) {
            let op = self
                .expect_match(|t| {
                    matches!(
                        t,
                        TokenType::Greater
                            | TokenType::GreaterEqual
                            | TokenType::Less
                            | TokenType::LessEqual
                    )
                })?
                .into();
            expr = Expr::BinOp(op, Box::new(expr), Box::new(self.parse_term()?))
        }

        Ok(expr)
    }

    // term → factor ( ( "-" | "+" ) factor )* ;
    fn parse_term(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_factor()?;

        while self.peek_match(|t| matches!(t, TokenType::Minus | TokenType::Plus)) {
            let op = self
                .expect_match(|t| matches!(t, TokenType::Minus | TokenType::Plus))?
                .into();
            expr = Expr::BinOp(op, Box::new(expr), Box::new(self.parse_factor()?));
        }

        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )* ;
    fn parse_factor(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_unary()?;

        while self.peek_match(|t| matches!(t, TokenType::Slash | TokenType::Star)) {
            let op = self
                .expect_match(|t| matches!(t, TokenType::Slash | TokenType::Star))?
                .into();
            expr = Expr::BinOp(op, Box::new(expr), Box::new(self.parse_unary()?));
        }

        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | primary ;
    fn parse_unary(&mut self) -> Result<Expr<'a>, Error> {
        if self.peek_match(|t| matches!(t, TokenType::Bang | TokenType::Minus)) {
            let op = self
                .expect_match(|t| matches!(t, TokenType::Bang | TokenType::Minus))?
                .into();
            return Ok(Expr::UnaryOp(op, Box::new(self.parse_unary()?)));
        }
        self.parse_primary()
    }

    // primary → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn parse_primary(&mut self) -> Result<Expr<'a>, Error> {
        let token = self.tokens.next().ok_or(Error {
            message: "No primary".to_string(),
        })?;

        match token.typ {
            TokenType::LeftParen => {
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(Expr::Group(Box::new(expr)))
            }
            TokenType::Identifier
            | TokenType::String
            | TokenType::Number
            | TokenType::Nil
            | TokenType::False
            | TokenType::True => Ok(Expr::Literal(token.into())),
            _ => Err(Error {
                message: "Unexpexted token in primary".to_string(),
            }),
        }
    }

    fn peek_match(&mut self, predicate: fn(TokenType) -> bool) -> bool {
        self.tokens
            .peek()
            .map(|token| predicate(token.typ))
            .unwrap_or_default()
    }

    fn expect_match(&mut self, predicate: impl Fn(TokenType) -> bool) -> Result<Token<'a>, Error> {
        match self.tokens.next() {
            Some(t) if predicate(t.typ) => Ok(t),
            Some(t) => Err(Error {
                message: format!("Unexpected token {:?}", t.lexeme),
            }),
            None => Err(Error {
                message: "Unexpected eof".to_string(),
            }),
        }
    }

    fn expect(&mut self, token_type: TokenType) -> Result<Token<'a>, Error> {
        self.expect_match(|t| t == token_type)
    }
}
