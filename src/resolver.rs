use std::{collections::HashMap, iter, ops::ControlFlow};

use crate::parser::{Expr, Func, FuncDecl, Identifier, Literal, Statement};
use thiserror::Error;

#[derive(Debug, PartialEq)]
enum VariableState {
    Declared,
    Defined,
}

#[derive(Debug, Error)]
#[error("Resolver error")]
pub enum Error {
    #[error("Can't read local variable in its own initializer")]
    ReadingLocalVariableInOwnInitializer,
    #[error("Already a variable with this name in this scope")]
    VariableRedeclarationInSameScope,
    #[error("Can't return from top-level code")]
    ReturnOutsideOfFunction,
    #[error("Can't use `this` outside of a class")]
    ThisOutsideOfClass,
    #[error("Can't return a value from an initializer.")]
    ReturnInInitializer,
    #[error("A class can't inherit from itself")]
    InheritanceFromSelf,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum CallableType {
    None,
    Func,
    Initializer,
    Method,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ClassType {
    None,
    Class,
}

#[derive(Debug, Default)]
pub struct Resolver<'a> {
    locals: HashMap<usize, usize>,
    scopes: Vec<HashMap<&'a str, VariableState>>,
}

impl<'a> Resolver<'a> {
    pub fn run(
        mut self,
        statements: impl Iterator<Item = &'a Statement<'a>>,
    ) -> Result<HashMap<usize, usize>, Error> {
        self.resolve(statements, CallableType::None, ClassType::None)?;
        Ok(self.locals)
    }

    fn resolve(
        &mut self,
        statements: impl Iterator<Item = &'a Statement<'a>>,
        callable_type: CallableType,
        class_type: ClassType,
    ) -> Result<(), Error> {
        for statement in statements {
            match statement {
                Statement::Expr(expr) => self.resolve_expr(expr, class_type)?,
                Statement::Print(expr) => self.resolve_expr(expr, class_type)?,
                Statement::VarDecl(name, expr) => {
                    self.declare_var(name)?;

                    if let Some(expr) = expr {
                        self.resolve_expr(expr, class_type)?
                    }

                    self.define_var(name);
                }
                Statement::Block(statements) => {
                    self.scopes.push(HashMap::new());
                    self.resolve(statements.iter(), callable_type, class_type)?;
                    self.scopes.pop();
                }
                Statement::IfElse(condition, yes, no) => {
                    self.resolve_expr(condition, class_type)?;
                    self.resolve(iter::once(yes.as_ref()), callable_type, class_type)?;
                    if let Some(no) = no {
                        self.resolve(iter::once(no.as_ref()), callable_type, class_type)?;
                    }
                }
                Statement::While(condition, body) => {
                    self.resolve_expr(condition, class_type)?;
                    self.resolve(iter::once(body.as_ref()), callable_type, class_type)?;
                }
                Statement::FuncDecl(func_decl) => {
                    self.resolve_func_decl(func_decl, CallableType::Func, class_type)?
                }
                Statement::Return(expr) => {
                    if !matches!(
                        callable_type,
                        CallableType::Func | CallableType::Method | CallableType::Initializer
                    ) {
                        return Err(Error::ReturnOutsideOfFunction);
                    }

                    if callable_type == CallableType::Initializer
                        && !matches!(expr, Expr::Literal(Literal::Nil))
                    {
                        return Err(Error::ReturnInInitializer);
                    }
                    self.resolve_expr(expr, class_type)?;
                }
                Statement::ClassDecl {
                    name,
                    methods,
                    super_class,
                } => {
                    self.declare_var(name)?;
                    self.define_var(name);

                    if let Some(super_class) = super_class {
                        if super_class.name == *name {
                            return Err(Error::InheritanceFromSelf);
                        }

                        self.resolve_identifier(super_class)?;
                    }

                    self.scopes.push(HashMap::new());
                    self.define_var("this");

                    methods.iter().try_for_each(|func_decl| {
                        let callable_type = if func_decl.name == "init" {
                            CallableType::Initializer
                        } else {
                            CallableType::Method
                        };
                        self.resolve_func_decl(func_decl, callable_type, ClassType::Class)
                    })?;

                    self.scopes.pop();
                }
            }
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expr: &'a Expr<'a>, class_type: ClassType) -> Result<(), Error> {
        match expr {
            Expr::Literal(literal) => {
                if let Literal::Identifier(identifier) = literal {
                    self.resolve_identifier(identifier)?;
                }
            }
            Expr::Group(expr) => {
                self.resolve_expr(expr, class_type)?;
            }
            Expr::BinOp(_, lhs, rhs) => {
                self.resolve_expr(lhs, class_type)?;
                self.resolve_expr(rhs, class_type)?;
            }
            Expr::UnaryOp(_, expr) => {
                self.resolve_expr(expr, class_type)?;
            }
            Expr::Assign { name, expr, id } => {
                self.resolve_expr(expr, class_type)?;
                self.resolve_local(name, *id);
            }
            Expr::LogicOp(_, lhs, rhs) => {
                self.resolve_expr(lhs, class_type)?;
                self.resolve_expr(rhs, class_type)?;
            }
            Expr::Call(Func { callee, args }) => {
                self.resolve_expr(callee, class_type)?;
                for arg in args {
                    self.resolve_expr(arg, class_type)?;
                }
            }
            Expr::Get { expr, .. } => {
                self.resolve_expr(expr, class_type)?;
            }
            Expr::Set { expr, value, .. } => {
                self.resolve_expr(expr, class_type)?;
                self.resolve_expr(value, class_type)?;
            }
            Expr::This { id } => {
                if class_type != ClassType::Class {
                    return Err(Error::ThisOutsideOfClass);
                }
                self.resolve_local("this", *id)
            }
        };

        Ok(())
    }

    fn resolve_identifier(&mut self, Identifier { name, id }: &'a Identifier) -> Result<(), Error> {
        if let Some(scope) = self.scopes.last()
            && scope.get(name) == Some(&VariableState::Declared)
        {
            return Err(Error::ReadingLocalVariableInOwnInitializer);
        }
        self.resolve_local(name, *id);
        Ok(())
    }

    fn resolve_func_decl(
        &mut self,
        FuncDecl { name, args, body }: &'a FuncDecl<'a>,
        callable_type: CallableType,
        class_type: ClassType,
    ) -> Result<(), Error> {
        self.declare_var(name)?;
        self.define_var(name);

        self.scopes.push(HashMap::new());
        for arg in args {
            self.declare_var(arg)?;
            self.define_var(arg);
        }
        self.resolve(body.as_ref().iter(), callable_type, class_type)?;
        self.scopes.pop();
        Ok(())
    }

    fn declare_var(&mut self, name: &'a str) -> Result<(), Error> {
        if let Some(scope) = self.scopes.last_mut() {
            // If the variable is already declared, return error
            if scope.insert(name, VariableState::Declared).is_some() {
                return Err(Error::VariableRedeclarationInSameScope);
            };
        };

        Ok(())
    }

    fn define_var(&mut self, name: &'a str) {
        self.scopes
            .last_mut()
            .map(|scope| scope.insert(name, VariableState::Defined));
    }

    fn resolve_local(&mut self, name: &'a str, id: usize) {
        let _ = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .try_for_each(|(i, scope)| {
                if scope.contains_key(name) {
                    self.locals.insert(id, i);
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            });
    }
}
