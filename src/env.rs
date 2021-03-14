use std::collections::HashMap;
use std::rc::Rc;

use super::builtins;
use super::function::Binding;
use super::module::{Associativity, Module, Stmt};

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

#[derive(Debug)]
pub struct Environment<'src> {
    pub module_scopes: im::Vector<Rc<Scope<'src>>>,
    pub local_scopes: im::Vector<Rc<Scope<'src>>>,
}

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

pub fn get_binding<'src>(
    environment: &Environment<'src>,
    target_name: &str,
) -> Option<Binding<'src>> {
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

    for scope in &environment.local_scopes {
        if let Some(value) = scope.bindings.get(target_name) {
            return Some(value.clone());
        }
    }

    for scope in &environment.module_scopes {
        if let Some(value) = scope.bindings.get(target_name) {
            return Some(value.clone());
        }
    }

    None
}

pub fn get_operator<'a, 'src>(
    environment: &Environment<'src>,
    target_name: &str,
) -> Option<Operator<'src>> {
    for scope in &environment.local_scopes {
        if let Some(value) = scope.operators.get(target_name) {
            return Some(value.clone());
        }
    }

    for scope in &environment.module_scopes {
        if let Some(value) = scope.operators.get(target_name) {
            return Some(value.clone());
        }
    }

    None
}

pub fn add_module_scope<'b>(
    environment: &Environment<'b>,
    new_scope: Scope<'b>,
) -> Environment<'b> {
    let mut new_scopes = environment.module_scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));

    Environment {
        module_scopes: new_scopes,
        local_scopes: environment.local_scopes.clone(),
    }
}

pub fn add_local_scope<'b>(environment: &Environment<'b>, new_scope: Scope<'b>) -> Environment<'b> {
    let mut new_scopes = environment.local_scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));

    Environment {
        module_scopes: environment.module_scopes.clone(),
        local_scopes: new_scopes,
    }
}
pub fn new_local_scope<'b>(environment: &Environment<'b>, new_scope: Scope<'b>) -> Environment<'b> {
    Environment {
        module_scopes: environment.module_scopes.clone(),
        local_scopes: im::vector![Rc::new(new_scope)],
    }
}
