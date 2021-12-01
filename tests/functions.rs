mod common;

mod functions {

    use crate::common::eval;

    #[test]
    fn function_call_simple() {
        let src = r#"
        module Main exposing (..)
        add1 x = x + 1
        main args =
          String.fromInt (add1 5)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn function_call_with_paren_args() {
        let src = r#"
        module Main exposing (..)
        addTogether x y = x + y
        main args =
          String.fromInt (addTogether (addTogether 2 5) 8)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn function_clashes_with_operator_func() {
        // This is to make sure that our attempt to call the user defined 'add' doesn't clash with the
        // 'add' that is defined as the implementation of '+' in basics
        let src = r#"
        module Main exposing (..)
        add x y = x + y
        main args =
          String.fromInt (add 2 5)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn function_calls_function() {
        // Make sure we can call a function from a function
        let src = r#"
        module Main exposing (..)
        sub1 y = y - 1
        add1 x = sub1 x + 2
        main args =
          String.fromInt (add1 2)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn function_calls_same_args() {
        // To cover a bug where we get confused when functions have the same arg name
        let src = r#"
        module Main exposing (..)
        sub1 x = x - 1
        add1 x = sub1 x + 2
        main args =
          String.fromInt (add1 2)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn function_partially_applied() {
        let src = r#"
        module Main exposing (..)
        add x y = x + y
        main args =
          String.fromInt ((add 2) 3)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn function_two_partially_applied() {
        let src = r#"
        module Main exposing (..)
        add x y = x + y
        main args =
          String.fromInt ((add 3) ((add 2) 3))
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn function_double_partially_applied() {
        let src = r#"
        module Main exposing (..)
        add x y z = x + y + z
        main args =
          String.fromInt (((add 3) 4) 5)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }

    #[test]
    fn underscore_as_argument() {
        let src = r#"
        module Main exposing (..)
        add x y z = x + y + z
        main args =
          String.fromInt (add _ 1 2)
        "#;
        let result = eval(src, None);
        insta::assert_snapshot!(result);
    }
}
