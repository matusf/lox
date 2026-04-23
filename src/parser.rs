use std::{fmt::Display, iter::Peekable, vec};

use crate::tokenizer::{Token, TokenType};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Parser error")]
pub enum Error {
    #[error("[line {line}] Error: Unexpected token: {lexeme}")]
    UnexpectedToken { line: usize, lexeme: String },
    #[error("Unexpected end of file.")]
    UnexpectedEof,
    #[error("Unvalid assignment to: {expr}")]
    InvalidAssignment { expr: String },
}

#[derive(Debug)]
pub enum Literal<'a> {
    Bool(bool),
    Nil,
    Number(f64),
    String(&'a str),
    Identifier(&'a str),
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Bool(b) => write!(f, "{b:?}"),
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
            Self::BangEqual => write!(f, "!="),
            Self::EqualEqual => write!(f, "=="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Sub => write!(f, "-"),
            Self::Add => write!(f, "+"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
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
    Negate,
    Minus,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negate => write!(f, "!"),
            Self::Minus => write!(f, "-"),
        }
    }
}

impl<'a> From<Token<'a>> for UnaryOp {
    fn from(token: Token<'a>) -> Self {
        match token.typ {
            TokenType::Bang => Self::Negate,
            TokenType::Minus => Self::Minus,
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
    Assign(&'a str, Box<Expr<'a>>),
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{literal}"),
            Expr::Group(expr) => write!(f, "(group {expr})"),
            Expr::BinOp(bin_op, lhs, rhs) => write!(f, "({bin_op} {lhs} {rhs})"),
            Expr::UnaryOp(unary_op, expr) => write!(f, "({unary_op} {expr})"),
            Expr::Assign(ident, expr) => write!(f, "(= {ident} {expr})"),
        }
    }
}

#[derive(Debug)]
pub enum Statement<'a> {
    Expr(Expr<'a>),
    Print(Expr<'a>),
    VarDecl(&'a str, Option<Expr<'a>>),
    Block(Vec<Statement<'a>>),
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Expr(expr) => write!(f, "(; {expr})"),
            Statement::Print(expr) => write!(f, "(print {expr})"),
            Statement::VarDecl(name, Some(expr)) => write!(f, "(var {name} {expr})"),
            Statement::VarDecl(name, None) => write!(f, "(var {name})"),
            Statement::Block(statements) => {
                write!(f, "(block")?;
                for statement in statements {
                    write!(f, " {statement}")?;
                }
                write!(f, ")")
            }
        }
    }
}

pub struct Parser<'a> {
    _source: &'a str,
    tokens: Peekable<vec::IntoIter<Token<'a>>>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Statement<'a>, Error>;

    // program → declaration* EOF ;
    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.peek()?;
        Some(self.parse_declaration())
    }
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(source: &'a str, tokens: Vec<Token<'a>>) -> Self {
        Self {
            _source: source,
            tokens: tokens.into_iter().peekable(),
        }
    }

    // declaration → varDecl | statement ;
    fn parse_declaration(&mut self) -> Result<Statement<'a>, Error> {
        // TODO: Sync the parser here?

        if self.peek_eq(TokenType::Var) {
            self.parse_var_decl()
        } else {
            self.parse_statement()
        }
    }

    // varDecl → "var" IDENTIFIER ( "=" expression )? ";" ;
    fn parse_var_decl(&mut self) -> Result<Statement<'a>, Error> {
        self.expect(TokenType::Var)?;
        let ident = self.expect(TokenType::Identifier)?.lexeme;

        let expr = if self.parse_if_eq(TokenType::Equal).is_some() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(TokenType::Semicolon)?;
        Ok(Statement::VarDecl(ident, expr))
    }

    // statement → exprStmt | printStmt | block ;
    fn parse_statement(&mut self) -> Result<Statement<'a>, Error> {
        if self.peek_eq(TokenType::Print) {
            self.parse_print_statement()
        } else if self.peek_eq(TokenType::LeftBrace) {
            self.parse_block()
        } else {
            self.parse_expr_statement()
        }
    }

    // exprStmt → expression ";" ;
    fn parse_expr_statement(&mut self) -> Result<Statement<'a>, Error> {
        let expr = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;
        Ok(Statement::Expr(expr))
    }

    // printStmt → "print" expression ";" ;
    fn parse_print_statement(&mut self) -> Result<Statement<'a>, Error> {
        self.expect(TokenType::Print)?;
        let expr = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;
        Ok(Statement::Print(expr))
    }

    // block → "{" declaration* "}" ;
    fn parse_block(&mut self) -> Result<Statement<'a>, Error> {
        self.expect(TokenType::LeftBrace)?;

        let mut block = Vec::new();
        while !self.peek_eq(TokenType::RightBrace) {
            block.push(self.parse_declaration()?);
        }

        self.expect(TokenType::RightBrace)?;
        Ok(Statement::Block(block))
    }

    // expression → assignment ;
    pub fn parse_expression(&mut self) -> Result<Expr<'a>, Error> {
        self.parse_assignment()
    }

    // assignment → IDENTIFIER "=" assignment | equality ;
    fn parse_assignment(&mut self) -> Result<Expr<'a>, Error> {
        let expr = self.parse_equality()?;
        if self.parse_if_eq(TokenType::Equal).is_some() {
            let rhs = self.parse_assignment()?;
            if let Expr::Literal(Literal::Identifier(name)) = expr {
                Ok(Expr::Assign(name, Box::new(rhs)))
            } else {
                Err(Error::InvalidAssignment {
                    expr: expr.to_string(),
                })
            }
        } else {
            // Parsed equality
            Ok(expr)
        }
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn parse_equality(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_comparison()?;

        while let Some(op) =
            self.parse_if(|t| matches!(t, TokenType::BangEqual | TokenType::EqualEqual))
        {
            let rhs = self.parse_comparison()?;
            expr = Expr::BinOp(op.into(), Box::new(expr), Box::new(rhs));
        }

        Ok(expr)
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn parse_comparison(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_term()?;
        while let Some(op) = self.parse_if(|t| {
            matches!(
                t,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        }) {
            expr = Expr::BinOp(op.into(), Box::new(expr), Box::new(self.parse_term()?));
        }

        Ok(expr)
    }

    // term → factor ( ( "-" | "+" ) factor )* ;
    fn parse_term(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_factor()?;

        while let Some(op) = self.parse_if(|t| matches!(t, TokenType::Minus | TokenType::Plus)) {
            expr = Expr::BinOp(op.into(), Box::new(expr), Box::new(self.parse_factor()?));
        }

        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )* ;
    fn parse_factor(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_unary()?;

        while let Some(op) = self.parse_if(|t| matches!(t, TokenType::Slash | TokenType::Star)) {
            expr = Expr::BinOp(op.into(), Box::new(expr), Box::new(self.parse_unary()?));
        }

        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | primary ;
    fn parse_unary(&mut self) -> Result<Expr<'a>, Error> {
        if let Some(op) = self.parse_if(|t| matches!(t, TokenType::Bang | TokenType::Minus)) {
            return Ok(Expr::UnaryOp(op.into(), Box::new(self.parse_unary()?)));
        }
        self.parse_primary()
    }

    // primary → "true" | "false" | "nil" | NUMBER | STRING | "(" expression ")" | IDENTIFIER ;
    fn parse_primary(&mut self) -> Result<Expr<'a>, Error> {
        let token = self.tokens.next().ok_or(Error::UnexpectedEof)?;

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
            _ => Err(Error::UnexpectedToken {
                line: token.line,
                lexeme: token.lexeme.to_string(),
            }),
        }
    }

    fn peek_eq(&mut self, token_type: TokenType) -> bool {
        self.tokens.peek().is_some_and(|t| t.typ == token_type)
    }

    fn parse_if(&mut self, predicate: impl Fn(TokenType) -> bool) -> Option<Token<'a>> {
        self.tokens.next_if(|t| predicate(t.typ))
    }

    fn parse_if_eq(&mut self, token_type: TokenType) -> Option<Token<'a>> {
        self.tokens.next_if(|t| t.typ == token_type)
    }

    fn expect_match(&mut self, predicate: impl Fn(TokenType) -> bool) -> Result<Token<'a>, Error> {
        match self.tokens.next() {
            Some(t) if predicate(t.typ) => Ok(t),
            Some(t) => Err(Error::UnexpectedToken {
                line: t.line,
                lexeme: t.lexeme.to_string(),
            }),
            None => Err(Error::UnexpectedEof),
        }
    }

    fn expect(&mut self, token_type: TokenType) -> Result<Token<'a>, Error> {
        self.expect_match(|t| t == token_type)
    }
}
