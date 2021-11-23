use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::Buffer;

use crate::checker;
use crate::env;
use crate::evaluator;
use crate::lexer::Range;
use crate::parser;

#[derive(Debug)]
pub enum Error {
    FileError,
    ParserError(parser::Error, String),
    CheckError(checker::Error),
    EvaluateError(evaluator::Error),
    ScopeError(env::Error),
}

pub fn to_user_output(error: Error) -> String {
    match error {
        Error::FileError => "File error".to_string(),
        Error::ParserError(error, source) => match error {
            parser::Error::UnexpectedToken {
                expected: _,
                found: _,
                range,
            } => format!(
                r#"Unexpected token.

{}"#,
                pretty_print(source, range)
            ),
            parser::Error::UnexpectedEnd => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::Indent { range } => format!(
                r#"Unexpected indentation.

{}"#,
                pretty_print(source, range)
            ),
            parser::Error::TokensRemaining(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::NoOperand => format!("Error text not written ({}) {:?}", line!(), error),
            parser::Error::NoOperator => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::EmptyOperatorStack => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::UnknownOperator(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::UnknownExposing(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::NegativePrecendence => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::Unknown => format!("Error text not written ({}) {:?}", line!(), error),
            parser::Error::NameMismatch => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            parser::Error::TokenNotAtLineStart(range) => format!(
                r#"Token not at line start.

{}"#,
                pretty_print(source, range)
            ),
        },
        Error::CheckError(error) => match error {
            checker::Error::UnknownBinding(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::UnhandledExpression(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::UnifyError(unify_error) => format!(
                r#"Type error:

{:#?}"#,
                unify_error
            ),
            checker::Error::UnknownFunction(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::UnknownOperator(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::UnknownVarName(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::UnknownPattern(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::ArgumentMismatch(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::TooManyArguments => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::Broken(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::ScopeError(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::ImpossiblyEmptyList => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::ImpossiblyEmptyCase => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            checker::Error::Unknown => format!("Error text not written ({}) {:?}", line!(), error),
        },
        Error::EvaluateError(error) => match error {
            evaluator::Error::UnsupportedOperation => {
                format!("Error text not written ({})", line!())
            }
            evaluator::Error::UnknownFunction => "Unable to find function".to_string(),
            evaluator::Error::UnknownBinding(name) => format!("Unknown binding: {}", name),
            evaluator::Error::FunctionError(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            evaluator::Error::WrongArity => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            evaluator::Error::TooManyArguments => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            evaluator::Error::ScopeError(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            evaluator::Error::UnexpectedBinding(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            evaluator::Error::UnsupportedArgumentPattern(_) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
            evaluator::Error::NoMatchingCase => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
        },
        Error::ScopeError(error) => match error {
            env::Error::UnableToFindModule(module) => format!(
                "Unable to find module:

{}",
                module
            ),
            env::Error::FailedToRead(_) => format!("Error text not written ({})", line!()),
            env::Error::FailedToParse(_, _) => {
                format!("Error text not written ({}) {:?}", line!(), error)
            }
        },
    }
}

pub fn pretty_print(source: String, range: Range) -> String {
    let mut files = SimpleFiles::new();
    let file_id = files.add("sample", source);
    let diagnostic = Diagnostic::error().with_labels(vec![Label::primary(file_id, range)]);

    let mut writer = Buffer::no_color();
    let config = codespan_reporting::term::Config::default();

    let _ = term::emit(&mut writer, &config, &files, &diagnostic);

    std::str::from_utf8(writer.as_slice())
        .unwrap_or("Failure")
        .to_string()
}
