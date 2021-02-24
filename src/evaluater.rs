pub mod values;

use std::rc::Rc;

use self::values::Value;
use super::env;
use super::function;
use super::function::Binding;
use super::parser::{Expr, Module, Pattern, Stmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnsupportedOperation,
    UnknownFunction(String, u32),
    UnknownBinding(String),
    FunctionError(function::Error),
    WrongArity,
}

pub fn evaluate<'src>(
    module: &Module<'src>,
    args: &'src Vec<String>,
    scopes: &env::Scopes<'src>,
) -> Result<Value, Error> {
    let scope = env::Scope::from_module(&module);
    let scopes = env::add_scope(&scopes, scope);
    if args.is_empty() {
        match env::get_binding(&scopes, "main") {
            Some(Binding::UserBinding(expr)) => evaluate_expression(&expr, &scopes),
            _ => Err(Error::UnknownBinding("main".to_string())),
        }
    } else {
        let call_main = Expr::Call {
            function_name: "main",
            args: vec![Rc::new(Expr::List(
                args.iter()
                    .map(|entry| Rc::new(Expr::String(entry)))
                    .collect(),
            ))],
        };

        evaluate_expression(&call_main, &scopes)
    }
}

fn evaluate_expression<'src>(
    expr: &Expr<'src>,
    scopes: &env::Scopes<'src>,
) -> Result<Value, Error> {
    match expr {
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::Integer(int) => Ok(Value::Integer(*int)),
        Expr::Float(float) => Ok(Value::Float(*float)),
        Expr::String(string) => Ok(Value::String(string.to_string())),
        Expr::BinOp {
            operator,
            left,
            right,
        } => evaluate_binary_expression(operator, left, right, &scopes),
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => evaluate_if_expression(condition, then_branch, else_branch, &scopes),
        Expr::List(items) => {
            let value_items = items
                .iter()
                .map(|expr| evaluate_expression(expr, &scopes))
                .collect::<Result<Vec<Value>, Error>>()?;
            Ok(Value::List(value_items))
        }
        Expr::Call {
            function_name,
            args,
        } => evaluate_function_call(function_name, args, &scopes),
        Expr::VarName(name) => env::get_binding(&scopes, name)
            .ok_or(Error::UnknownBinding(name.to_string()))
            .and_then(|binding| match binding {
                Binding::UserBinding(expr) => evaluate_expression(&expr, &scopes),
                _ => Err(Error::UnknownBinding(name.to_string())),
            }),
    }
}

fn evaluate_function_call<'a, 'b, 'src: 'd, 'd>(
    name: &str,
    arg_exprs: &Vec<Rc<Expr<'src>>>,
    scopes: &env::Scopes<'d>,
) -> Result<Value, Error> {
    match env::get_binding(&scopes, name) {
        Some(Binding::UserFunc(stmt_rc)) => match &*stmt_rc {
            Stmt::Function { args, expr, .. } => {
                if arg_exprs.len() != args.len() {
                    Err(Error::WrongArity)
                } else {
                    let arg_scope = env::Scope::from_bindings(
                        args.iter()
                            .zip(arg_exprs.iter())
                            .map(|(Pattern::Name(name), expr)| {
                                (name.to_string(), Binding::UserBinding(expr.clone()))
                            })
                            .collect(),
                    );

                    let new_scope = env::add_scope(scopes, arg_scope);
                    evaluate_expression(&expr, &new_scope)
                }
            }
            _ => Err(Error::UnknownFunction(name.to_string(), line!())),
        },
        Some(Binding::BuiltInFunc(func)) => {
            let arg_values = arg_exprs
                .iter()
                .map(|expr| evaluate_expression(&expr, &scopes))
                .collect::<Result<Vec<Value>, Error>>()?;

            func.call(arg_values).map_err(Error::FunctionError)
        }
        entry => {
            println!("{:?}", entry);
            Err(Error::UnknownFunction(name.to_string(), line!()))
        }
    }
}

fn evaluate_binary_expression<'a, 'b, 'src: 'd, 'd>(
    operator: &str,
    left: &Expr<'src>,
    right: &Expr<'src>,
    scopes: &env::Scopes<'d>,
) -> Result<Value, Error> {
    let left_value = evaluate_expression(left, &scopes)?;
    let right_value = evaluate_expression(right, &scopes)?;

    match (operator, left_value, right_value) {
        ("+", Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
        ("+", Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
        ("-", Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
        ("-", Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
        ("*", Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
        ("*", Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
        ("++", Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
        (">", Value::Integer(l), Value::Integer(r)) => Ok(Value::Bool(l > r)),
        (">", Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l > r)),
        (">=", Value::Integer(l), Value::Integer(r)) => Ok(Value::Bool(l >= r)),
        (">=", Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l >= r)),
        ("<", Value::Integer(l), Value::Integer(r)) => Ok(Value::Bool(l < r)),
        ("<", Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l < r)),
        ("<=", Value::Integer(l), Value::Integer(r)) => Ok(Value::Bool(l <= r)),
        ("<=", Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l <= r)),
        _ => Err(Error::UnsupportedOperation),
    }
}

fn evaluate_if_expression<'a, 'b, 'src: 'd, 'd>(
    condition: &Expr<'src>,
    then_branch: &Expr<'src>,
    else_branch: &Expr<'src>,
    scopes: &env::Scopes<'d>,
) -> Result<Value, Error> {
    let condition_value = evaluate_expression(condition, &scopes)?;

    match condition_value {
        Value::Bool(true) => evaluate_expression(then_branch, &scopes),
        Value::Bool(false) => evaluate_expression(else_branch, &scopes),
        _ => Err(Error::UnsupportedOperation),
    }
}
