extern crate codespan_reporting;
extern crate erm;
extern crate im;
extern crate unindent;

mod common;

mod prelude {

    use std::path::PathBuf;

    use erm::project;

    use common::{eval, pretty_print, string};

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

    #[test]
    fn imported_module_can_use_prelude() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test
        main =
          Impl.Test.hello_from_prelude
        "#;

        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        assert_eq!(
            result,
            Ok(string("Hello, from prelude")),
            "{}",
            pretty_print(&result)
        );
    }
}
