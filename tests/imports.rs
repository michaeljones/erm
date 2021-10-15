extern crate codespan_reporting;
extern crate erm;
extern crate im;
extern crate unindent;

mod common;

mod imports {

    use std::path::PathBuf;

    use erm::env;
    use erm::project;

    use common::{eval, pretty_print, string, Error};

    #[test]
    fn unable_to_find_import() {
        let src = r#"
        module Main exposing (..)
        import Does.Not.Exist
        main =
          String.append "Hello, " "World"
        "#;
        let result = eval(src, None);
        assert_eq!(
            result,
            Err(Error::ScopeError(env::Error::UnableToFindModule(
                "Does.Not.Exist".to_string()
            )))
        );
    }

    #[test]
    fn imports_module_from_configured_folder() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test
        main =
          Impl.Test.hello
        "#;

        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        assert_eq!(
            result,
            Ok(string("Hello from Impl.Test")),
            "{}",
            pretty_print(&result)
        );
    }
}
