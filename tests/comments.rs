extern crate erm;
extern crate im;
extern crate insta;
extern crate logos;
extern crate unindent;

mod common;

mod comments {

    use common::eval;

    #[test]
    fn single_line_comment() {
        let src = r#"
        module Main exposing (..)

        -- This is a comment
        main : List String -> String
        main args =
            "Hello comments"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }
}
