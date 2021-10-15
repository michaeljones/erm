#[macro_use]
extern crate im;
extern crate codespan_reporting;
extern crate env_logger;
extern crate erm;
extern crate logos;
extern crate unindent;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::Buffer;
use std::rc::Rc;
use unindent::unindent;

use erm::ast;
use erm::checker::{self, unify};
use erm::env;
use erm::evaluater;
use erm::evaluater::values::Value;
use erm::lexer::Range;
use erm::parser;

#[derive(Debug, PartialEq)]
enum Error {
    ParserError(parser::Error, String),
    CheckError(checker::Error),
    EvaluateError(evaluater::Error),
    ScopeError(env::Error),
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

fn pretty_print(result: &Result<Value, Error>) -> String {
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

fn eval(string: &str) -> Result<Value, Error> {
    eval_with_args(string, Vec::new())
}

fn eval_with_args(string: &str, args: Vec<String>) -> Result<Value, Error> {
    let _ = env_logger::builder().is_test(true).try_init();

    log::trace!("eval_with_args");

    let src = unindent(&string);
    let module =
        erm::parse_source(src).map_err(|err| Error::ParserError(err, string.to_string()))?;

    let module = ast::with_default_imports(&module);

    let scope = env::Scope::from_module(&module).map_err(Error::ScopeError)?;
    let scopes = vector![Rc::new(scope)];
    let environment = env::Environment {
        module_scopes: scopes,
        local_scopes: vector![],
    };
    checker::check(&module, &environment).map_err(Error::CheckError)?;
    evaluater::evaluate(&module, args, &environment).map_err(Error::EvaluateError)
}

fn string(str: &str) -> Value {
    Value::String(str.to_string())
}

#[test]
fn basic_string() {
    let src = r#"
        module Main exposing (..)
        main =
          "hello, world"
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("hello, world")));
}

#[test]
fn string_from_int() {
    let src = "
        module Main exposing (..)
        main =
          String.fromInt 5
        ";
    let result = eval(src);
    assert_eq!(result, Ok(string("5")));
}

#[test]
fn add_ints() {
    let src = "
        module Main exposing (..)
        main =
          String.fromInt (1 + 3)
        ";
    let result = eval(src);
    assert_eq!(result, Ok(string("4")));
}

#[test]
fn add_int_and_string_fails() {
    let src = r#"
        module Main exposing (..)
        main =
          String.fromInt (1 + "string")
        "#;
    let result = eval(src);
    assert_eq!(
        result,
        Err(Error::CheckError(checker::Error::UnifyError(
            unify::Error::FailedToUnify(
                "Constant(String)".to_string(),
                "Constant(Integer)".to_string()
            )
        )))
    );
}

#[test]
fn arithmetic_precedence() {
    let module = "
        module Main exposing (..)
        main =
          String.fromInt (10 - 11 * 12 + 13)
        ";
    assert_eq!(eval(module), Ok(Value::String("-109".to_string())));
}

#[test]
fn arithmetic_parenthesis() {
    let src = "
        module Main exposing (..)
        main =
          String.fromInt ((10 - 11) * (12 + 13))
        ";
    let result = eval(src);
    assert_eq!(result, Ok(string("-25")), "{}", pretty_print(&result));
}

#[test]
fn int_comparison_gt() {
    let src = r#"
        module Main exposing (..)
        stringFromBool boolean =
          if boolean then
            "True"
          else
            "False"
        main =
          stringFromBool (8 + 12 > 7 + 5)
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("True")), "{}", pretty_print(&result));
}

#[test]
fn int_comparison_lt() {
    let src = r#"
        module Main exposing (..)
        stringFromBool boolean = if boolean then "True" else "False"
        main =
          stringFromBool (8 + 12 < 7 + 5)
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("False")), "{}", pretty_print(&result));
}

#[test]
fn string_concatenation() {
    let module = r#"
        module Main exposing (..)
        main =
          "a" ++ "bc" ++ "def"
        "#;
    assert_eq!(eval(module), Ok(string("abcdef")));
}

#[test]
fn if_statement_single_line() {
    let src = r#"
        module Main exposing (..)
        main =
          if True then "5" else "4"
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("5")), "{}", pretty_print(&result));
}

#[test]
fn if_statement_multi_line() {
    let src = r#"
        module Main exposing (..)
        main =
          if False then
            String.fromInt 5
          else
            "4"
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("4")), "{}", pretty_print(&result));
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
            src.to_string()
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
              "8"
            else
              "12"
          else
            "23"
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("12")), "{}", pretty_print(&result));
}

#[test]
fn main_args() {
    let src = r#"
        module Main exposing (..)
        main args =
          String.join args
        "#;
    let result = eval_with_args(src, vec!["Hello".to_string(), " world".to_string()]);
    assert_eq!(
        result,
        Ok(string("Hello world")),
        "{}",
        pretty_print(&result)
    );
}

#[test]
fn function_call_simple() {
    let src = r#"
        module Main exposing (..)
        add1 x = x + 1
        main =
          String.fromInt (add1 5)
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("6")), "{}", pretty_print(&result));
}

#[test]
fn function_call_with_paren_args() {
    let src = r#"
        module Main exposing (..)
        addTogether x y = x + y
        main =
          String.fromInt (addTogether (addTogether 2 5) 8)
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("15")), "{}", pretty_print(&result));
}

#[test]
fn function_clashes_with_operator_func() {
    // This is to make sure that our attempt to call the user defined 'add' doesn't clash with the
    // 'add' that is defined as the implementation of '+' in basics
    let src = r#"
        module Main exposing (..)
        add x y = x + y
        main =
          String.fromInt (add 2 5)
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("7")), "{}", pretty_print(&result));
}

#[test]
fn function_calls_function() {
    // Make sure we can call a function from a function
    let src = r#"
        module Main exposing (..)
        sub1 y = y - 1
        add1 x = sub1 x + 2
        main =
          String.fromInt (add1 2)
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("3")), "{}", pretty_print(&result));
}

#[test]
fn function_calls_same_args() {
    // To cover a bug where we get confused when functions have the same arg name
    let src = r#"
        module Main exposing (..)
        sub1 x = x - 1
        add1 x = sub1 x + 2
        main =
          String.fromInt (add1 2)
        "#;
    let result = eval(src);
    assert_eq!(result, Ok(string("3")), "{}", pretty_print(&result));
}

#[test]
fn use_built_in_string_module() {
    // To check that something like the String module can exist in the prelude and be accessed in
    // general functions without importing it
    let src = r#"
        module Main exposing (..)
        main =
          String.append "Hello, " "World"
        "#;
    let result = eval(src);
    assert_eq!(
        result,
        Ok(string("Hello, World")),
        "{}",
        pretty_print(&result)
    );
}

#[test]
fn unable_to_find_import() {
    let src = r#"
        module Main exposing (..)
        import Does.Not.Exist
        main =
          String.append "Hello, " "World"
        "#;
    let result = eval(src);
    assert_eq!(
        result,
        Err(Error::ScopeError(env::Error::UnableToFindModule(
            "Does.Not.Exist".to_string()
        )))
    );
}
