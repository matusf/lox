use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::interpreter::Value;

#[derive(Debug, Default)]
pub struct Environment<'a> {
    values: RefCell<HashMap<&'a str, Rc<Value<'a>>>>,
    enclosing: Option<Rc<Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub(crate) fn from_enclosing(enclosing: Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::default(),
            enclosing: Some(enclosing),
        })
    }

    pub(crate) fn define(&self, name: &'a str, value: Rc<Value<'a>>) {
        self.values.borrow_mut().insert(name, value);
    }

    pub(crate) fn assign(
        &self,
        name: &'a str,
        value: Rc<Value<'a>>,
        level: Option<usize>,
    ) -> Option<()> {
        let previous = match (level, self.enclosing.clone()) {
            // Write to glabals and current is globals
            (None, None) => self.values.borrow_mut().insert(name, value),
            // Write to globals
            (None, Some(mut env)) => {
                while let Some(e) = &env.enclosing {
                    env = e.clone();
                }
                env.values.borrow_mut().insert(name, value)
            }
            // Write to current
            (Some(0), _) => self.values.borrow_mut().insert(name, value),
            (Some(_), None) => unreachable!(),
            // Write to specified level (-1 because env is set to self.enclosing)
            (Some(level), Some(mut env)) => {
                for _ in 0..(level - 1) {
                    match &env.enclosing {
                        Some(e) => env = e.clone(),
                        None => break,
                    }
                }
                env.values.borrow_mut().insert(name, value)
            }
        };
        previous.map(|_| ())
    }

    pub(crate) fn get(&self, name: &str, level: Option<usize>) -> Option<Rc<Value<'a>>> {
        match (level, self.enclosing.clone()) {
            (None, None) => self.values.borrow().get(name).cloned(),
            (None, Some(mut env)) => {
                while let Some(e) = &env.enclosing {
                    env = e.clone();
                }
                env.values.borrow().get(name).cloned()
            }
            (Some(0), _) => self.values.borrow().get(name).cloned(),
            (Some(_), None) => unreachable!(),
            (Some(level), Some(mut env)) => {
                for _ in 0..(level - 1) {
                    match &env.enclosing {
                        Some(e) => env = e.clone(),
                        None => break,
                    }
                }
                env.values.borrow().get(name).cloned()
            }
        }
    }

    #[must_use]
    pub fn with_globals() -> Self {
        let env = Self::default();

        let name = "clock";
        env.define(
            name,
            Rc::new(Value::NativeFunc {
                name,
                arity: 0,
                body: |_| {
                    Ok(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|t| Rc::new(Value::Number(t.as_secs_f64())))?)
                },
            }),
        );
        env
    }
}

impl Display for Environment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.values)?;
        if let Some(env) = &self.enclosing {
            write!(f, "{env}")?;
        }
        Ok(())
    }
}
