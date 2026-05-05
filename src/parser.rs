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
    #[error("Function cannot have more than 255 parameters")]
    TooManyArguments,
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

#[derive(Debug, Clone, Copy)]
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
pub enum LogicOp {
    And,
    Or,
}

impl<'a> From<Token<'a>> for LogicOp {
    fn from(token: Token<'a>) -> Self {
        match token.typ {
            TokenType::And => Self::And,
            TokenType::Or => Self::Or,
            _ => unreachable!(),
        }
    }
}

impl Display for LogicOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicOp::And => write!(f, "and"),
            LogicOp::Or => write!(f, "or"),
        }
    }
}

#[derive(Debug)]
pub struct Func<'a> {
    pub callee: Box<Expr<'a>>,
    pub args: Box<[Expr<'a>]>,
}

#[derive(Debug)]
pub enum Expr<'a> {
    Literal(Literal<'a>),
    Group(Box<Expr<'a>>),
    BinOp(BinOp, Box<Expr<'a>>, Box<Expr<'a>>),
    UnaryOp(UnaryOp, Box<Expr<'a>>),
    Assign(&'a str, Box<Expr<'a>>),
    LogicOp(LogicOp, Box<Expr<'a>>, Box<Expr<'a>>),
    Call(Func<'a>),
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{literal}"),
            Expr::Group(expr) => write!(f, "(group {expr})"),
            Expr::BinOp(bin_op, lhs, rhs) => write!(f, "({bin_op} {lhs} {rhs})"),
            Expr::UnaryOp(unary_op, expr) => write!(f, "({unary_op} {expr})"),
            Expr::Assign(ident, expr) => write!(f, "(= {ident} {expr})"),
            Expr::LogicOp(op, lhs, rhs) => write!(f, "({op} ({lhs}) ({rhs})"),
            Expr::Call(Func { callee, args }) => {
                write!(f, "(call ({callee}) (")?;
                for arg in args {
                    write!(f, " ({arg})")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug)]
pub enum Statement<'a> {
    Expr(Expr<'a>),
    Print(Expr<'a>),
    VarDecl(&'a str, Option<Expr<'a>>),
    Block(Vec<Statement<'a>>),
    IfElse(Expr<'a>, Box<Statement<'a>>, Option<Box<Statement<'a>>>),
    While(Expr<'a>, Box<Statement<'a>>),
    Func {
        name: &'a str,
        args: Box<[&'a str]>,
        body: Box<[Statement<'a>]>,
    },
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
            Statement::IfElse(expr, yes, Some(no)) => write!(f, "(if {expr} {yes} {no})"),
            Statement::IfElse(expr, yes, None) => write!(f, "(if {expr} {yes})"),
            Statement::While(expr, statement) => write!(f, "(while {expr} {statement})"),
            Statement::Func { name, args, body } => {
                write!(f, "(func {name} (")?;
                for arg in args.iter() {
                    write!(f, " {arg}")?
                }
                write!(f, ")")?;
                for statement in body {
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

    // declaration → funDecl | varDecl | statement ;
    fn parse_declaration(&mut self) -> Result<Statement<'a>, Error> {
        // TODO: Sync the parser here?

        if self.peek_eq(TokenType::Fun) {
            self.parse_function_declaration()
        } else if self.peek_eq(TokenType::Var) {
            self.parse_var_declaration()
        } else {
            self.parse_statement()
        }
    }

    // funDecl → "fun" function ;
    fn parse_function_declaration(&mut self) -> Result<Statement<'a>, Error> {
        self.expect(TokenType::Fun)?;
        self.parse_function()
    }

    // function → IDENTIFIER "(" parameters? ")" block ;
    fn parse_function(&mut self) -> Result<Statement<'a>, Error> {
        let name = self.expect(TokenType::Identifier)?.lexeme;

        self.expect(TokenType::LeftParen)?;

        let mut args = Vec::new();
        if !self.peek_eq(TokenType::RightParen) {
            args = self.parse_parameters()?;
        }

        self.expect(TokenType::RightParen)?;

        let body = self.parse_block_to_vec()?;

        Ok(Statement::Func {
            name,
            args: args.into_boxed_slice(),
            body: body.into_boxed_slice(),
        })
    }

    // parameters → IDENTIFIER ( "," IDENTIFIER )* ;
    fn parse_parameters(&mut self) -> Result<Vec<&'a str>, Error> {
        let mut args = Vec::new();
        loop {
            args.push(self.expect(TokenType::Identifier)?.lexeme);
            if self.peek_eq(TokenType::RightParen) {
                break;
            }
            self.expect(TokenType::Comma)?;
        }

        Ok(args)
    }

    // varDecl → "var" IDENTIFIER ( "=" expression )? ";" ;
    fn parse_var_declaration(&mut self) -> Result<Statement<'a>, Error> {
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

    // statement → exprStmt | forStmt | ifStmt | printStmt | whileStmt | block ;
    fn parse_statement(&mut self) -> Result<Statement<'a>, Error> {
        if self.peek_eq(TokenType::Print) {
            self.parse_print_statement()
        } else if self.peek_eq(TokenType::For) {
            self.parse_for_statement()
        } else if self.peek_eq(TokenType::If) {
            self.parse_if_statement()
        } else if self.peek_eq(TokenType::LeftBrace) {
            self.parse_block()
        } else if self.peek_eq(TokenType::While) {
            self.parse_while_statement()
        } else {
            self.parse_expr_statement()
        }
    }

    // forStmt → "for" "(" ( varDecl | exprStmt | ";" ) | expression? ";" | expression? ")" statement ;
    fn parse_for_statement(&mut self) -> Result<Statement<'a>, Error> {
        let mut block = Vec::new();

        // for (
        self.expect(TokenType::For)?;
        self.expect(TokenType::LeftParen)?;

        // var = <expr>;
        if self.peek_eq(TokenType::Var) {
            block.push(self.parse_var_declaration()?);
        } else if self.peek_eq(TokenType::Semicolon) {
            self.expect(TokenType::Semicolon)?;
        } else {
            block.push(self.parse_expr_statement()?);
        };

        // <condition> ;
        let condition = if !self.peek_eq(TokenType::Semicolon) {
            self.parse_expression()?
        } else {
            Expr::Literal(Literal::Bool(true))
        };
        self.expect(TokenType::Semicolon)?;

        // <increment> )
        let mut increment = None;
        if !self.peek_eq(TokenType::RightParen) {
            increment = Some(self.parse_expression()?);
        }
        self.expect(TokenType::RightParen)?;

        // <body>
        let body = self.parse_statement()?;

        // If there is increment create a new block as the while block consisting of
        // increment and while body. e.g. while { <body>; incr }
        let mut increment_block = Vec::new();
        if let Some(increment) = increment {
            increment_block.push(body);
            increment_block.push(Statement::Expr(increment));
            block.push(Statement::While(
                condition,
                Box::new(Statement::Block(increment_block)),
            ));
        } else {
            block.push(Statement::While(condition, Box::new(body)));
        }

        Ok(Statement::Block(block))
    }

    // ifStmt → "if" "(" expression ")" statement ( "else" statement )? ;
    fn parse_if_statement(&mut self) -> Result<Statement<'a>, Error> {
        self.expect(TokenType::If)?;
        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;
        let yes = self.parse_statement()?;

        let no = if self.peek_eq(TokenType::Else) {
            self.expect(TokenType::Else)?;
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Statement::IfElse(condition, Box::new(yes), no))
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
        let block = self.parse_block_to_vec()?;
        Ok(Statement::Block(block))
    }

    // Helper that returns parsed block without wrapping it in Statement::Block
    fn parse_block_to_vec(&mut self) -> Result<Vec<Statement<'a>>, Error> {
        self.expect(TokenType::LeftBrace)?;

        let mut block = Vec::new();
        while !self.peek_eq(TokenType::RightBrace) {
            block.push(self.parse_declaration()?);
        }

        self.expect(TokenType::RightBrace)?;
        Ok(block)
    }

    // whileStmt → "while" "(" expression ")" statement ;
    fn parse_while_statement(&mut self) -> Result<Statement<'a>, Error> {
        self.expect(TokenType::While)?;
        self.expect(TokenType::LeftParen)?;
        let expr = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;
        Ok(Statement::While(expr, Box::new(self.parse_statement()?)))
    }

    // expression → assignment ;
    pub fn parse_expression(&mut self) -> Result<Expr<'a>, Error> {
        self.parse_assignment()
    }

    // assignment → IDENTIFIER "=" assignment | logic_or ;
    fn parse_assignment(&mut self) -> Result<Expr<'a>, Error> {
        let expr = self.parse_logic_or()?;
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
            // Parsed logic or
            Ok(expr)
        }
    }

    // logic_or → logic_and ( "or" logic_and )* ;
    fn parse_logic_or(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_logic_and()?;
        while let Some(op) = self.parse_if_eq(TokenType::Or) {
            expr = Expr::LogicOp(op.into(), Box::new(expr), Box::new(self.parse_logic_and()?));
        }
        Ok(expr)
    }

    // logic_and → equality ( "and" equality )* ;
    fn parse_logic_and(&mut self) -> Result<Expr<'a>, Error> {
        let mut expr = self.parse_equality()?;
        while let Some(op) = self.parse_if_eq(TokenType::And) {
            expr = Expr::LogicOp(op.into(), Box::new(expr), Box::new(self.parse_equality()?));
        }
        Ok(expr)
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

    // unary → ( "!" | "-" ) unary | call ;
    fn parse_unary(&mut self) -> Result<Expr<'a>, Error> {
        if let Some(op) = self.parse_if(|t| matches!(t, TokenType::Bang | TokenType::Minus)) {
            return Ok(Expr::UnaryOp(op.into(), Box::new(self.parse_unary()?)));
        }
        self.parse_call()
    }

    // call → primary ( "(" arguments? ")" )* ;
    fn parse_call(&mut self) -> Result<Expr<'a>, Error> {
        let expr = self.parse_primary()?;

        if self.parse_if_eq(TokenType::LeftParen).is_some() {
            let mut args = Vec::new();
            // Start parsing function call argument list
            if !self.peek_eq(TokenType::RightParen) {
                args = self.parse_arguments()?;
            }
            self.expect(TokenType::RightParen)?;

            if args.len() >= 255 {
                return Err(Error::TooManyArguments);
            }
            Ok(Expr::Call(Func {
                callee: Box::new(expr),
                args: args.into_boxed_slice(),
            }))
        } else {
            Ok(expr)
        }
    }

    // arguments → expression ( "," expression )* ;
    fn parse_arguments(&mut self) -> Result<Vec<Expr<'a>>, Error> {
        let mut args = Vec::new();
        loop {
            args.push(self.parse_expression()?);
            if self.peek_eq(TokenType::RightParen) {
                break;
            }
            self.expect(TokenType::Comma)?;
        }

        Ok(args)
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
