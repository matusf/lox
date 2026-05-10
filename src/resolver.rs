use std::{collections::HashMap, iter, ops::ControlFlow};

use crate::parser::{Expr, Func, Literal, Statement};
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
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum FunctionType {
    None,
    Func,
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
        self.resolve(statements, FunctionType::None)?;
        Ok(self.locals)
    }

    fn resolve(
        &mut self,
        statements: impl Iterator<Item = &'a Statement<'a>>,
        current_function: FunctionType,
    ) -> Result<(), Error> {
        for statement in statements {
            match statement {
                Statement::Expr(expr) => self.resolve_expr(expr)?,
                Statement::Print(expr) => self.resolve_expr(expr)?,
                Statement::VarDecl(name, expr) => {
                    self.declare_var(name)?;

                    if let Some(expr) = expr {
                        self.resolve_expr(expr)?
                    }

                    self.define_var(name);
                }
                Statement::Block(statements) => {
                    self.scopes.push(HashMap::new());
                    self.resolve(statements.iter(), current_function)?;
                    self.scopes.pop();
                }
                Statement::IfElse(condition, yes, no) => {
                    self.resolve_expr(condition)?;
                    self.resolve(iter::once(yes.as_ref()), current_function)?;
                    if let Some(no) = no {
                        self.resolve(iter::once(no.as_ref()), current_function)?;
                    }
                }
                Statement::While(condition, body) => {
                    self.resolve_expr(condition)?;
                    self.resolve(iter::once(body.as_ref()), current_function)?;
                }
                Statement::Func { name, args, body } => {
                    self.declare_var(name)?;
                    self.define_var(name);

                    self.scopes.push(HashMap::new());
                    for arg in args {
                        self.declare_var(arg)?;
                        self.define_var(arg);
                    }
                    self.resolve(body.as_ref().iter(), FunctionType::Func)?;
                    self.scopes.pop();
                }
                Statement::Return(expr) => {
                    if current_function != FunctionType::Func {
                        return Err(Error::ReturnOutsideOfFunction);
                    }
                    self.resolve_expr(expr)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expr: &'a Expr<'a>) -> Result<(), Error> {
        match expr {
            Expr::Literal(literal) => {
                if let Literal::Identifier { name, id } = *literal {
                    if let Some(scope) = self.scopes.last()
                        && scope.get(name) == Some(&VariableState::Declared)
                    {
                        return Err(Error::ReadingLocalVariableInOwnInitializer);
                    }
                    self.resolve_local(name, id);
                }
            }
            Expr::Group(expr) => {
                self.resolve_expr(expr)?;
            }
            Expr::BinOp(_, lhs, rhs) => {
                self.resolve_expr(lhs)?;
                self.resolve_expr(rhs)?;
            }
            Expr::UnaryOp(_, expr) => {
                self.resolve_expr(expr)?;
            }
            Expr::Assign { name, expr, id } => {
                self.resolve_expr(expr)?;
                self.resolve_local(name, *id);
            }
            Expr::LogicOp(_, lhs, rhs) => {
                self.resolve_expr(lhs)?;
                self.resolve_expr(rhs)?;
            }
            Expr::Call(Func { callee, args }) => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }
        };

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
