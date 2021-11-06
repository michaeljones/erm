// TODO: Unsure about making this 'Clone'. Done so that we can have the Value binding without too
// much effort at the moment but seems wrong
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(String),
    List(Vec<Value>),
}
