use logos::{Lexer, Logos};

pub type Range = std::ops::Range<usize>;

#[allow(dead_code)]
pub type SrcToken<'src> = (Token<'src>, Range);

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token<'src> {
    // Keywords
    #[token("module")]
    Module,
    #[token("port")]
    Port,
    #[token("type")]
    Type,
    #[token("alias")]
    Alias,
    #[token("exposing")]
    Exposing,
    #[token("as")]
    As,
    #[token("import")]
    Import,
    #[token("case")]
    Case,
    #[token("of")]
    Of,
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,

    // Open & Close
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    // Whitespace
    #[regex(" +", |lex| lex.slice().len())]
    Space(usize),

    #[token("\n")]
    NewLine,

    // Key symbols
    #[token("|")]
    Bar,

    #[token(",")]
    Comma,

    #[token(".")]
    Point,

    #[token("..")]
    Ellipsis,

    #[token("=")]
    Equals,

    #[token(":")]
    Colon,

    #[token("\\")]
    BackSlash,

    #[token("->")]
    RightArrow,

    // Names
    #[regex("[A-Z][a-zA-Z0-9]*")]
    UpperName(&'src str),

    #[regex("[a-z_][a-zA-Z0-9]*")]
    LowerName(&'src str),

    #[regex(r#"[+><!*-:|]+"#)]
    Operator(&'src str),

    #[regex("--[^\n]*")]
    SingleLineComment(&'src str),

    #[regex(r#"\{-(?:[^-]|\-[^}])*\-}"#)]
    MultiLineComment(&'src str),

    #[regex(r#"\[glsl\|(?:[^|]|\|[^]])*\|]"#)]
    WebGL(&'src str),

    #[regex("-?[0-9]+", |lex| lex.slice().parse::<i32>(), priority = 2)]
    LiteralInteger(i32),

    #[regex("[0-9]*\\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f32>())]
    LiteralFloat(f32),

    #[regex(r#""([^"])*""#, string_contents)]
    LiteralString(&'src str),

    #[regex(r#"'[^']'"#, string_contents)]
    LiteralChar(&'src str),

    // Error
    #[error]
    Error,
}

fn string_contents<'src>(lex: &mut Lexer<'src, Token<'src>>) -> Option<&'src str> {
    let slice = lex.slice();
    Some(&slice[1..slice.len() - 1])
}

impl<'src> std::fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
