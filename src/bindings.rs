use std::rc::Rc;

use super::ast::{Expr, Stmt};
use super::checker::term;
use super::evaluator::values;

#[derive(Debug, Clone)]
pub enum Binding {
    // Represents a binding of a name to function statement
    UserFunc(Rc<Stmt>),
    // Represents a binding of a name to a simple expression (ie. no arguments involved.)
    UserBinding(Rc<Expr>),
    // TODO: Feels wrong to have a 'term ' in here with other things
    UserArg(term::Term),
    // TODO: Unsure about this entry especially as it means we need to make Value 'Clone' which
    // seems bad
    Value(values::Value),
}
