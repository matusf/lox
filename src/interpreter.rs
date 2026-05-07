use std::{cell::RefCell, collections::HashMap, fmt::Display, iter, ops::ControlFlow, rc::Rc};

use thiserror::Error;

use crate::parser::{BinOp, Expr, Func, Literal, LogicOp, Statement, UnaryOp};

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Number(f64),
    String(Rc<str>),
    Func {
        name: &'a str,
        args: &'a [&'a str],
        body: &'a [Statement<'a>],
        closure: Rc<Environment<'a>>,
    },
    NativeFunc {
        name: &'a str,
        arity: usize,
        body: fn(&[Value<'a>]) -> Result<Value<'a>, Error>,
    },
    Bool(bool),
    Nil,
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::Func { name, .. } => write!(f, "<fn {name}>"),
            Value::NativeFunc { name, .. } => write!(f, "<fn {name}>"),
        }
    }
}

impl<'a> Value<'a> {
    fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

#[derive(Debug, Error)]
#[error("Runtime error")]
pub enum Error {
    #[error("Mismatched types: expected `{expected}` but got `{got}`")]
    MismachedTypes {
        expected: String,
        got: String,
    },

    #[error("Invalid Unary operation: {op} {value}")]
    InvalidUnaryOperation {
        op: String,
        value: String,
    },

    #[error("Invalid binary operation: {lhs} {op} {rhs}")]
    InvalidBinaryOperation {
        op: String,
        lhs: String,
        rhs: String,
    },

    #[error("Variable `{name}` is not defined")]
    UndefinedVariable {
        name: String,
    },
    ValueNotCallable,
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("Mismatched arity for function `{name}`:  expected {expected} but got `{got}`")]
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
}

#[derive(Debug, Default)]
pub struct Environment<'a> {
    values: RefCell<HashMap<&'a str, Value<'a>>>,
    enclosing: Option<Rc<Environment<'a>>>,
}

impl<'a> Environment<'a> {
    fn from_enclosing(enclosing: Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::default(),
            enclosing: Some(enclosing),
        })
    }

    fn define(&self, name: &'a str, value: Value<'a>) {
        self.values.borrow_mut().insert(name, value);
    }

    fn assign(&self, name: &'a str, value: Value<'a>) -> Result<(), Error> {
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

    fn get(&self, name: &str) -> Result<Value<'a>, Error> {
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

    pub fn with_builtins() -> Self {
        let env = Self::default();
        let name = "clock";
        env.define(
            name,
            Value::NativeFunc {
                name,
                arity: 0,
                body: |_| {
                    Ok(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|t| Value::Number(t.as_secs_f64()))?)
                },
            },
        );

        env
    }
}

pub fn execute<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
    env: Rc<Environment<'a>>,
) -> Result<ControlFlow<Value<'a>>, Error> {
    for statement in statements {
        let env = env.clone();
        match statement {
            Statement::Expr(expr) => {
                eval(expr, env)?;
            }
            Statement::Print(expr) => {
                println!("{}", eval(expr, env)?);
            }
            Statement::VarDecl(name, None) => env.define(name, Value::Nil),
            Statement::VarDecl(name, Some(expr)) => {
                let value = eval(expr, env.clone())?;
                env.define(name, value);
            }
            Statement::Block(statements) => {
                let value = execute(statements.iter(), Environment::from_enclosing(env))?;
                if let Some(return_value) = value.break_value() {
                    return Ok(ControlFlow::Break(return_value));
                };
            }
            Statement::IfElse(condition, yes, no) => {
                let condition = eval(condition, env.clone())?;
                if condition.is_truthy() {
                    let value = execute(iter::once(yes.as_ref()), env)?;
                    if let Some(return_value) = value.break_value() {
                        return Ok(ControlFlow::Break(return_value));
                    };
                } else if let Some(no) = no {
                    let value = execute(iter::once(no.as_ref()), env)?;
                    if let Some(return_value) = value.break_value() {
                        return Ok(ControlFlow::Break(return_value));
                    };
                }
            }
            Statement::While(condition, statement) => {
                while eval(condition, env.clone())?.is_truthy() {
                    let value = execute(iter::once(statement.as_ref()), env.clone())?;
                    if let Some(return_value) = value.break_value() {
                        return Ok(ControlFlow::Break(return_value));
                    };
                }
            }
            Statement::Func { name, args, body } => {
                env.define(
                    name,
                    Value::Func {
                        name,
                        args,
                        body,
                        closure: Environment::from_enclosing(env.clone()),
                    },
                );
            }
            Statement::Return(expr) => {
                let value = eval(expr, env)?;
                return Ok(ControlFlow::Break(value));
            }
        };
    }
    Ok(ControlFlow::Continue(()))
}

pub fn eval<'a>(expr: &Expr<'a>, env: Rc<Environment<'a>>) -> Result<Value<'a>, Error> {
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
            let value = eval(expr, env.clone())?;
            env.assign(name, value.clone())?;
            value
        }
        Expr::LogicOp(op, lhs, rhs) => match op {
            LogicOp::And => {
                let lhs = eval(lhs, env.clone())?;
                if lhs.is_truthy() {
                    eval(rhs, env)?
                } else {
                    lhs
                }
            }
            LogicOp::Or => {
                let lhs = eval(lhs, env.clone())?;
                if lhs.is_truthy() {
                    lhs
                } else {
                    eval(rhs, env)?
                }
            }
        },
        Expr::Call(Func { callee, args }) => {
            let callee = eval(callee, env.clone())?;

            let args: Result<Vec<Value<'_>>, _> =
                args.iter().map(|arg| eval(arg, env.clone())).collect();
            let args = args?;

            match callee {
                Value::Func {
                    name,
                    args: arg_names,
                    body,
                    closure,
                } => {
                    if args.len() != arg_names.len() {
                        return Err(Error::ArityMismatch {
                            name: name.to_string(),
                            expected: arg_names.len(),
                            got: args.len(),
                        });
                    }

                    let env = Environment::from_enclosing(closure);
                    arg_names
                        .iter()
                        .zip(args)
                        .for_each(|(name, arg)| env.define(name, arg));

                    match execute(body.iter(), env)? {
                        ControlFlow::Continue(()) => Value::Nil,
                        ControlFlow::Break(value) => value,
                    }
                }
                Value::NativeFunc { name, arity, body } => {
                    if args.len() != arity {
                        return Err(Error::ArityMismatch {
                            name: name.to_string(),
                            expected: arity,
                            got: args.len(),
                        });
                    }

                    body(&args)?
                }
                _ => Err(Error::ValueNotCallable)?,
            }
        }
    };

    Ok(value)
}

fn eval_unary_op<'a>(
    unary_op: &UnaryOp,
    expr: &Expr<'a>,
    env: Rc<Environment<'a>>,
) -> Result<Value<'a>, Error> {
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
    env: Rc<Environment<'a>>,
) -> Result<Value<'a>, Error> {
    use BinOp::{
        Add, BangEqual, Div, EqualEqual, Greater, GreaterEqual, Less, LessEqual, Mul, Sub,
    };
    let lhs = eval(lhs, env.clone())?;
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
