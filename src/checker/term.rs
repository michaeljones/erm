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
    Function { name: String, signature: Vec<Term> },
}
