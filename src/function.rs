use std::rc::Rc;

use super::checker::term;
use super::evaluater::values;
use super::parser::{Expr, Stmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    WrongArity,
    WrongArgumentType,
}

#[derive(Clone)]
pub enum Binding<'src> {
    BuiltInFunc(Rc<dyn Func>),
    UserFunc(Rc<Stmt<'src>>),
    UserBinding(Rc<Expr<'src>>),
    UserArg(term::Term),
}

impl<'src> std::fmt::Debug for Binding<'src> {
    // Implemented because we can't derive Debug for 'dyn Func'
    // TODO: Add more detail
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binding")
    }
}

pub trait Func {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error>;
    fn term(&self) -> term::Term;
}
