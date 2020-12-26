extern crate im;
extern crate logos;

use logos::Logos;

pub mod checker;
pub mod env;
pub mod evaluater;
pub mod function;
pub mod lexer;
pub mod parser;

use self::lexer::Token;

pub fn parse_source<'src>(source: &'src str) -> parser::ParseResult<'src> {
    let tokens = Token::lexer(&source);
    let mut iter = tokens.spanned().peekable();
    parser::parse(&mut iter)
}
