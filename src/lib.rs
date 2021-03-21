extern crate im;
extern crate log;
extern crate logos;

use logos::Logos;

pub mod bindings;
pub mod builtins;
pub mod checker;
pub mod env;
pub mod evaluater;
pub mod lexer;
pub mod module;
pub mod parser;

use self::lexer::Token;

pub enum Error {
    ParserError(parser::Error, String),
    ScopeError(env::Error, String),
}

pub fn parse_source(source: String) -> parser::ParseResult {
    let tokens = Token::lexer(&source);
    let mut iter = tokens.spanned().peekable();
    parser::parse(&mut iter)
}

pub fn prelude() -> Result<env::ModuleScope, Error> {
    let basics = parse_basics().map_err(|err| Error::ParserError(err, basics_source()))?;
    // let string = parse_string().map_err(|err| Error::ParserError(err, string_source()))?;
    env::Scope::from_module(&basics).map_err(|err| Error::ScopeError(err, basics_source()))
}

pub fn parse_basics() -> parser::ParseResult {
    let src = basics_source();
    parse_source(src)
}

pub fn basics_source() -> String {
    String::from(include_str!("../core/Basics.elm"))
}

pub fn parse_string() -> parser::ParseResult {
    let src = basics_string();
    parse_source(src)
}

pub fn basics_string() -> String {
    String::from(include_str!("../core/String.elm"))
}
