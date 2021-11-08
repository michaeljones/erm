use crate::env;
use crate::evaluator;
use crate::parser;

#[derive(Debug)]
pub enum Error {
    FileError,
    ParserError(parser::Error),
    EvaluateError(evaluator::Error),
    ScopeError(env::Error),
}

pub fn to_user_output(error: Error) -> String {
    match error {
        Error::FileError => "File error".to_string(),
        Error::ParserError(error) => match error {
            parser::Error::UnexpectedToken {
                expected,
                found,
                range: _,
                line: _,
            } => format!(
                r#"Unexpected token.

Found: {}
Expected: {}

                "#,
                found, expected
            ),
            parser::Error::UnexpectedEnd => format!("Error text not written ({})", line!()),
            parser::Error::Indent { range: _ } => format!("Error text not written ({})", line!()),
            parser::Error::TokensRemaining(_) => format!("Error text not written ({})", line!()),
            parser::Error::NoOperand => format!("Error text not written ({})", line!()),
            parser::Error::NoOperator => format!("Error text not written ({})", line!()),
            parser::Error::EmptyOperatorStack => format!("Error text not written ({})", line!()),
            parser::Error::UnknownOperator(_) => format!("Error text not written ({})", line!()),
            parser::Error::UnknownExposing(_) => format!("Error text not written ({})", line!()),
            parser::Error::NegativePrecendence => format!("Error text not written ({})", line!()),
        },
        Error::EvaluateError(error) => match error {
            evaluator::Error::UnsupportedOperation => {
                format!("Error text not written ({})", line!())
            }
            evaluator::Error::UnknownFunction(name) => {
                format!("Unable to find function: {}", name)
            }
            evaluator::Error::UnknownBinding(name) => format!("Unknown binding: {}", name),
            evaluator::Error::FunctionError(_) => format!("Error text not written ({})", line!()),
            evaluator::Error::WrongArity => format!("Error text not written ({})", line!()),
            evaluator::Error::ScopeError(_) => format!("Error text not written ({})", line!()),
            evaluator::Error::UnexpectedBinding(_) => {
                format!("Error text not written ({})", line!())
            }
        },
        Error::ScopeError(error) => match error {
            env::Error::UnableToFindModule(_) => format!("Error text not written ({})", line!()),
            env::Error::FailedToRead(_) => format!("Error text not written ({})", line!()),
            env::Error::FailedToParse(_, _) => format!("Error text not written ({})", line!()),
        },
    }
}
