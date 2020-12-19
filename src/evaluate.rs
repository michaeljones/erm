use super::parser::{Expr, Module, Stmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    NoMain,
    UnsupportedOperation,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(String),
}

pub fn evaluate(module: &Module) -> Result<Value, Error> {
    let main_expr = module
        .statements
        .iter()
        .find_map(|stmt| match stmt {
            Stmt::Function { name, expr } => {
                if name == &"main" {
                    Some(expr)
                } else {
                    None
                }
            }
        })
        .ok_or(Error::NoMain)?;

    let value = evaluate_expression(&main_expr);

    println!("{:?}", &value);

    value
}

fn evaluate_expression<'a>(expr: &Expr<'a>) -> Result<Value, Error> {
    match expr {
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::Integer(int) => Ok(Value::Integer(*int)),
        Expr::Float(float) => Ok(Value::Float(*float)),
        Expr::String(string) => Ok(Value::String(string.to_string())),
        Expr::BinOp {
            operator,
            left,
            right,
        } => evaluate_binary_expression(operator, left, right),
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => evaluate_if_expression(condition, then_branch, else_branch),
    }
}

fn evaluate_binary_expression<'a>(
    operator: &'a str,
    left: &Expr<'a>,
    right: &Expr<'a>,
) -> Result<Value, Error> {
    let left_value = evaluate_expression(left)?;
    let right_value = evaluate_expression(right)?;

    match (operator, left_value, right_value) {
        ("+", Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
        ("+", Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
        ("-", Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
        ("-", Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
        ("*", Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
        ("*", Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
        ("++", Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
        _ => Err(Error::UnsupportedOperation),
    }
}

fn evaluate_if_expression<'a>(
    condition: &Expr<'a>,
    then_branch: &Expr<'a>,
    else_branch: &Expr<'a>,
) -> Result<Value, Error> {
    let condition_value = evaluate_expression(condition)?;

    match condition_value {
        Value::Bool(true) => evaluate_expression(then_branch),
        Value::Bool(false) => evaluate_expression(else_branch),
        _ => Err(Error::UnsupportedOperation),
    }
}
