mod common;

mod full {

    use crate::common::{eval, eval_with_args};

    #[test]
    fn basic_string() {
        let src = r#"
        module Main exposing (..)
        main args =
          "hello, world"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn string_from_int() {
        let src = "
        module Main exposing (..)
        main args =
          String.fromInt 5
        ";
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn add_ints() {
        let src = "
        module Main exposing (..)
        main args =
          String.fromInt (1 + 3)
        ";
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn add_int_and_string_fails() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.fromInt (1 + "string")
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn arithmetic_precedence() {
        let src = "
        module Main exposing (..)
        main args =
          String.fromInt (10 - 11 * 12 + 13)
        ";
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn arithmetic_parenthesis() {
        let src = "
        module Main exposing (..)
        main args =
          String.fromInt ((10 - 11) * (12 + 13))
        ";
        let result = eval(src, None);
        insta::assert_snapshot!(result);
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
        main args =
          stringFromBool (8 + 12 > 7 + 5)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn int_comparison_lt() {
        let src = r#"
        module Main exposing (..)
        stringFromBool boolean = if boolean then "True" else "False"
        main args =
          stringFromBool (8 + 12 < 7 + 5)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn string_concatenation() {
        let src = r#"
        module Main exposing (..)
        main args =
          "a" ++ "bc" ++ "def"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn if_statement_single_line() {
        let src = r#"
        module Main exposing (..)
        main args =
          if True then "5" else "4"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn if_statement_multi_line() {
        let src = r#"
        module Main exposing (..)
        main args =
          if False then
            String.fromInt 5
          else
            "4"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn if_statement_multi_line_bad() {
        // From testing elm, it seems like if-statements don't mind a branch having less of an
        // ident than the if-line provided that it isn't all the way back the level of the parent
        let src = r#"
        module Main exposing (..)
        main args =
          if False then
        5
          else
            4
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn nested_if_statement() {
        let src = r#"
        module Main exposing (..)
        main args =
          if True then
            if False then
              "8"
            else
              "12"
          else
            "23"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn main_args() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join "" args
        "#;
        let result = eval_with_args(src, vec!["Hello".to_string(), " world".to_string()], None);
        insta::assert_snapshot!(result);
    }
}
