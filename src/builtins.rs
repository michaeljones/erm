use super::bindings::{Error, Func};
use super::checker::term;
use super::evaluater::values;

// stringFromInt
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

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::Integer)),
            Box::new(term::Term::Constant(term::Value::String)),
        )
    }
}

// stringFromBool
pub struct StringFromBool {}

impl Func for StringFromBool {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 1 {
            return Err(Error::WrongArity);
        }

        match args.first() {
            Some(values::Value::Bool(bool)) => Ok(values::Value::String(bool.to_string())),
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::Bool)),
            Box::new(term::Term::Constant(term::Value::String)),
        )
    }
}

// stringJoin
pub struct StringJoin {}

impl Func for StringJoin {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 1 {
            return Err(Error::WrongArity);
        }

        match args.first() {
            Some(values::Value::List(entries)) => Ok(values::Value::String(
                entries
                    .iter()
                    .flat_map(|value| {
                        if let values::Value::String(string) = value {
                            Some(string.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(""),
            )),
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Type(
                "List".to_string(),
                vec![term::Term::Constant(term::Value::String)],
            )),
            Box::new(term::Term::Constant(term::Value::String)),
        )
    }
}

// Elm.Kernel.Basics.add
pub struct Add {}

impl Func for Add {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 2 {
            return Err(Error::WrongArity);
        }

        match (args.first(), args.last()) {
            (Some(values::Value::Integer(a)), Some(values::Value::Integer(b))) => {
                Ok(values::Value::Integer(a + b))
            }
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::Integer)),
            Box::new(term::Term::Function(
                Box::new(term::Term::Constant(term::Value::Integer)),
                Box::new(term::Term::Constant(term::Value::Integer)),
            )),
        )
    }
}

// Elm.Kernel.Basics.sub
pub struct Sub {}

impl Func for Sub {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 2 {
            return Err(Error::WrongArity);
        }

        match (args.first(), args.last()) {
            (Some(values::Value::Integer(a)), Some(values::Value::Integer(b))) => {
                Ok(values::Value::Integer(a - b))
            }
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::Integer)),
            Box::new(term::Term::Function(
                Box::new(term::Term::Constant(term::Value::Integer)),
                Box::new(term::Term::Constant(term::Value::Integer)),
            )),
        )
    }
}

// Elm.Kernel.Basics.mul
pub struct Mul {}

impl Func for Mul {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 2 {
            return Err(Error::WrongArity);
        }

        match (args.first(), args.last()) {
            (Some(values::Value::Integer(a)), Some(values::Value::Integer(b))) => {
                Ok(values::Value::Integer(a * b))
            }
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::Integer)),
            Box::new(term::Term::Function(
                Box::new(term::Term::Constant(term::Value::Integer)),
                Box::new(term::Term::Constant(term::Value::Integer)),
            )),
        )
    }
}

// Elm.Kernel.Basics.gt
pub struct Gt {}

impl Func for Gt {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 2 {
            return Err(Error::WrongArity);
        }

        match (args.first(), args.last()) {
            (Some(values::Value::Integer(a)), Some(values::Value::Integer(b))) => {
                Ok(values::Value::Bool(a > b))
            }
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::Integer)),
            Box::new(term::Term::Function(
                Box::new(term::Term::Constant(term::Value::Integer)),
                Box::new(term::Term::Constant(term::Value::Bool)),
            )),
        )
    }
}

// Elm.Kernel.Basics.gt
pub struct Lt {}

impl Func for Lt {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 2 {
            return Err(Error::WrongArity);
        }

        match (args.first(), args.last()) {
            (Some(values::Value::Integer(a)), Some(values::Value::Integer(b))) => {
                Ok(values::Value::Bool(a < b))
            }
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::Integer)),
            Box::new(term::Term::Function(
                Box::new(term::Term::Constant(term::Value::Integer)),
                Box::new(term::Term::Constant(term::Value::Bool)),
            )),
        )
    }
}

// Elm.Kernel.Basics.append
pub struct Append {}

impl Func for Append {
    fn call<'a>(&self, args: Vec<values::Value>) -> Result<values::Value, Error> {
        if args.len() != 2 {
            return Err(Error::WrongArity);
        }

        match (args.first(), args.last()) {
            (Some(values::Value::String(a)), Some(values::Value::String(b))) => {
                Ok(values::Value::String(a.to_owned() + b))
            }
            _ => Err(Error::WrongArgumentType),
        }
    }

    fn term(&self) -> term::Term {
        term::Term::Function(
            Box::new(term::Term::Constant(term::Value::String)),
            Box::new(term::Term::Function(
                Box::new(term::Term::Constant(term::Value::String)),
                Box::new(term::Term::Constant(term::Value::String)),
            )),
        )
    }
}
