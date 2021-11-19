extern crate erm;
extern crate im;
extern crate insta;
extern crate logos;
extern crate unindent;

mod common;

mod types {

    use common::eval;

    #[test]
    fn main_string_type_signature() {
        let src = r#"
        module Main exposing (..)

        main : List String -> String
        main args =
          "hello, world with type annotation"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn other_type_annotation() {
        let src = r#"
        module Main exposing (..)

        add1 : Int -> Int
        add1 x =
          x + 1

        main : List String -> String
        main args =
          String.fromInt (add1 3)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn custom_type() {
        let src = r#"
        module Main exposing (..)

        type MyType
          = MyA
          | MyB

        main : List String -> String
        main args =
          "Hello with type"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn custom_type_with_concrete_args() {
        let src = r#"
        module Main exposing (..)

        type MyType
          = MyA Int
          | MyB String

        main : List String -> String
        main args =
          "Hello with type"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn custom_type_with_type_variable() {
        let src = r#"
        module Main exposing (..)

        type MyType a
          = MyA a
          | MyB String

        main : List String -> String
        main args =
          "Hello with type variable"
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }
}
