pub mod values;

use log;
use std::rc::Rc;

use self::values::Value;
use super::ast::{self, Expr, Module, Pattern, Stmt};
use super::bindings::Binding;
use super::builtins;
use super::env::{self, FoundBinding};
use super::project;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnsupportedOperation,
    UnknownFunction(String, u32),
    UnknownBinding(String),
    FunctionError(builtins::Error),
    WrongArity,
    ScopeError(env::Error),
}

pub fn evaluate(
    _module: &Module,
    args: Vec<String>,
    environment: &env::Environment,
    _settings: &project::Settings,
) -> Result<Value, Error> {
    log::trace!("evaluate");

    if args.is_empty() {
        let main_name = ast::LowerName::simple("main".to_string());
        match env::get_binding(&environment, &main_name) {
            Some(FoundBinding::WithEnv(Binding::UserBinding(expr), _env)) => {
                evaluate_expression(&expr, &environment)
            }
            _ => Err(Error::UnknownBinding("main".to_string())),
        }
    } else {
        let call_main = Expr::Call {
            function_name: ast::LowerName::simple("main".to_string()),
            args: vec![Rc::new(Expr::List(
                args.iter()
                    .map(|entry| Rc::new(Expr::String(String::from(entry))))
                    .collect(),
            ))],
        };

        evaluate_expression(&call_main, &environment)
    }
}

fn evaluate_expression(expr: &Expr, environment: &env::Environment) -> Result<Value, Error> {
    match dbg!(expr) {
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::Integer(int) => Ok(Value::Integer(*int)),
        Expr::Float(float) => Ok(Value::Float(*float)),
        Expr::String(string) => Ok(Value::String(string.to_string())),
        Expr::BinOp {
            operator,
            left,
            right,
        } => evaluate_binary_expression(operator, left, right, &environment),
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => evaluate_if_expression(condition, then_branch, else_branch, &environment),
        Expr::List(items) => {
            let value_items = items
                .iter()
                .map(|expr| evaluate_expression(expr, &environment))
                .collect::<Result<Vec<Value>, Error>>()?;
            Ok(Value::List(value_items))
        }
        Expr::Call {
            function_name,
            args,
        } => evaluate_function_call(function_name, args, &environment),
        Expr::VarName(name) => env::get_binding(&environment, name)
            .ok_or(Error::UnknownBinding(name.to_string()))
            .and_then(|binding| match binding {
                FoundBinding::WithEnv(Binding::UserBinding(expr), env) => {
                    evaluate_expression(&expr, &env)
                }
                FoundBinding::WithEnv(Binding::Value(value), _env) => Ok(value),
                _ => Err(Error::UnknownBinding(name.to_string())),
            }),
    }
}

fn evaluate_function_call<'a, 'b, 'src: 'd, 'd>(
    name: &ast::LowerName,
    arg_exprs: &Vec<Rc<Expr>>,
    environment: &env::Environment,
) -> Result<Value, Error> {
    match env::get_binding(&environment, name) {
        Some(FoundBinding::BuiltInFunc(func)) => {
            let arg_values = arg_exprs
                .iter()
                .map(|expr| evaluate_expression(&expr, &environment))
                .collect::<Result<Vec<Value>, Error>>()?;

            func.call(arg_values).map_err(Error::FunctionError)
        }
        Some(FoundBinding::WithEnv(Binding::UserFunc(stmt_rc), _env)) => match &*stmt_rc {
            Stmt::Function { args, expr, .. } => {
                if arg_exprs.len() != args.len() {
                    Err(Error::WrongArity)
                } else {
                    // Evaluate each argument to the function call and create a map from argument
                    // value to argument name to use as a scope within the function evaluation
                    let pairs = args
                        .iter()
                        .zip(arg_exprs.iter())
                        .map(|(Pattern::Name(name), expr)| {
                            evaluate_expression(expr, &environment).map(|value| {
                                (
                                    ast::LowerName::simple(name.to_string()),
                                    Binding::Value(value),
                                )
                            })
                        })
                        .collect::<Result<_, _>>()?;

                    let arg_scope = env::Scope::from_bindings(pairs);

                    let environment = env::add_local_scope(environment, arg_scope);
                    println!("Environment: {:#?}", environment);
                    evaluate_expression(&expr, &environment)
                }
            }
            _ => Err(Error::UnknownFunction(name.to_string(), line!())),
        },
        Some(FoundBinding::WithEnv(Binding::UserBinding(expr), _env)) => {
            println!("expr {:#?}", expr);
            match &*expr {
                Expr::VarName(lower_name) => {
                    evaluate_function_call(lower_name, arg_exprs, environment)
                }
                _ => Err(Error::UnknownFunction(name.to_string(), line!())),
            }
        }
        entry => {
            println!("{:?}", entry);
            Err(Error::UnknownFunction(name.to_string(), line!()))
        }
    }
}

fn evaluate_binary_expression<'a, 'b, 'src: 'd, 'd>(
    operator: &str,
    left: &Expr,
    right: &Expr,
    environment: &env::Environment,
) -> Result<Value, Error> {
    let left_value = evaluate_expression(left, &environment)?;
    let right_value = evaluate_expression(right, &environment)?;

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

fn evaluate_if_expression(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
    environment: &env::Environment,
) -> Result<Value, Error> {
    let condition_value = evaluate_expression(condition, &environment)?;

    match condition_value {
        Value::Bool(true) => evaluate_expression(then_branch, &environment),
        Value::Bool(false) => evaluate_expression(else_branch, &environment),
        _ => Err(Error::UnsupportedOperation),
    }
}
