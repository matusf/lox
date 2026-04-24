use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use thiserror::Error;

use crate::parser::{BinOp, Expr, Literal, LogicOp, Statement, UnaryOp};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(Rc<str>),
    Bool(bool),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Value {
    fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
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
    #[error("Variable `{name}` is not defined")]
    UndefinedVariable { name: String },
}

#[derive(Debug, Default, Clone)]
pub struct Environment<'a> {
    values: Rc<RefCell<HashMap<&'a str, Value>>>,
    enclosing: Option<Box<Environment<'a>>>,
}

impl<'a> Environment<'a> {
    fn from_enclosing(enclosing: Self) -> Self {
        Self {
            values: Rc::default(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    fn define(&self, name: &'a str, value: Value) {
        self.values.borrow_mut().insert(name, value);
    }

    fn assign(&self, name: &'a str, value: Value) -> Result<(), Error> {
        if !self.values.borrow().contains_key(name) {
            return self.enclosing.as_ref().map_or(
                Err(Error::UndefinedVariable {
                    name: name.to_string(),
                }),
                |env| env.assign(name, value),
            );
        }
        self.values.borrow_mut().insert(name, value);
        Ok(())
    }
    fn get(&self, name: &str) -> Result<Value, Error> {
        match self.values.borrow().get(name) {
            // NOTE: This clone should be relatively cheap as string is
            // behind Rc and other fields are small
            Some(value) => Ok(value.clone()),
            None => self.enclosing.as_ref().map_or(
                Err(Error::UndefinedVariable {
                    name: name.to_string(),
                }),
                |env| env.get(name),
            ),
        }
    }
}

pub fn execute<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
    env: &mut Environment<'a>,
) -> Result<(), Error> {
    for statement in statements {
        match statement {
            Statement::Expr(expr) => {
                eval(expr, env)?;
            }
            Statement::Print(expr) => {
                println!("{}", eval(expr, env)?);
            }
            Statement::VarDecl(name, None) => env.define(name, Value::Nil),
            Statement::VarDecl(name, Some(expr)) => {
                let value = eval(expr, env)?;
                env.define(name, value);
            }
            Statement::Block(statements) => {
                let mut env = Environment::from_enclosing(env.clone());
                execute(statements.iter(), &mut env)?;
            }
            Statement::IfElse(condition, yes, no) => {
                let condition = eval(condition, env)?;
                if condition.is_truthy() {
                    execute(Some(yes.as_ref()).into_iter(), env)?;
                } else if let Some(no) = no {
                    execute(Some(no.as_ref()).into_iter(), env)?;
                };
            }
            Statement::While(condition, statement) => {
                while eval(condition, env)?.is_truthy() {
                    execute(Some(statement.as_ref()).into_iter(), env)?;
                }
            }
        }
    }
    Ok(())
}

pub fn eval<'a>(expr: &Expr<'a>, env: &mut Environment<'a>) -> Result<Value, Error> {
    let value = match expr {
        Expr::Literal(literal) => match literal {
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Nil => Value::Nil,
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(Rc::from(s.trim_matches('"'))),
            Literal::Identifier(name) => env.get(name)?,
        },
        Expr::Group(expr) => eval(expr, env)?,
        Expr::BinOp(bin_op, lhs, rhs) => eval_bin_op(bin_op, lhs, rhs, env)?,
        Expr::UnaryOp(unary_op, expr) => eval_unary_op(unary_op, expr, env)?,
        Expr::Assign(name, expr) => {
            let value = eval(expr, env)?;
            env.assign(name, value.clone())?;
            value
        }
        Expr::LogicOp(op, lhs, rhs) => match op {
            LogicOp::And => {
                let lhs = eval(lhs, env)?;
                if lhs.is_truthy() {
                    eval(rhs, env)?
                } else {
                    lhs
                }
            }
            LogicOp::Or => {
                let lhs = eval(lhs, env)?;
                if lhs.is_truthy() {
                    lhs
                } else {
                    eval(rhs, env)?
                }
            }
        },
    };

    Ok(value)
}

fn eval_unary_op<'a>(
    unary_op: &UnaryOp,
    expr: &Expr<'a>,
    env: &mut Environment<'a>,
) -> Result<Value, Error> {
    let value = eval(expr, env)?;
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

fn eval_bin_op<'a>(
    bin_op: &BinOp,
    lhs: &Expr<'a>,
    rhs: &Expr<'a>,
    env: &mut Environment<'a>,
) -> Result<Value, Error> {
    use BinOp::{
        Add, BangEqual, Div, EqualEqual, Greater, GreaterEqual, Less, LessEqual, Mul, Sub,
    };
    let lhs = eval(lhs, env)?;
    let rhs = eval(rhs, env)?;
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
            let mut s = String::with_capacity(lhs.len() + rhs.len());
            s.push_str(&lhs);
            s.push_str(&rhs);
            Ok(Value::String(Rc::from(s)))
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
