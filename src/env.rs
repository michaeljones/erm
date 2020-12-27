use std::collections::HashMap;
use std::rc::Rc;

use super::function::{Function, StringFromInt};
use super::parser::{Expr, Module, Stmt};

type Bindings<'src> = HashMap<String, &'src Expr<'src>>;
type Functions<'src> = HashMap<String, Function<'src>>;

pub struct Scope<'src> {
    pub bindings: Bindings<'src>,
    pub functions: Functions<'src>,
}

impl<'src> Scope<'src> {
    pub fn from_module(module: &'src Module<'src>) -> Self {
        let bindings = module
            .statements
            .iter()
            .flat_map(|entry| match entry {
                Stmt::Binding { name, expr } => Some((name.to_string(), expr)),
                _ => None,
            })
            .collect();

        let functions = module
            .statements
            .iter()
            .flat_map(|entry| match entry {
                _ => None,
            })
            .collect();

        Scope {
            bindings,
            functions,
        }
    }

    pub fn from_bindings(bindings: Bindings<'src>) -> Self {
        Scope {
            bindings,
            functions: HashMap::new(),
        }
    }
}

pub type Scopes<'src> = im::Vector<Rc<Scope<'src>>>;

pub fn get_function<'src>(scopes: &Scopes<'src>, target_name: &str) -> Option<Function<'src>> {
    match target_name {
        "stringFromInt" => return Some(Function::BuiltIn(Rc::new(StringFromInt {}))),
        _ => {}
    }

    for scope in scopes {
        if let Some(value) = scope.functions.get(target_name) {
            return Some(value.clone());
        }
    }

    None
}

pub fn get_binding<'a, 'b>(scopes: &Scopes<'b>, target_name: &str) -> Option<&'a Expr<'b>> {
    for scope in scopes {
        if let Some(value) = scope.bindings.get(target_name) {
            return Some(&*value);
        }
    }

    None

    /*
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
    */
}

pub fn add_scope<'a, 'b>(scopes: &'a Scopes<'b>, new_scope: Scope<'b>) -> Scopes<'b> {
    let mut new_scopes = scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));
    new_scopes
}
