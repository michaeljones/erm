mod common;

mod case {

    use crate::common::eval;

    #[test]
    fn boolean_case_statement() {
        let src = r#"
        module Main exposing (..)

        toText arg =
          case arg of
            True -> "Hello"
            False -> " case statements"

        main : List String -> String
        main args =
            (toText True) ++ (toText False)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn integer_case_statement() {
        let src = r#"
        module Main exposing (..)

        toText arg =
          case arg of
            1 -> "Hello"
            2 -> " integer"
            _ -> " case statements"

        main : List String -> String
        main args =
            (toText 1) ++ (toText 2) ++ (toText 3)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn error_on_float_case_statement() {
        let src = r#"
        module Main exposing (..)

        toText arg =
          case arg of
            1.1 -> "Hello"
            2.3 -> " integer"
            _ -> " case statements"

        main : List String -> String
        main args =
            (toText 1.1) ++ (toText 2.3)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn match_any_with_underscore() {
        let src = r#"
        module Main exposing (..)

        toText arg =
          case arg of
            True -> "Hello"
            _ -> " case statements with an underscore"

        main : List String -> String
        main args =
            (toText True) ++ (toText False)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }
}
