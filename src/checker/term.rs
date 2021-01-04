#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool,
    Integer,
    Float,
    String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    Constant(Value),
    Var(String),
    Function(Box<Term>, Box<Term>),
}
