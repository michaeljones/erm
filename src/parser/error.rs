use crate::lexer::Range;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedToken {
        expected: String,
        found: String,
        range: Range,
    },
    UnexpectedEnd,
    Indent {
        range: Range,
    },
    TokensRemaining(Vec<String>),
    NoOperand,
    NoOperator,
    EmptyOperatorStack,
    UnknownOperator(String),
    UnknownExposing(String),
    NegativePrecendence,
    NameMismatch,
    Unknown,
}
