use std::collections::HashMap;
use std::rc::Rc;

use super::builtins;
use super::function::Binding;
use super::parser::{Associativity, Module, Stmt};

#[derive(Debug, Clone)]
pub struct Operator<'src> {
    pub operator_name: &'src str,
    pub associativity: Associativity,
    pub precedence: usize,
    pub function_name: &'src str,
    pub binding: Binding<'src>,
}

pub type Bindings<'src> = HashMap<String, Binding<'src>>;
type Operators<'src> = HashMap<String, Operator<'src>>;

#[derive(Debug)]
pub struct Scope<'src> {
    pub bindings: Bindings<'src>,
    pub operators: Operators<'src>,
}

pub type Scopes<'src> = im::Vector<Rc<Scope<'src>>>;

impl<'src> Scope<'src> {
    pub fn from_module(module: &Module<'src>) -> Self {
        let bindings: Bindings<'src> = module
            .statements
            .iter()
            .flat_map(|entry| match &**entry {
                Stmt::Binding { name, expr } => {
                    Some((name.to_string(), Binding::UserBinding(expr.clone())))
                }
                Stmt::Function { name, .. } => {
                    Some((name.to_string(), Binding::UserFunc(entry.clone())))
                }
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
                } => bindings.get(&function_name.to_string()).map(|binding| {
                    (
                        operator_name.to_string(),
                        Operator {
                            operator_name,
                            associativity: associativity.clone(),
                            precedence: *precedence,
                            function_name,
                            // Store the binding for the operator's function along with the
                            // operator for easy access with checking & evaluating
                            binding: binding.clone(),
                        },
                    )
                }),

                _ => None,
            })
            .collect();

        Scope {
            bindings,
            operators,
        }
    }

    pub fn from_bindings(bindings: Bindings<'src>) -> Self {
        Scope {
            bindings,
            operators: HashMap::new(),
        }
    }
}

pub fn get_binding<'src>(scopes: &Scopes<'src>, target_name: &str) -> Option<Binding<'src>> {
    match target_name {
        "stringFromInt" => return Some(Binding::BuiltInFunc(Rc::new(builtins::StringFromInt {}))),
        "stringFromBool" => {
            return Some(Binding::BuiltInFunc(Rc::new(builtins::StringFromBool {})))
        }
        "stringJoin" => return Some(Binding::BuiltInFunc(Rc::new(builtins::StringJoin {}))),
        "Elm.Kernel.Basics.add" => return Some(Binding::BuiltInFunc(Rc::new(builtins::Add {}))),
        "Elm.Kernel.Basics.sub" => return Some(Binding::BuiltInFunc(Rc::new(builtins::Sub {}))),
        "Elm.Kernel.Basics.mul" => return Some(Binding::BuiltInFunc(Rc::new(builtins::Mul {}))),
        "Elm.Kernel.Basics.gt" => return Some(Binding::BuiltInFunc(Rc::new(builtins::Gt {}))),
        "Elm.Kernel.Basics.lt" => return Some(Binding::BuiltInFunc(Rc::new(builtins::Lt {}))),
        "Elm.Kernel.Basics.append" => {
            return Some(Binding::BuiltInFunc(Rc::new(builtins::Append {})))
        }
        _ => {}
    }

    for scope in scopes {
        if let Some(value) = scope.bindings.get(target_name) {
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

/*
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
*/

pub fn add_scope<'a, 'b>(scopes: &Scopes<'b>, new_scope: Scope<'b>) -> Scopes<'b> {
    let mut new_scopes = scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));
    new_scopes
}
