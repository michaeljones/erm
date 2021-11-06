extern crate clap;
extern crate erm;
extern crate im;
extern crate logos;
extern crate regex;
extern crate walkdir;

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
    filename: &str,
    program_args: Vec<String>,
    settings: erm::project::Settings,
) -> Result<evaluator::values::Value, Error> {
    let mut f = File::open(filename).map_err(|_| Error::FileError)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|_| Error::FileError)?;

    let result = Token::lexer(&contents);
    let mut iter = result.spanned().peekable();

    let module = parser::parse(&mut iter).map_err(Error::ParserError)?;
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
                    .unwrap_or("-".to_string()),
                record.level(),
                record.args()
            )
        })
        .init();
}

fn main() -> () {
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
        .unwrap_or(vec![]);

    let settings = project::Settings::new();

    match matches.value_of("path") {
        Some(path) => {
            let result = std::fs::metadata(path)
                .map_err(|_| Error::FileError)
                .and_then(|attr| {
                    if attr.is_dir() {
                        Err(Error::FileError)
                    } else {
                        run(path, program_args, settings)
                    }
                });

            match result {
                Err(error) => {
                    println!("{}", error::to_user_output(error));
                }
                Ok(value) => {
                    println!("{:?}", value);
                }
            }
        }
        None => {
            println!("Usage: erm [path]");
        }
    }
}
