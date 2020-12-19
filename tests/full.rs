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
        ParserError,
        EvaluateError,
    }

    fn eval(string: &str) -> Result<Value, Error> {
        let src = unindent(&string);
        let tokens = Token::lexer(&src);
        let module = parser::parse(tokens).map_err(|_| Error::ParserError)?;
        evaluate::evaluate(&module).map_err(|_| Error::EvaluateError)
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
}
