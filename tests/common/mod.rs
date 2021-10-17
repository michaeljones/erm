use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::Buffer;
use unindent::unindent;

use erm::ast;
use erm::checker;
use erm::env;
use erm::evaluater;
use erm::evaluater::values::Value;
use erm::lexer::Range;
use erm::parser;
use erm::project;

pub fn string(str: &str) -> Value {
    Value::String(str.to_string())
}

fn get_range(result: &Result<Value, Error>) -> Option<(String, Range)> {
    match result {
        Err(Error::ParserError(parser::Error::UnexpectedToken { range, .. }, src)) => {
            Some((src.to_string(), range.clone()))
        }
        Err(Error::ParserError(parser::Error::Indent { range }, src)) => {
            Some((src.to_string(), range.clone()))
        }
        _ => None,
    }
}

pub fn pretty_print(result: &Result<Value, Error>) -> String {
    if let Some((src, range)) = get_range(result) {
        let mut files = SimpleFiles::new();
        let file_id = files.add("sample", unindent(&src));
        let diagnostic =
            Diagnostic::error().with_labels(vec![Label::primary(file_id, range.clone())]);

        let mut writer = Buffer::no_color();
        let config = codespan_reporting::term::Config::default();

        let _ = term::emit(&mut writer, &config, &files, &diagnostic);

        std::str::from_utf8(writer.as_slice())
            .unwrap_or("Failure")
            .to_string()
    } else {
        "".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ParserError(parser::Error, String),
    CheckError(checker::Error),
    EvaluateError(evaluater::Error),
    ScopeError(env::Error),
}

pub fn eval(string: &str, settings: Option<project::Settings>) -> Result<Value, Error> {
    eval_with_args(string, Vec::new(), settings)
}

pub fn eval_with_args(
    string: &str,
    args: Vec<String>,
    settings: Option<project::Settings>,
) -> Result<Value, Error> {
    let _ = env_logger::builder().is_test(true).try_init();

    log::trace!("eval_with_args");

    let settings = settings.unwrap_or_else(project::Settings::new);

    let src = unindent(&string);
    let module =
        erm::parse_source(src).map_err(|err| Error::ParserError(err, string.to_string()))?;

    let module = ast::with_default_imports(&module);

    let scope = env::ModuleScope::from_module(&module, &settings).map_err(Error::ScopeError)?;
    let environment = env::Environment::from_module_scope(scope);

    checker::check(&module, &environment, &settings).map_err(Error::CheckError)?;
    evaluater::evaluate(&module, args, &environment, &settings).map_err(Error::EvaluateError)
}
