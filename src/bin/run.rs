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
use erm::evaluater;
use erm::lexer::Token;
use erm::parser;
use erm::project;

#[derive(Debug)]
enum Error {
    FileError,
    ParserError,
    EvaluateError,
    ScopeError(env::Error),
}

fn dump_file(filename: &str, _quiet: bool) -> Result<(), Error> {
    let mut f = File::open(filename).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let result = Token::lexer(&contents);
    let mut iter = result.spanned().peekable();

    let module = parser::parse(&mut iter).map_err(|_| Error::ParserError)?;

    let args = vec!["example_arg".to_string()];

    let basics = erm::parse_basics().map_err(|_| Error::ParserError)?;
    let settings = project::Settings::new();
    let scope = env::ModuleScope::from_module(&basics, &settings).map_err(Error::ScopeError)?;
    let environment = env::Environment::from_module_scope(scope);

    evaluater::evaluate(&module, args, &environment, &settings)
        .map_err(|_| Error::EvaluateError)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let matches = App::new("erm")
        .version("0.1")
        .arg(Arg::with_name("quiet").short("q").long("quiet"))
        .arg(Arg::with_name("path").index(1))
        .get_matches();

    match matches.value_of("path") {
        Some(path) => {
            let quiet = matches.is_present("quiet");

            let attr = std::fs::metadata(path).map_err(|_| Error::FileError)?;

            if attr.is_dir() {
                Ok(())
            } else {
                dump_file(path, quiet)
            }
        }
        None => Ok(()),
    }
}
