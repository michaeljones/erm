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

        main : List String -> String
        main args =
          case True of
            True -> "True"
            False -> "False"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }
}
