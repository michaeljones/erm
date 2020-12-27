use std::collections::HashMap;
use std::rc::Rc;

use super::function::{Function, StringFromInt};
use super::parser::{Associativity, Expr, Module, Stmt};

#[derive(Debug, Clone)]
pub struct Operator<'src> {
    pub operator_name: &'src str,
    pub associativity: Associativity,
    pub precedence: usize,
    pub function_name: &'src str,
}

type Bindings<'src> = HashMap<String, Rc<Expr<'src>>>;
type Functions<'src> = HashMap<String, Function<'src>>;
type Operators<'src> = HashMap<String, Operator<'src>>;

pub struct Scope<'src> {
    pub bindings: Bindings<'src>,
    pub functions: Functions<'src>,
    pub operators: Operators<'src>,
}

pub type Scopes<'src> = im::Vector<Rc<Scope<'src>>>;

impl<'src> Scope<'src> {
    pub fn from_module(module: &Module<'src>) -> Self {
        let bindings = module
            .statements
            .iter()
            .flat_map(|entry| match &**entry {
                Stmt::Binding { name, expr } => Some((name.to_string(), expr.clone())),
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

        let operators = module
            .statements
            .iter()
            .flat_map(|entry| match &**entry {
                Stmt::Infix {
                    operator_name,
                    associativity,
                    precedence,
                    function_name,
                } => Some((
                    operator_name.to_string(),
                    Operator {
                        operator_name,
                        associativity: associativity.clone(),
                        precedence: *precedence,
                        function_name,
                    },
                )),
                _ => None,
            })
            .collect();

        Scope {
            bindings,
            functions,
            operators,
        }
    }

    pub fn from_bindings(bindings: Bindings<'src>) -> Self {
        Scope {
            bindings,
            functions: HashMap::new(),
            operators: HashMap::new(),
        }
    }
}

pub fn get_function<'a, 'b, 'src>(
    scopes: &'b Scopes<'src>,
    target_name: &str,
) -> Option<Function<'src>> {
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

pub fn get_operator<'a, 'src>(scopes: &Scopes<'src>, target_name: &str) -> Option<Operator<'src>> {
    for scope in scopes {
        if let Some(value) = scope.operators.get(target_name) {
            return Some(value.clone());
        }
    }

    None
}

pub fn get_binding<'src>(scopes: &Scopes<'src>, target_name: &str) -> Option<Rc<Expr<'src>>> {
    for scope in scopes {
        if let Some(value) = scope.bindings.get(target_name) {
            return Some(value.clone());
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

pub fn add_scope<'a, 'b>(scopes: &Scopes<'b>, new_scope: Scope<'b>) -> Scopes<'b> {
    let mut new_scopes = scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));
    new_scopes
}
