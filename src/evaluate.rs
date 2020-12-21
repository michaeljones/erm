use super::parser::{Expr, Module, Pattern, Stmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnsupportedOperation,
    UnknownFunction(String),
    UnknownBinding(String),
    WrongArity,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(String),
    List(Vec<Value>),
}

mod env {
    use super::{Expr, Module, Stmt};
    use std::collections::HashMap;

    pub type Scope<'b> = HashMap<String, &'b Expr<'b>>;
    pub type Scopes<'b> = im::Vector<Scope<'b>>;

    pub fn get_function<'a>(module: &'a Module<'a>, target_name: &str) -> Option<&'a Stmt<'a>> {
        module.statements.iter().find(|stmt| match stmt {
            Stmt::Function { name, .. } => {
                if name == &target_name {
                    true
                } else {
                    false
                }
            }
            _ => false,
        })
    }

    pub fn get_binding<'a, 'b>(
        module: &'a Module<'b>,
        scopes: &Scopes<'b>,
        target_name: &str,
    ) -> Option<&'a Expr<'b>> {
        for scope in scopes {
            if let Some(value) = scope.get(target_name) {
                return Some(&*value);
            }
        }

        module.statements.iter().find_map(|stmt| match stmt {
            Stmt::Binding { name, expr } => {
                if name == &target_name {
                    Some(expr)
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    pub fn add_scope<'a, 'b>(scopes: &'a Scopes<'b>, new_scope: Scope<'b>) -> Scopes<'b> {
        let mut new_scopes = scopes.clone();
        new_scopes.push_front(new_scope);
        new_scopes
    }
}

pub fn evaluate(module: &Module, args: Vec<String>) -> Result<Value, Error> {
    if args.is_empty() {
        let scopes = im::Vector::new();
        match env::get_binding(&module, &scopes, "main") {
            Some(expr) => evaluate_expression(&expr, &module, &scopes),
            _ => Err(Error::UnknownBinding("main".to_string())),
        }
    } else {
        let call_main = Expr::Call {
            function: "main",
            args: vec![Expr::List(
                args.iter().map(|entry| Expr::String(entry)).collect(),
            )],
        };

        let scopes = im::Vector::new();
        evaluate_expression(&call_main, &module, &scopes)
    }
}

fn evaluate_expression<'b, 'c>(
    expr: &'b Expr<'b>,
    module: &'b Module<'b>,
    scopes: &'c env::Scopes<'b>,
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
        } => evaluate_binary_expression(operator, left, right, &module, &scopes),
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => evaluate_if_expression(condition, then_branch, else_branch, &module, &scopes),
        Expr::List(items) => {
            let value_items = items
                .iter()
                .map(|expr| evaluate_expression(expr, &module, &scopes))
                .collect::<Result<Vec<Value>, Error>>()?;
            Ok(Value::List(value_items))
        }
        Expr::Call { function, args } => evaluate_function_call(function, args, &module, &scopes),
        Expr::VarName(name) => env::get_binding(&module, &scopes, name)
            .ok_or(Error::UnknownBinding(name.to_string()))
            .and_then(|bob| evaluate_expression(bob, &module, &scopes)),
    }
}

fn evaluate_function_call<'b>(
    name: &str,
    arg_exprs: &'b Vec<Expr<'b>>,
    module: &'b Module<'b>,
    scopes: &env::Scopes<'b>,
) -> Result<Value, Error> {
    if let Some(Stmt::Function { args, expr, .. }) = env::get_function(&module, name) {
        if arg_exprs.len() != args.len() {
            Err(Error::WrongArity)
        } else {
            let arg_scope = args
                .iter()
                .zip(arg_exprs.iter())
                .map(|(Pattern::Name(name), expr)| (name.to_string(), expr))
                .collect();

            let new_scope = env::add_scope(&scopes, arg_scope);
            evaluate_expression(expr, &module, &new_scope)
        }
    } else {
        Err(Error::UnknownFunction(name.to_string()))
    }
}

fn evaluate_binary_expression<'b>(
    operator: &'b str,
    left: &'b Expr<'b>,
    right: &'b Expr<'b>,
    module: &'b Module<'b>,
    scopes: &env::Scopes<'b>,
) -> Result<Value, Error> {
    let left_value = evaluate_expression(left, &module, &scopes)?;
    let right_value = evaluate_expression(right, &module, &scopes)?;

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

fn evaluate_if_expression<'b>(
    condition: &'b Expr<'b>,
    then_branch: &'b Expr<'b>,
    else_branch: &'b Expr<'b>,
    module: &'b Module<'b>,
    scopes: &env::Scopes<'b>,
) -> Result<Value, Error> {
    let condition_value = evaluate_expression(condition, &module, &scopes)?;

    match condition_value {
        Value::Bool(true) => evaluate_expression(then_branch, &module, &scopes),
        Value::Bool(false) => evaluate_expression(else_branch, &module, &scopes),
        _ => Err(Error::UnsupportedOperation),
    }
}
