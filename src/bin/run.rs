use clap::{App, Arg};
use logos::Logos;

use std::fs::File;
use std::io::prelude::*;

use erm::env;
use erm::error::{self, Error};
use erm::evaluator;
use erm::lexer::Token;
use erm::parser;
use erm::project;

fn run(
    contents: String,
    program_args: Vec<String>,
    settings: erm::project::Settings,
) -> Result<evaluator::values::Value, Error> {
    let result = Token::lexer(&contents);
    let mut iter = result.spanned().peekable();

    let module =
        parser::parse(&mut iter).map_err(|err| Error::ParserError(err, contents.clone()))?;
    let module = erm::ast::with_default_imports(&module);
    let scope = env::ModuleScope::from_module(&module, &settings).map_err(Error::ScopeError)?;
    let environment = env::Environment::from_module_scope(scope);

    evaluator::evaluate(&module, program_args, &environment, &settings)
        .map_err(Error::EvaluateError)
}

fn init_logger() {
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
                    .unwrap_or_else(|| "-".to_string()),
                record.level(),
                record.args()
            )
        })
        .init();
}

fn filter_hash_bang(code: String) -> String {
    code.split('\n')
        .enumerate()
        // Remove the first line if it starts with #!
        .filter(|(index, line)| !(*index == 0 && line.starts_with("#!")))
        .map(|(_, line)| line)
        .collect()
}

fn main() {
    // Set up logger
    init_logger();

    // Parse command line args
    let matches = App::new("erm")
        .version("0.1")
        .arg(Arg::with_name("path").index(1))
        .arg(Arg::with_name("arguments").multiple(true))
        .get_matches();

    let program_args: Vec<String> = matches
        .values_of("arguments")
        .map(|values| values.map(|value| value.to_string()).collect())
        .unwrap_or_else(Vec::new);

    let settings = project::Settings::new();

    let contents_result = matches
        .value_of("path")
        // Treat '-' as no argument so we default to standardin
        .and_then(|path| if path == "-" { None } else { Some(path) })
        .map_or_else(
            // If we don't have a path from the args
            || {
                let mut input = String::new();

                std::io::stdin()
                    .read_to_string(&mut input)
                    .map_err(|_| Error::FileError)?;

                Ok(input)
            },
            // If we have a path from the args
            |path| {
                std::fs::metadata(path)
                    .map_err(|_| Error::FileError)
                    .and_then(|attr| {
                        if attr.is_dir() {
                            Err(Error::FileError)
                        } else {
                            let mut f = File::open(path).map_err(|_| Error::FileError)?;

                            let mut contents = String::new();
                            f.read_to_string(&mut contents)
                                .map_err(|_| Error::FileError)?;

                            Ok(filter_hash_bang(contents))
                        }
                    })
            },
        );

    let result = contents_result.and_then(|contents| run(contents, program_args, settings));

    match result {
        Err(error) => {
            println!("{}", error::to_user_output(error));
        }
        Ok(evaluator::values::Value::String(string)) => {
            println!("{}", string);
        }
        Ok(value) => {
            println!("{:?}", value);
        }
    }
}
