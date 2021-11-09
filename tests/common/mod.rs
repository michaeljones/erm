use std::io::Write;

use logos::Logos;
use unindent::unindent;

use erm::ast;
use erm::checker;
use erm::env;
use erm::error::{self, Error};
use erm::evaluator;
use erm::evaluator::values::Value;
use erm::lexer::Token;
use erm::parser;
use erm::project;

fn init_logger() -> Result<(), log::SetLoggerError> {
    env_logger::builder()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(
                buf,
                "{} {}:{} [{}] - {}",
                ts,
                record.file().unwrap_or("unknown"),
                record
                    .line()
                    .map(|num| num.to_string())
                    .unwrap_or("-".to_string()),
                record.level(),
                record.args()
            )
        })
        .is_test(true)
        .try_init()
}

pub fn eval(string: &str, settings: Option<project::Settings>) -> String {
    log::trace!("eval");
    eval_with_args(string, Vec::new(), settings)
}

pub fn eval_with_args(
    string: &str,
    args: Vec<String>,
    settings: Option<project::Settings>,
) -> String {
    log::trace!("eval_with_args");
    let result = eval_it(string, args, settings);

    match result {
        Err(error) => {
            format!("{}", error::to_user_output(error))
        }
        Ok(evaluator::values::Value::String(string)) => {
            format!("{}", string)
        }
        Ok(value) => {
            format!("{:?}", value)
        }
    }
}

fn eval_it(
    string: &str,
    args: Vec<String>,
    settings: Option<project::Settings>,
) -> Result<Value, Error> {
    let _ = init_logger();

    log::trace!("eval_it");

    let settings = settings.unwrap_or_else(project::Settings::new);

    let source = unindent(&string);

    let tokens = Token::lexer(&source);
    let mut iter = tokens.spanned().peekable();
    let module = parser::parse(&mut iter).map_err(|err| Error::ParserError(err, source.clone()))?;

    let module = ast::with_default_imports(&module);

    let scope = env::ModuleScope::from_module(&module, &settings).map_err(Error::ScopeError)?;
    let environment = env::Environment::from_module_scope(scope);

    checker::check(&module, &environment, &settings).map_err(Error::CheckError)?;
    evaluator::evaluate(&module, args, &environment, &settings).map_err(Error::EvaluateError)
}
