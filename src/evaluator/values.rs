use crate::ast;

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Func {
    UserFunc {
        args: Vec<ast::Pattern>,
        expr: Rc<ast::Expr>,
    },
    BuiltInFunc(ast::QualifiedLowerName),
}

// TODO: Unsure about making this 'Clone'. Done so that we can have the Value binding without too
// much effort at the moment but seems wrong
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(String),
    List(Vec<Value>),
    PartiallyAppliedFunc { func: Func, values: Vec<Value> },
}
