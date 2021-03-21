#[macro_use]
extern crate im;
extern crate clap;
extern crate erm;
extern crate logos;
extern crate regex;
extern crate walkdir;

use clap::{App, Arg};
use logos::Logos;
// use walkdir::WalkDir;

// use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

use erm::env;
use erm::evaluater;
use erm::lexer::Token;
use erm::parser;

#[derive(Debug)]
enum Error {
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

    let module = parser::parse(&mut iter).map_err(|err| {
        println!("{:?}", &err);
        Error::ParserError
    })?;

    println!("{:#?}", &module);

    let args = vec!["example_arg".to_string()];

    let basics = erm::parse_basics().map_err(|_| Error::ParserError)?;
    let scope = env::Scope::from_module(&basics).map_err(Error::ScopeError)?;

    let scopes = vector![Rc::new(scope)];
    let environment = env::Environment {
        module_scopes: scopes,
        local_scopes: vector![],
    };
    evaluater::evaluate(&module, args, &environment).map_err(|err| {
        println!("{:?}", &err);
        Error::EvaluateError
    })?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let matches = App::new("elm-parser-dump")
        .version("0.1")
        .arg(Arg::with_name("quiet").short("q").long("quiet"))
        .arg(Arg::with_name("path").index(1))
        .get_matches();

    match matches.value_of("path") {
        Some(path) => {
            let quiet = matches.is_present("quiet");

            let attr = std::fs::metadata(path).expect("file not found");

            if attr.is_dir() {
                Ok(())
            } else {
                dump_file(path, quiet)
            }
        }
        None => Ok(()),
    }
}
