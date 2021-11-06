extern crate codespan_reporting;
extern crate erm;
extern crate im;
extern crate unindent;

mod common;

mod lists {

    use erm::checker;
    use erm::checker::unify;

    use common::{eval, pretty_print, string, Error};

    #[test]
    fn empty_list_literal() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join "," []
        "#;
        let result = eval(src, None);
        assert_eq!(result, Ok(string("")), "{}", pretty_print(&result));
    }

    #[test]
    fn list_literal_with_strings() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join "," ["Hello", " World"]
        "#;
        let result = eval(src, None);
        assert_eq!(
            result,
            Ok(string("Hello, World")),
            "{}",
            pretty_print(&result)
        );
    }

    #[test]
    fn list_literal_with_inconsistent_types() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.join "," ["Hello", 1]
        "#;
        let result = eval(src, None);
        assert_eq!(
            result,
            Err(Error::CheckError(checker::Error::UnifyError(
                unify::Error::FailedToUnify(
                    "Constant(Integer)".to_string(),
                    "Constant(String)".to_string()
                )
            ))),
            "{}",
            pretty_print(&result)
        );
    }

    #[test]
    fn list_sum() {
        let src = r#"
        module Main exposing (..)
        main args =
          String.fromInt (List.sum [1, 2])
        "#;
        let result = eval(src, None);
        assert_eq!(result, Ok(string("3")), "{}", pretty_print(&result));
    }
}
