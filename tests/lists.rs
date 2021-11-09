extern crate codespan_reporting;
extern crate erm;
extern crate im;
extern crate logos;
extern crate unindent;

mod common;

mod lists {

    use common::eval;

    #[test]
    fn empty_list_literal() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join "," []
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn list_literal_with_strings() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join "," ["Hello", " World"]
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn list_literal_with_inconsistent_types() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join "," ["Hello", 1]
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn list_sum() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.fromInt (List.sum [1, 2])
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }
}
