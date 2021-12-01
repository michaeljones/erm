mod common;

mod imports {

    use std::path::PathBuf;

    use erm::project;

    use crate::common::eval;

    #[test]
    fn unable_to_find_import() {
        let src = r#"
        module Main exposing (..)
        import Does.Not.Exist
        main args =
          String.append "Hello, " "World"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    /*
    #[test]
    fn imported_symbol_does_not_exist() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test exposing (does_not_exist)
        main args =
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
    */

    #[test]
    fn imports_module_from_configured_folder() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test
        main args =
          Impl.Test.hello
        "#;

        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        insta::assert_snapshot!(result);
    }

    #[test]
    fn imports_module_from_imported_module() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test
        main args =
          Impl.Test.hello_from_import
        "#;

        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        insta::assert_snapshot!(result);
    }

    #[test]
    fn import_specific_symbol() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test exposing (hello)
        main args =
          hello
        "#;
        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        insta::assert_snapshot!(result);
    }

    #[test]
    fn indented_first_import_fails() {
        let src = r#"
        module Main exposing (..)
          import Impl.Test exposing (hello)
        main args =
          hello
        "#;
        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        insta::assert_snapshot!(result);
    }

    #[test]
    fn indented_second_import_fails() {
        let src = r#"
        module Main exposing (..)
        import Impl.Test exposing (hello)
          import Impl.Test.Other exposing (hello)
        main args =
          hello
        "#;
        let settings = project::Settings {
            source_directories: vec![PathBuf::from("tests/modules")],
        };

        let result = eval(src, Some(settings));
        insta::assert_snapshot!(result);
    }
}
