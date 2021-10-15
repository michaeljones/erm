use std::rc::Rc;

use super::ast::{Expr, Stmt};
use super::checker::term;
use super::evaluater::values;

#[derive(Debug, PartialEq)]
pub enum Error {
    WrongArity,
    WrongArgumentType,
}

#[derive(Clone)]
pub enum Binding {
    BuiltInFunc(Rc<dyn Func>),
    UserFunc(Rc<Stmt>),
    UserBinding(Rc<Expr>),
    // TODO: Feels wrong to have a 'term ' in here with other things
    UserArg(term::Term),
    // TODO: Unsure about this entry especially as it means we need to make Value 'Clone' which
    // seems bad
    Value(values::Value),
}

impl std::fmt::Debug for Binding {
    // Implemented because we can't derive Debug for 'dyn Func'
    // TODO: Add more detail
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::BuiltInFunc(_) => write!(f, "Binding::BuiltInFunc"),
            Binding::UserFunc(_) => write!(f, "Binding::UserFunc"),
            Binding::UserBinding(expr_rc) => {
                write!(f, "{}", format!("Binding::UserBinding: {:?}", expr_rc))
            }
            Binding::UserArg(_) => write!(f, "Binding::UserArg"),
            Binding::Value(_) => write!(f, "Binding::Value"),
        }
    }
}

pub trait Func {
    fn call(&self, args: Vec<values::Value>) -> Result<values::Value, Error>;
    fn term(&self) -> term::Term;
}
