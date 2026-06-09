use std::{cell::RefCell, collections::HashMap, fmt::Display, iter, ops::ControlFlow, rc::Rc};

use thiserror::Error;

use crate::{
    interpreter::environment::Environment,
    parser::{
        BinOp, Expr, ExprId, Func, FuncDecl, Identifier, Literal, LogicOp, Statement, UnaryOp,
    },
};

pub mod environment;

#[derive(Debug)]
pub struct Class<'a> {
    name: &'a str,
    super_class: Option<Rc<Class<'a>>>,
    methods: HashMap<&'a str, Function<'a>>,
}

impl<'a> Class<'a> {
    fn find_method(&self, name: &str) -> Option<Function<'a>> {
        self.methods
            .get(name)
            .cloned()
            .or_else(|| self.super_class.as_ref().and_then(|c| c.find_method(name)))
    }
}

#[derive(Debug, Clone)]
pub struct Function<'a> {
    name: &'a str,
    args: &'a [&'a str],
    body: &'a [Statement<'a>],
    closure: Rc<Environment<'a>>,
    is_initializer: bool,
}

impl<'a> Function<'a> {
    fn bind(mut self, value: Rc<Value<'a>>) -> Self {
        let env = Environment::from_enclosing(self.closure);
        env.define("this", value);
        self.closure = env;
        self
    }

    fn call(
        &self,
        args: Vec<Rc<Value<'a>>>,
        interpreter: &Interpreter,
    ) -> Result<Rc<Value<'a>>, Error> {
        if args.len() != self.args.len() {
            return Err(Error::ArityMismatch {
                name: self.name.to_string(),
                expected: self.args.len(),
                got: args.len(),
            });
        }

        let env = Environment::from_enclosing(self.closure.clone());
        self.args
            .iter()
            .zip(args)
            .for_each(|(name, arg)| env.define(name, arg));

        let value = match interpreter.execute(self.body.iter(), &env)? {
            ControlFlow::Continue(()) => Rc::new(Value::Nil),
            ControlFlow::Break(value) => value,
        };

        if self.is_initializer {
            self.closure
                .get("this", Some(0))
                .ok_or_else(|| Error::undefined_variable("this"))
        } else {
            Ok(value)
        }
    }
}

impl<'a> FuncDecl<'a> {
    fn into(&'a self, env: &Rc<Environment<'a>>, is_initializer: bool) -> Function<'a> {
        Function {
            name: self.name,
            args: &self.args,
            body: &self.body,
            closure: env.clone(),
            is_initializer,
        }
    }
}

#[derive(Debug)]
pub enum Value<'a> {
    Number(f64),
    String(Rc<str>),
    Func(Function<'a>),
    NativeFunc {
        name: &'a str,
        arity: usize,
        body: fn(&[ValueRef<'a>]) -> Result<ValueRef<'a>, Error>,
    },
    Class(Rc<Class<'a>>),
    Instance {
        class: Rc<Class<'a>>,
        fields: RefCell<HashMap<&'a str, ValueRef<'a>>>,
    },
    Bool(bool),
    Nil,
}

type ValueRef<'a> = Rc<Value<'a>>;

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::Func(Function { name, .. }) | Value::NativeFunc { name, .. } => {
                write!(f, "<fn {name}>")
            }
            Value::Class(class) => write!(f, "{}", class.name),
            Value::Instance { class, .. } => write!(f, "{} instance", class.name),
        }
    }
}

impl Value<'_> {
    const fn is_truthy(&self) -> bool {
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

    #[error("Only instances have properties")]
    NoProperties,

    #[error("Property `{name}` not found")]
    MissingProperty {
        name: String,
    },
    #[error("Superclass `{name}` is not of type class")]
    SuperClassNotAClass {
        name: String,
    },
}

impl Error {
    fn undefined_variable(name: &str) -> Self {
        Self::UndefinedVariable {
            name: name.to_string(),
        }
    }
}

pub struct Interpreter {
    locals: HashMap<ExprId, usize>,
}

impl Interpreter {
    #[must_use]
    pub const fn new(locals: HashMap<ExprId, usize>) -> Self {
        Self { locals }
    }

    pub fn execute<'a>(
        &self,
        statements: impl Iterator<Item = &'a Statement<'a>>,
        env: &Rc<Environment<'a>>,
    ) -> Result<ControlFlow<Rc<Value<'a>>>, Error> {
        for statement in statements {
            let env = env.clone();
            match statement {
                Statement::Expr(expr) => {
                    self.eval(expr, env)?;
                }
                Statement::Print(expr) => {
                    println!("{}", self.eval(expr, env)?);
                }
                Statement::VarDecl(name, None) => env.define(name, Rc::new(Value::Nil)),
                Statement::VarDecl(name, Some(expr)) => {
                    let value = self.eval(expr, env.clone())?;
                    env.define(name, value);
                }
                Statement::Block(statements) => {
                    let value =
                        self.execute(statements.iter(), &Environment::from_enclosing(env))?;
                    if let Some(return_value) = value.break_value() {
                        return Ok(ControlFlow::Break(return_value));
                    }
                }
                Statement::IfElse(condition, yes, no) => {
                    let condition = self.eval(condition, env.clone())?;
                    if condition.is_truthy() {
                        let value = self.execute(iter::once(yes.as_ref()), &env)?;
                        if let Some(return_value) = value.break_value() {
                            return Ok(ControlFlow::Break(return_value));
                        }
                    } else if let Some(no) = no {
                        let value = self.execute(iter::once(no.as_ref()), &env)?;
                        if let Some(return_value) = value.break_value() {
                            return Ok(ControlFlow::Break(return_value));
                        }
                    }
                }
                Statement::While(condition, statement) => {
                    while self.eval(condition, env.clone())?.is_truthy() {
                        let value = self.execute(iter::once(statement.as_ref()), &env)?;
                        if let Some(return_value) = value.break_value() {
                            return Ok(ControlFlow::Break(return_value));
                        }
                    }
                }
                Statement::FuncDecl(f) => {
                    let func = Value::Func(f.into(&env, false));
                    env.define(f.name, Rc::new(func));
                }
                Statement::Return(expr) => {
                    let value = self.eval(expr, env)?;
                    return Ok(ControlFlow::Break(value));
                }
                Statement::ClassDecl {
                    name,
                    methods,
                    super_class,
                } => {
                    // Two step definition to allow referencing class from the methods
                    env.define(name, Rc::new(Value::Nil));

                    let (super_class, class_env) = match super_class {
                        Some(identifier) => {
                            let value = env
                                .get(identifier.name, self.locals.get(&identifier.id).copied())
                                .ok_or_else(|| Error::undefined_variable(identifier.name))?;

                            let Value::Class(super_class) = value.as_ref() else {
                                return Err(Error::SuperClassNotAClass {
                                    name: identifier.name.to_string(),
                                });
                            };
                            let class_env = Environment::from_enclosing(env.clone());
                            class_env.define("super", value.clone());

                            (Some(super_class.clone()), class_env)
                        }
                        None => (None, env.clone()),
                    };

                    let methods: HashMap<&str, Function<'_>> = methods
                        .iter()
                        .map(|f| (f.name, f.into(&class_env, f.name == "init")))
                        .collect();

                    let class = Value::Class(Rc::new(Class {
                        name,
                        super_class,
                        methods,
                    }));
                    env.assign(name, Rc::new(class), Some(0))
                        .ok_or_else(|| Error::undefined_variable(name))?;
                }
            }
        }
        Ok(ControlFlow::Continue(()))
    }

    pub fn eval<'a>(
        &self,
        expr: &Expr<'a>,
        env: Rc<Environment<'a>>,
    ) -> Result<Rc<Value<'a>>, Error> {
        let value = match expr {
            Expr::Literal(literal) => match literal {
                Literal::Bool(b) => Rc::new(Value::Bool(*b)),
                Literal::Nil => Rc::new(Value::Nil),
                Literal::Number(n) => Rc::new(Value::Number(*n)),
                Literal::String(s) => Rc::new(Value::String(Rc::from(s.trim_matches('"')))),
                Literal::Identifier(Identifier { name, id }) => env
                    .get(name, self.locals.get(id).copied())
                    .ok_or_else(|| Error::undefined_variable(name))?,
            },
            Expr::Group(expr) => self.eval(expr, env)?,
            Expr::BinOp(bin_op, lhs, rhs) => self.eval_bin_op(*bin_op, lhs, rhs, env)?,
            Expr::UnaryOp(unary_op, expr) => self.eval_unary_op(*unary_op, expr, env)?,
            Expr::Assign { name, expr, id } => {
                let value = self.eval(expr, env.clone())?;
                env.assign(name, value.clone(), self.locals.get(id).copied())
                    .ok_or_else(|| Error::undefined_variable(name))?;
                value
            }
            Expr::LogicOp(op, lhs, rhs) => match op {
                LogicOp::And => {
                    let lhs = self.eval(lhs, env.clone())?;
                    if lhs.is_truthy() {
                        self.eval(rhs, env)?
                    } else {
                        lhs
                    }
                }
                LogicOp::Or => {
                    let lhs = self.eval(lhs, env.clone())?;
                    if lhs.is_truthy() {
                        lhs
                    } else {
                        self.eval(rhs, env)?
                    }
                }
            },
            Expr::Call(Func { callee, args }) => {
                let callee = self.eval(callee, env.clone())?;

                let args: Result<Vec<Rc<Value<'_>>>, _> =
                    args.iter().map(|arg| self.eval(arg, env.clone())).collect();
                let args = args?;

                match callee.as_ref() {
                    Value::Func(function) => function.call(args, self)?,
                    Value::NativeFunc { name, arity, body } => {
                        if args.len() != *arity {
                            return Err(Error::ArityMismatch {
                                name: name.to_string(),
                                expected: *arity,
                                got: args.len(),
                            });
                        }

                        body(&args)?
                    }
                    Value::Class(class) => {
                        let instance = Rc::new(Value::Instance {
                            class: class.clone(),
                            fields: RefCell::default(),
                        });

                        // Call initializer
                        class
                            .find_method("init")
                            .map(|f| f.clone().bind(instance.clone()).call(args, self));

                        instance
                    }
                    _ => Err(Error::ValueNotCallable)?,
                }
            }
            Expr::Get { expr, name } => {
                let this = self.eval(expr, env)?;
                let Value::Instance { class, fields } = this.as_ref() else {
                    return Err(Error::NoProperties);
                };

                fields
                    .borrow()
                    .get(name)
                    .cloned()
                    .or_else(|| {
                        class
                            .find_method(name)
                            .map(|f| Rc::new(Value::Func(f.clone().bind(this.clone()))))
                    })
                    .ok_or_else(|| Error::MissingProperty {
                        name: name.to_string(),
                    })?
            }
            Expr::Set { expr, name, value } => {
                let getter = self.eval(expr, env.clone())?;
                let Value::Instance { fields, .. } = getter.as_ref() else {
                    return Err(Error::NoProperties);
                };
                let value = self.eval(value, env)?;
                fields.borrow_mut().insert(*name, value.clone());
                value
            }
            Expr::This { id } => env
                .get("this", self.locals.get(id).copied())
                .ok_or_else(|| Error::undefined_variable("this"))?,
            Expr::Super(Identifier { name, id }) => {
                let super_class = env
                    .get("super", self.locals.get(id).copied())
                    .ok_or_else(|| Error::undefined_variable("super"))?;
                let this = env
                    .get("this", self.locals.get(id).map(|i| i - 1))
                    .ok_or_else(|| Error::undefined_variable("this"))?;

                let Value::Class(class) = super_class.as_ref() else {
                    return Err(Error::SuperClassNotAClass {
                        name: super_class.to_string(),
                    });
                };

                class
                    .find_method(name)
                    .map(|f| Rc::new(Value::Func(f.clone().bind(this.clone()))))
                    .ok_or_else(|| Error::MissingProperty {
                        name: name.to_string(),
                    })?
            }
        };

        Ok(value)
    }

    fn eval_unary_op<'a>(
        &self,
        unary_op: UnaryOp,
        expr: &Expr<'a>,
        env: Rc<Environment<'a>>,
    ) -> Result<Rc<Value<'a>>, Error> {
        let value = self.eval(expr, env)?;
        let value = match (&unary_op, value.as_ref()) {
            (UnaryOp::Negate, Value::Bool(b)) => Value::Bool(!b),
            (UnaryOp::Negate, Value::Nil) => Value::Bool(true),
            (UnaryOp::Negate, _) => Value::Bool(false),
            (UnaryOp::Minus, Value::Number(n)) => Value::Number(-n),
            (_, _) => {
                return Err(Error::InvalidUnaryOperation {
                    op: unary_op.to_string(),
                    value: value.to_string(),
                });
            }
        };
        Ok(Rc::new(value))
    }

    fn eval_bin_op<'a>(
        &self,
        bin_op: BinOp,
        lhs: &Expr<'a>,
        rhs: &Expr<'a>,
        env: Rc<Environment<'a>>,
    ) -> Result<Rc<Value<'a>>, Error> {
        use BinOp::{
            Add, BangEqual, Div, EqualEqual, Greater, GreaterEqual, Less, LessEqual, Mul, Sub,
        };
        let lhs = self.eval(lhs, env.clone())?;
        let rhs = self.eval(rhs, env)?;
        let value = match (bin_op, lhs.as_ref(), rhs.as_ref()) {
            (BangEqual, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs != rhs),
            (BangEqual, Value::String(lhs), Value::String(rhs)) => Value::Bool(lhs != rhs),
            (BangEqual, Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs != rhs),
            (BangEqual, Value::Nil, Value::Nil) => Value::Bool(false),
            (BangEqual, _, _) => Value::Bool(true),

            (EqualEqual, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs == rhs),
            (EqualEqual, Value::String(lhs), Value::String(rhs)) => Value::Bool(lhs == rhs),
            (EqualEqual, Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs == rhs),
            (EqualEqual, Value::Nil, Value::Nil) => Value::Bool(true),
            (EqualEqual, _, _) => Value::Bool(false),

            (Greater, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs > rhs),
            (GreaterEqual, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs >= rhs),
            (Less, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs < rhs),
            (LessEqual, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs <= rhs),
            (Sub, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs - rhs),
            (Add, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
            (Add, Value::String(lhs), Value::String(rhs)) => {
                let mut s = String::with_capacity(lhs.len() + rhs.len());
                s.push_str(lhs);
                s.push_str(rhs);
                Value::String(Rc::from(s))
            }
            (Mul, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs * rhs),
            (Div, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs / rhs),
            (op, lhs, rhs) => {
                return Err(Error::InvalidBinaryOperation {
                    op: op.to_string(),
                    lhs: lhs.to_string(),
                    rhs: rhs.to_string(),
                });
            }
        };
        Ok(Rc::new(value))
    }
}
