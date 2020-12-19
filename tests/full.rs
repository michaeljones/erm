extern crate elm;
extern crate logos;
extern crate unindent;

#[cfg(test)]
mod tests {
    use elm::evaluate;
    use elm::evaluate::Value;
    use elm::lexer::Token;
    use elm::parser;
    use logos::Logos;
    use unindent::unindent;

    #[derive(Debug, PartialEq)]
    enum Error {
        ParserError(parser::Error),
        EvaluateError(evaluate::Error),
    }

    fn eval(string: &str) -> Result<Value, Error> {
        let src = unindent(&string);
        let tokens = Token::lexer(&src);
        let module = parser::parse(&mut tokens.peekable()).map_err(Error::ParserError)?;
        evaluate::evaluate(&module).map_err(Error::EvaluateError)
    }

    #[test]
    fn add_ints() {
        let module = "
        module Main exposing (..)
        main =
          1 + 3
        ";
        assert_eq!(eval(module), Ok(Value::Integer(4)));
    }

    #[test]
    fn arithmetic_precendence() {
        let module = "
        module Main exposing (..)
        main =
          10 - 11 * 12 + 13
        ";
        assert_eq!(eval(module), Ok(Value::Integer(-109)));
    }

    #[test]
    fn string_concatenation() {
        let module = r#"
        module Main exposing (..)
        main =
          "a" ++ "bc" ++ "def"
        "#;
        assert_eq!(eval(module), Ok(Value::String("abcdef".to_string())));
    }

    #[test]
    fn if_statement() {
        let module = r#"
        module Main exposing (..)
        main =
          if True then
            5
          else
            4
        "#;
        assert_eq!(eval(module), Ok(Value::Integer(5)));
    }
}
