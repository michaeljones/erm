extern crate erm;
extern crate im;
extern crate insta;
extern crate logos;
extern crate unindent;

mod common;

mod case {

    use common::eval;

    #[test]
    fn case_statement() {
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
}
