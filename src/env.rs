use super::evaluater::{Function, StringFromInt};
use super::parser::{Expr, Module, Stmt};
use std::collections::HashMap;

pub type Scope<'b> = HashMap<String, &'b Expr<'b>>;
pub type Scopes<'b> = im::Vector<Scope<'b>>;

pub fn get_function<'a>(module: &'a Module<'a>, target_name: &str) -> Option<Function<'a>> {
    match target_name {
        "stringFromInt" => return Some(Function::BuiltIn(Box::new(StringFromInt {}))),
        _ => {}
    }

    module.statements.iter().find_map(|stmt| match stmt {
        Stmt::Function { name, .. } => {
            if name == &target_name {
                Some(Function::UserDefined(stmt))
            } else {
                None
            }
        }
        _ => None,
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
