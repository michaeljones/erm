mod common;

mod prelude {

    use std::path::PathBuf;

    use erm::project;

    use crate::common::eval;

    #[test]
    fn use_built_in_string_module() {
        // To check that something like the String module can exist in the prelude and be accessed in
        // general functions without importing it
        let src = r#"
        module Main exposing (..)
        main args =
          String.append "Hello, " "World"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn imported_module_can_use_prelude() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test
        main args =
          Impl.Test.hello_from_prelude
        "#;

        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        insta::assert_snapshot!(result);
    }
}
