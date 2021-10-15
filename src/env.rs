use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;

use super::ast::{self, Associativity, Module, Stmt};
use super::bindings::Binding;
use super::builtins;
use super::parser;
use crate::parse_source;

#[derive(Debug, Clone)]
pub struct Operator {
    pub operator_name: String,
    pub associativity: Associativity,
    pub precedence: usize,
    pub function_name: ast::LowerName,
    pub binding: Binding,
}

pub type Bindings = HashMap<ast::LowerName, Binding>;
type Operators = HashMap<String, Operator>;

#[derive(Debug)]
pub struct Scope {
    pub bindings: Bindings,
    pub operators: Operators,
}

#[derive(Debug)]
pub struct ModuleScope {
    pub name: Vec<String>,
    pub internal_scopes: im::Vector<Rc<ModuleScope>>,
    pub main_scope: Scope,
    pub exposing: ast::Exposing,
}

impl ModuleScope {
    pub fn get_binding(&self, target_name: &ast::LowerName) -> Option<Binding> {
        log::trace!(
            "ModuleScope:get_binding: {:?} from {:?}",
            &target_name,
            &self.name
        );

        // If there is no module part of the lower name then we look in the main scope for the
        // target name
        if target_name.modules.is_empty() {
            if let Some(value) = self.main_scope.bindings.get(target_name) {
                return Some(value.clone());
            }
        } else {
            // If there is a module part then we find the corresponding module and then look in the
            // main scope of that module for just the 'access' part of the lower name
            if let Some(module_scope) = &self
                .internal_scopes
                .iter()
                .find(|scope| scope.name == target_name.modules)
            {
                if let Some(value) = &module_scope
                    .main_scope
                    .bindings
                    .get(&target_name.without_module())
                {
                    return Some((*value).clone());
                }
            }
        }

        // TODO: Filter by exposing

        None
    }

    pub fn get_operator(&self, target_name: &str) -> Option<Operator> {
        log::trace!(
            "ModuleScope:get_operator: {} from {:?}",
            &target_name,
            &self.name
        );

        // TODO: Filter by exposing
        if let Some(value) = self.main_scope.operators.get(target_name) {
            return Some(value.clone());
        }

        // Backwards through list to check lowest imports first
        for scope in self.internal_scopes.iter().rev() {
            // TODO: Filter by exposing
            if let Some(value) = &scope.get_operator(target_name) {
                return Some(value.clone());
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct Environment {
    pub module_scopes: im::Vector<Rc<ModuleScope>>,
    pub local_scopes: im::Vector<Rc<Scope>>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    FileNotFound(String),
    FailedToRead(String),
    FailedToParse(String, parser::Error),
}

impl Scope {
    pub fn from_module(module: &Module) -> Result<ModuleScope, Error> {
        log::trace!("from_module {:?}", &module.name);
        let internal_scopes: im::Vector<Rc<ModuleScope>> = module
            .imports
            .iter()
            .map(|import| {
                // Read & parse import.name
                let filename = format!("core/{}.elm", &import.module_name.join("/"));
                let mut file = std::fs::File::open(&filename)
                    .map_err(|_| Error::FileNotFound(filename.clone()))?;

                let mut source = String::new();
                file.read_to_string(&mut source)
                    .map_err(|_| Error::FailedToRead(filename.clone()))?;

                let module = parse_source(source)
                    .map_err(|err| Error::FailedToParse(filename.clone(), err))?;

                // Create a scope from it maybe?
                Self::from_module(&module).map(Rc::new)
            })
            .collect::<Result<_, _>>()?;

        let bindings: Bindings = module
            .statements
            .iter()
            .flat_map(|entry| match &**entry {
                Stmt::Binding { name, expr } => Some((
                    ast::LowerName {
                        modules: Vec::new(),
                        access: vec![name.to_string()],
                    },
                    Binding::UserBinding(expr.clone()),
                )),
                Stmt::Function { name, .. } => Some((
                    ast::LowerName {
                        modules: Vec::new(),
                        access: vec![name.to_string()],
                    },
                    Binding::UserFunc(entry.clone()),
                )),
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
                } => bindings.get(&function_name).map(|binding| {
                    (
                        operator_name.to_string(),
                        Operator {
                            operator_name: operator_name.clone(),
                            associativity: associativity.clone(),
                            precedence: *precedence,
                            function_name: function_name.clone(),
                            // Store the binding for the operator's function along with the
                            // operator for easy access with checking & evaluating
                            binding: binding.clone(),
                        },
                    )
                }),

                _ => None,
            })
            .collect();

        Ok(ModuleScope {
            name: module.name.clone(),
            internal_scopes,
            main_scope: Scope {
                bindings,
                operators,
            },
            exposing: module.exposing.clone(),
        })
    }

    pub fn from_bindings(bindings: Bindings) -> Self {
        log::trace!("from_bindings");
        Scope {
            bindings,
            operators: HashMap::new(),
        }
    }
}

pub fn get_binding(environment: &Environment, target_name: &ast::LowerName) -> Option<Binding> {
    let full_name = target_name.to_string();
    log::trace!("get_binding: {:?}", full_name);
    match full_name.as_str() {
        "Elm.Kernel.String.fromInt" => {
            return Some(Binding::BuiltInFunc(Rc::new(builtins::StringFromInt {})))
        }
        "Elm.Kernel.String.join" => {
            return Some(Binding::BuiltInFunc(Rc::new(builtins::StringJoin {})))
        }
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
        if let Some(value) = scope.get_binding(target_name) {
            return Some(value.clone());
        }
    }

    None
}

pub fn get_operator<'a, 'src>(environment: &Environment, target_name: &str) -> Option<Operator> {
    log::trace!("get_operator: {}", &target_name);
    for scope in &environment.local_scopes {
        if let Some(value) = scope.operators.get(target_name) {
            return Some(value.clone());
        }
    }

    for scope in &environment.module_scopes {
        if let Some(value) = scope.get_operator(target_name) {
            return Some(value.clone());
        }
    }

    None
}

pub fn add_module_scope(environment: &Environment, new_scope: ModuleScope) -> Environment {
    log::trace!("add_module_scope");
    let mut new_scopes = environment.module_scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));

    Environment {
        module_scopes: new_scopes,
        local_scopes: environment.local_scopes.clone(),
    }
}

pub fn add_local_scope(environment: &Environment, new_scope: Scope) -> Environment {
    log::trace!("add_local_scope");
    let mut new_scopes = environment.local_scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));

    Environment {
        module_scopes: environment.module_scopes.clone(),
        local_scopes: new_scopes,
    }
}

pub fn new_local_scope(environment: &Environment, new_scope: Scope) -> Environment {
    log::trace!("new_local_scope");
    Environment {
        module_scopes: environment.module_scopes.clone(),
        local_scopes: im::vector![Rc::new(new_scope)],
    }
}
