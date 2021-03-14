use std::rc::Rc;

use super::checker::term;

#[derive(Debug)]
pub struct Module<'src> {
    pub name: &'src str,
    pub exposing: Exposing<'src>,
    pub imports: Vec<Import<'src>>,
    pub statements: Vec<Rc<Stmt<'src>>>,
}

#[derive(Debug)]
pub struct Import<'src> {
    pub module_name: &'src str,
}

#[derive(Debug)]
pub enum Exposing<'a> {
    All,
    List(Vec<ExposingDetail<'a>>),
}

#[derive(Debug)]
pub enum ExposingDetail<'a> {
    Operator(&'a str),
    Name(&'a str),
}

#[derive(Debug)]
pub enum Stmt<'a> {
    Binding {
        name: &'a str,
        expr: Rc<Expr<'a>>,
    },
    Function {
        name: &'a str,
        args: Vec<Pattern<'a>>,
        expr: Rc<Expr<'a>>,
    },
    Infix {
        operator_name: &'a str,
        associativity: Associativity,
        precedence: usize,
        function_name: &'a str,
    },
}

#[derive(Clone, Debug)]
pub enum Associativity {
    Left,
    Right,
    Non,
}

#[derive(Debug)]
pub enum Pattern<'a> {
    Name(&'a str),
}

impl<'a> Pattern<'a> {
    pub fn names(&self) -> Vec<String> {
        match self {
            Pattern::Name(name) => vec![name.to_string()],
        }
    }

    pub fn term(&self) -> term::Term {
        match self {
            Pattern::Name(name) => term::Term::Var(name.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(&'a str),
    List(Vec<Rc<Expr<'a>>>),
    BinOp {
        operator: &'a str,
        left: Rc<Expr<'a>>,
        right: Rc<Expr<'a>>,
    },
    If {
        condition: Rc<Expr<'a>>,
        then_branch: Rc<Expr<'a>>,
        else_branch: Rc<Expr<'a>>,
    },
    Call {
        function_name: &'a str,
        args: Vec<Rc<Expr<'a>>>,
    },
    VarName(&'a str),
}
