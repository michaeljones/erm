use super::checker::term;
use super::evaluater::values;
use super::parser::Stmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    WrongArity,
    WrongArgumentType,
}

pub enum Function<'a> {
    BuiltIn(Box<dyn Func>),
    UserDefined(&'a Stmt<'a>),
}

pub trait Func {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error>;
    fn term(&self) -> Vec<term::Term>;
}

pub struct StringFromInt {}

impl Func for StringFromInt {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 1 {
            return Err(Error::WrongArity);
        }

        match args.first() {
            Some(values::Value::Integer(int)) => Ok(values::Value::String(int.to_string())),
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> Vec<term::Term> {
        vec![
            term::Term::Constant(term::Value::Integer),
            term::Term::Constant(term::Value::String),
        ]
    }
}
