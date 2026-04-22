use std::{borrow::Cow, fmt::Display};

use thiserror::Error;

use crate::parser::{BinOp, Expr, Literal, UnaryOp};

#[derive(Debug)]
pub enum Value<'a> {
    Number(f64),
    String(Cow<'a, str>),
    Bool(bool),
    Nil,
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(_), Self::Number(_)) => true,
            (Self::String(_), Self::String(_)) => true,
            (Self::Bool(_), Self::Bool(_)) => true,
            (Self::Nil, Self::Nil) => true,
            (_, _) => false,
        }
    }
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Error)]
#[error("Runtime error")]
pub enum Error {
    #[error("Mismatched types: expected `{expected}` but got `{got}`")]
    MismachedTypes { expected: String, got: String },
    #[error("Invalid Unary operation: {op} {value}")]
    InvalidUnaryOperation { op: String, value: String },
    #[error("Invalid binary operation: {lhs} {op} {rhs}")]
    InvalidBinaryOperation {
        op: String,
        lhs: String,
        rhs: String,
    },
}

pub fn eval(expr: Expr<'_>) -> Result<Value<'_>, Error> {
    let value = match expr {
        Expr::Literal(literal) => match literal {
            Literal::Bool(b) => Value::Bool(b),
            Literal::Nil => Value::Nil,
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(Cow::Borrowed(s.trim_matches('"'))),
            Literal::Identifier(_) => todo!(),
        },
        Expr::Group(expr) => eval(*expr)?,
        Expr::BinOp(bin_op, lhs, rhs) => eval_bin_op(bin_op, *lhs, *rhs)?,
        Expr::UnaryOp(unary_op, expr) => eval_unary_op(unary_op, *expr)?,
    };

    Ok(value)
}

fn eval_unary_op(unary_op: UnaryOp, expr: Expr<'_>) -> Result<Value<'_>, Error> {
    let value = eval(expr)?;
    match (&unary_op, &value) {
        (UnaryOp::Negate, Value::Bool(b)) => Ok(Value::Bool(!b)),
        (UnaryOp::Negate, Value::Nil) => Ok(Value::Bool(true)),
        (UnaryOp::Negate, _) => Ok(Value::Bool(false)),
        (UnaryOp::Minus, Value::Number(n)) => Ok(Value::Number(-n)),
        (_, _) => Err(Error::InvalidUnaryOperation {
            op: unary_op.to_string(),
            value: value.to_string(),
        }),
    }
}

fn eval_bin_op<'a>(bin_op: BinOp, lhs: Expr<'a>, rhs: Expr<'a>) -> Result<Value<'a>, Error> {
    let lhs = eval(lhs)?;
    let rhs = eval(rhs)?;
    use BinOp::*;
    match (bin_op, lhs, rhs) {
        (BangEqual, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Bool(lhs != rhs)),
        (BangEqual, Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs != rhs)),
        (BangEqual, Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs != rhs)),
        (BangEqual, Value::Nil, Value::Nil) => Ok(Value::Bool(false)),
        (BangEqual, _, _) => Ok(Value::Bool(true)),

        (EqualEqual, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Bool(lhs == rhs)),
        (EqualEqual, Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs == rhs)),
        (EqualEqual, Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs == rhs)),
        (EqualEqual, Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
        (EqualEqual, _, _) => Ok(Value::Bool(false)),

        (Greater, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Bool(lhs > rhs)),
        (GreaterEqual, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Bool(lhs >= rhs)),
        (Less, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Bool(lhs < rhs)),
        (LessEqual, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Bool(lhs <= rhs)),
        (Sub, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs - rhs)),
        (Add, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs + rhs)),
        (Add, Value::String(lhs), Value::String(rhs)) => {
            // TODO: Can reuse allocation if lhs is already owned
            Ok(Value::String(Cow::Owned(lhs.into_owned() + &rhs)))
        }
        (Mul, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs * rhs)),
        (Div, Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs / rhs)),
        (op, lhs, rhs) => Err(Error::InvalidBinaryOperation {
            op: op.to_string(),
            lhs: lhs.to_string(),
            rhs: rhs.to_string(),
        }),
    }
}
