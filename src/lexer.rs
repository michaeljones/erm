use logos::{Lexer, Logos};

pub type Range = std::ops::Range<usize>;

#[allow(dead_code)]
pub type SrcToken<'src> = (Token<'src>, Range);

#[allow(dead_code)]
pub type TokenIter<'src> = std::iter::Peekable<logos::SpannedIter<'src, Token<'src>>>;

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
    #[token("infix")]
    Infix,

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
    //
    // Any number of capitalised names separated by dots
    #[regex("([A-Z][a-zA-Z0-9_]*\\.)+[A-Z][a-zA-Z0-9_]*")]
    QualifiedUpperName(&'src str),

    // A single capitalised name, no dots
    #[regex("[A-Z][a-zA-Z0-9_]*")]
    UpperName(&'src str),

    // Any number of capitalised names, followed by at least one lower-case-starting name all
    // separated by dots
    #[regex("([A-Z][a-zA-Z0-9_]*\\.)+([a-z_][a-zA-Z0-9_]*\\.)*[a-z_][a-zA-Z0-9_]*")]
    QualifiedLowerName(&'src str),

    // A single lower-case-starting name, no dots
    #[regex("[a-z_][a-zA-Z0-9_]*")]
    LowerName(&'src str),

    #[regex(r#"[+><!*\-:|]+"#)]
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
