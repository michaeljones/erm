extern crate codespan_reporting;
extern crate erm;
extern crate logos;
extern crate unindent;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::Buffer;
use unindent::unindent;

use erm::checker;
use erm::evaluater;
use erm::evaluater::values::Value;
use erm::lexer::Range;
use erm::parser;

#[derive(Debug, PartialEq)]
enum Error<'src> {
    ParserError(parser::Error, &'src str),
    CheckError(checker::Error),
    EvaluateError(evaluater::Error),
}

fn get_range<'src>(result: &Result<Value, Error<'src>>) -> Option<(&'src str, Range)> {
    match result {
        Err(Error::ParserError(parser::Error::UnexpectedToken { range, .. }, src)) => {
            Some((src, range.clone()))
        }
        Err(Error::ParserError(parser::Error::Indent { range }, src)) => Some((src, range.clone())),
        _ => None,
    }
}

fn pretty_print(result: &Result<Value, Error>) -> String {
    if let Some((src, range)) = get_range(result) {
        let mut files = SimpleFiles::new();
        let file_id = files.add("sample", unindent(src));
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

fn eval(string: &str) -> Result<Value, Error> {
    let src = unindent(&string);
    let _basics =
        erm::parse_basics().map_err(|err| Error::ParserError(err, erm::basics_source()))?;
    let module = erm::parse_source(&src).map_err(|err| Error::ParserError(err, string))?;
    checker::check(&module).map_err(Error::CheckError)?;
    evaluater::evaluate(&module, Vec::new()).map_err(Error::EvaluateError)
}

fn eval_with_args(string: &str, args: Vec<String>) -> Result<Value, Error> {
    let src = unindent(&string);
    let module = erm::parse_source(&src).map_err(|err| Error::ParserError(err, string))?;
    checker::check(&module).map_err(Error::CheckError)?;
    evaluater::evaluate(&module, args).map_err(Error::EvaluateError)
}

#[test]
fn basic_string() {
    let src = r#"
        module Main exposing (..)
        main =
          "hello, world"
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(Value::String("hello, world".to_string())),);
}

#[test]
fn string_from_int() {
    let src = "
        module Main exposing (..)
        main =
          stringFromInt 5
        ";
    let result = eval(src);
    assert_eq!(result, Ok(Value::String("5".to_string())),);
}

#[test]
fn add_ints() {
    let src = "
        module Main exposing (..)
        main =
          stringFromInt (1 + 3)
        ";
    let result = eval(src);
    assert_eq!(result, Ok(Value::Integer(4)),);
}

#[test]
fn arithmetic_precendence() {
    let module = "
        module Main exposing (..)
        main =
          10 - 11 * 12 + 13
        ";
    assert_eq!(eval(module), Ok(Value::Integer(-109)));
}

#[test]
fn arithmetic_parenthesis() {
    let src = "
        module Main exposing (..)
        main =
          (10 - 11) * (12 + 13)
        ";
    let result = eval(src);
    assert_eq!(result, Ok(Value::Integer(-25)), "{}", pretty_print(&result));
}

#[test]
fn int_comparison_gt() {
    let src = "
        module Main exposing (..)
        main =
          8 + 12 > 7 + 5
        ";
    let result = eval(src);
    assert_eq!(result, Ok(Value::Bool(true)), "{}", pretty_print(&result));
}

#[test]
fn int_comparison_lt() {
    let src = "
        module Main exposing (..)
        main =
          8 + 12 < 7 + 5
        ";
    let result = eval(src);
    assert_eq!(result, Ok(Value::Bool(false)), "{}", pretty_print(&result));
}

#[test]
fn string_concatenation() {
    let module = r#"
        module Main exposing (..)
        main =
          "a" ++ "bc" ++ "def"
        "#;
    assert_eq!(eval(module), Ok(Value::String("abcdef".to_string())));
}

#[test]
fn if_statement_single_line() {
    let src = r#"
        module Main exposing (..)
        main =
          if True then 5 else 4
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(Value::Integer(5)), "{}", pretty_print(&result));
}

#[test]
fn if_statement_multi_line() {
    let src = r#"
        module Main exposing (..)
        main =
          if False then
            5
          else
            4
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(Value::Integer(4)), "{}", pretty_print(&result));
}

#[test]
fn if_statement_multi_line_bad() {
    let src = r#"
        module Main exposing (..)
        main =
          if False then
         5
          else
            4
        "#;
    let result = eval(src);
    assert_eq!(
        result,
        Err(Error::ParserError(
            parser::Error::Indent { range: 50..51 },
            &src
        )),
    );
}

#[test]
fn nested_if_statement() {
    let src = r#"
        module Main exposing (..)
        main =
          if True then
            if False then
              8
            else
              12
          else
            23
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(Value::Integer(12)), "{}", pretty_print(&result));
}

#[test]
fn main_args() {
    let src = r#"
        module Main exposing (..)
        main args =
          args
        "#;
    let result = eval_with_args(src, vec!["Hello".to_string()]);
    assert_eq!(
        result,
        Ok(Value::List(vec![Value::String("Hello".to_string())])),
        "{}",
        pretty_print(&result)
    );
}

#[test]
fn function_call() {
    let src = r#"
        module Main exposing (..)
        add1 x = x + 1
        main =
          add1 5
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(Value::Integer(6)), "{}", pretty_print(&result));
}

#[test]
fn function_call_with_paren_args() {
    let src = r#"
        module Main exposing (..)
        add x y = x + y
        main =
          add (add 2 5) 8
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(Value::Integer(15)), "{}", pretty_print(&result));
}
