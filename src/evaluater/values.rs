#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(String),
    List(Vec<Value>),
}
