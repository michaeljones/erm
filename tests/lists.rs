extern crate codespan_reporting;
extern crate erm;
extern crate im;
extern crate unindent;

mod common;

mod lists {

    use common::{eval, pretty_print, string};

    #[test]
    fn empty_list_literal() {
        // To check that something like the String module can exist in the prelude and be accessed in
        // general functions without importing it
        let src = r#"
        module Main exposing (..)
        main =
          String.join "," []
        "#;
        let result = eval(src, None);
        assert_eq!(result, Ok(string("")), "{}", pretty_print(&result));
    }

    /*
    #[test]
    fn list_literal_with_strings() {
        // To check that something like the String module can exist in the prelude and be accessed in
        // general functions without importing it
        let src = r#"
        module Main exposing (..)
        main =
          String.join "," ["Hello", "World]
        "#;
        let result = eval(src, None);
        assert_eq!(result, Ok(string("")), "{}", pretty_print(&result));
    }
    */
}
