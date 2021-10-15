extern crate codespan_reporting;
extern crate erm;
extern crate im;
extern crate unindent;

mod common;

mod full {

    use erm::checker::{self, unify};
    use erm::evaluater::values::Value;
    use erm::parser;

    use common::{eval, eval_with_args, pretty_print, string, Error};

    #[test]
    fn basic_string() {
        let src = r#"
        module Main exposing (..)
        main =
          "hello, world"
        "#;
        let result = eval(src, None);
        assert_eq!(result, Ok(string("hello, world")));
    }

    #[test]
    fn string_from_int() {
        let src = "
        module Main exposing (..)
        main =
          String.fromInt 5
        ";
        let result = eval(src, None);
        assert_eq!(result, Ok(string("5")));
    }

    #[test]
    fn add_ints() {
        let src = "
        module Main exposing (..)
        main =
          String.fromInt (1 + 3)
        ";
        let result = eval(src, None);
        assert_eq!(result, Ok(string("4")));
    }

    #[test]
    fn add_int_and_string_fails() {
        let src = r#"
        module Main exposing (..)
        main =
          String.fromInt (1 + "string")
        "#;
        let result = eval(src, None);
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
        assert_eq!(eval(module, None), Ok(Value::String("-109".to_string())));
    }

    #[test]
    fn arithmetic_parenthesis() {
        let src = "
        module Main exposing (..)
        main =
          String.fromInt ((10 - 11) * (12 + 13))
        ";
        let result = eval(src, None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
        assert_eq!(result, Ok(string("False")), "{}", pretty_print(&result));
    }

    #[test]
    fn string_concatenation() {
        let module = r#"
        module Main exposing (..)
        main =
          "a" ++ "bc" ++ "def"
        "#;
        assert_eq!(eval(module, None), Ok(string("abcdef")));
    }

    #[test]
    fn if_statement_single_line() {
        let src = r#"
        module Main exposing (..)
        main =
          if True then "5" else "4"
        "#;
        let result = eval(src, None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
        assert_eq!(result, Ok(string("12")), "{}", pretty_print(&result));
    }

    #[test]
    fn main_args() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join args
        "#;
        let result = eval_with_args(src, vec!["Hello".to_string(), " world".to_string()], None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
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
        let result = eval(src, None);
        assert_eq!(
            result,
            Ok(string("Hello, World")),
            "{}",
            pretty_print(&result)
        );
    }
}
