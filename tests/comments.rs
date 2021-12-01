mod common;

mod comments {

    use crate::common::eval;

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

    #[test]
    fn multi_line_comment() {
        let src = r#"
        module Main exposing (..)

        {- This is a comment
        -}
        main : List String -> String
        main args =
            "Hello comments"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn multi_line_comment_on_single_line() {
        let src = r#"
        module Main exposing (..)

        {- This is a comment -}
        main : List String -> String
        main args =
            "Hello comments"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }
}
