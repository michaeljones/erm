use im::vector;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use super::ast::{self, Associativity, Module, Stmt};
use super::bindings::Binding;
use super::builtins;
use super::parser;
use super::project;
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

#[derive(Debug, Clone)]
pub struct ModuleImport {
    pub module_scope: Rc<ModuleScope>,
    pub exposing: Option<ast::Exposing>,
}

impl ModuleImport {
    pub fn get_binding(&self, target_name: &ast::LowerName) -> Option<Binding> {
        log::trace!(
            "ModuleImport:get_binding: {:?} from {:?}",
            &target_name,
            &self.module_scope.name
        );

        if target_name.modules == self.module_scope.name {
            // TODO: Check that target name is in exposing
            self.module_scope.get_binding(&target_name.without_module())
        } else if target_name.modules.is_empty() {
            // TODO: Check that target name is in exposing
            self.module_scope.get_binding(&target_name)
        } else {
            None
        }
    }

    pub fn get_operator(&self, target_name: &str) -> Option<Operator> {
        log::trace!(
            "ModuleImport:get_operator: {} from {:?}",
            &target_name,
            &self.module_scope.name
        );

        self.module_scope.get_operator(target_name)
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    UnableToFindModule(String),
    FailedToRead(PathBuf),
    FailedToParse(PathBuf, parser::Error),
}

#[derive(Debug)]
pub struct Scope {
    pub bindings: Bindings,
    pub operators: Operators,
}

impl Scope {
    pub fn from_bindings(bindings: Bindings) -> Self {
        log::trace!("from_bindings");
        Scope {
            bindings,
            operators: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ModuleScope {
    pub name: Vec<String>,
    pub module_imports: im::Vector<ModuleImport>,
    pub local_scope: Rc<Scope>,
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
            if let Some(value) = self.local_scope.bindings.get(target_name) {
                return Some(value.clone());
            }
        } else {
            // If there is a module part then we find the corresponding module and then look in the
            // main scope of that module for just the 'access' part of the lower name
            if let Some(import) = &self
                .module_imports
                .iter()
                .find(|import| import.module_scope.name == target_name.modules)
            {
                if let Some(value) = &import
                    .module_scope
                    .local_scope
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
        if let Some(value) = self.local_scope.operators.get(target_name) {
            return Some(value.clone());
        }

        // Backwards through list to check lowest imports first
        for import in self.module_imports.iter().rev() {
            // TODO: Filter by exposing
            if let Some(value) = &import.module_scope.get_operator(target_name) {
                return Some(value.clone());
            }
        }

        None
    }

    pub fn from_module(
        module: &Module,
        settings: &project::Settings,
    ) -> Result<ModuleScope, Error> {
        log::trace!("from_module {:?}", &module.name);
        let module_imports: im::Vector<ModuleImport> = module
            .imports
            .iter()
            .map(|import| {
                // Read & parse import.name
                let mut filenames: Vec<(PathBuf, bool)> = settings
                    .source_directories
                    .iter()
                    .map(|dir| {
                        let mut path = dir.clone();
                        path.push(format!("{}.elm", &import.module_name.join("/")));
                        (path, false)
                    })
                    .collect();

                let mut core_module_path = PathBuf::new();
                core_module_path.push("core");
                core_module_path.push(format!("{}.elm", &import.module_name.join("/")));
                filenames.push((core_module_path, true));

                let (filename, mut file, is_core) = filenames
                    .into_iter()
                    .find_map(|(path, is_core)| {
                        std::fs::File::open(&path).ok().map(|f| (path, f, is_core))
                    })
                    .ok_or(Error::UnableToFindModule(import.module_name.join(".")))?;

                let mut source = String::new();
                file.read_to_string(&mut source)
                    .map_err(|_| Error::FailedToRead(filename.clone()))?;

                let mut module = parse_source(source)
                    .map_err(|err| Error::FailedToParse(filename.clone(), err))?;

                // See readme for how Elm determines when to include prelude
                if !is_core {
                    module = ast::with_default_imports(&module);
                }

                // Create a scope from it maybe?
                Self::from_module(&module, settings).map(|module_scope| ModuleImport {
                    module_scope: Rc::new(module_scope),
                    exposing: import.exposing.clone(),
                })
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
            module_imports,
            local_scope: Rc::new(Scope {
                bindings,
                operators,
            }),
            exposing: module.exposing.clone(),
        })
    }
}

#[derive(Debug)]
pub struct Environment {
    pub module_imports: im::Vector<ModuleImport>,
    pub local_scopes: im::Vector<Rc<Scope>>,
}

impl Environment {
    pub fn from_module_scope(module_scope: ModuleScope) -> Environment {
        Environment {
            module_imports: module_scope.module_imports,
            local_scopes: vector![module_scope.local_scope],
        }
    }
}

pub enum FoundBinding {
    BuiltInFunc(Rc<dyn builtins::Func>),
    WithEnv(Binding, Environment),
}

impl std::fmt::Debug for FoundBinding {
    // Implemented because we can't derive Debug for 'dyn Func'
    // TODO: Add more detail
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoundBinding::BuiltInFunc(_) => write!(f, "FoundBinding::BuiltInFunc"),
            FoundBinding::WithEnv(_, _) => write!(f, "FoundBinding::WithEnv"),
        }
    }
}

/* Returns the binding for the target name and the environment in which that binding should be
  evaluated.
*/
pub fn get_binding(
    environment: &Environment,
    target_name: &ast::LowerName,
) -> Option<FoundBinding> {
    let full_name = target_name.to_string();
    log::trace!("get_binding: {:?}", full_name);
    match full_name.as_str() {
        // core/Basics
        "Elm.Kernel.Basics.add" => {
            return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::Add {})))
        }
        "Elm.Kernel.Basics.sub" => {
            return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::Sub {})))
        }
        "Elm.Kernel.Basics.mul" => {
            return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::Mul {})))
        }
        "Elm.Kernel.Basics.gt" => return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::Gt {}))),
        "Elm.Kernel.Basics.lt" => return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::Lt {}))),
        "Elm.Kernel.Basics.append" => {
            return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::Append {})))
        }
        // core/String
        "Elm.Kernel.String.fromInt" => {
            return Some(FoundBinding::BuiltInFunc(Rc::new(
                builtins::StringFromInt {},
            )))
        }
        "Elm.Kernel.String.join" => {
            return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::StringJoin {})))
        }
        // core/List
        "Elm.Kernel.List.sum" => {
            return Some(FoundBinding::BuiltInFunc(Rc::new(builtins::ListSum {})))
        }
        _ => {}
    }

    // TODO: Only check local scope if there is not module section to the LowerName
    for (i, scope) in environment.local_scopes.iter().enumerate() {
        if let Some(value) = scope.bindings.get(target_name) {
            let env = Environment {
                module_imports: environment.module_imports.clone(),
                local_scopes: environment.local_scopes.iter().skip(i).cloned().collect(),
            };
            return Some(FoundBinding::WithEnv(value.clone(), env));
        }
    }

    // TODO: Iterate in reverse through imports so later ones override earlier ones?
    for module_import in &environment.module_imports {
        if let Some(value) = module_import.get_binding(target_name) {
            let env = Environment {
                module_imports: module_import.module_scope.module_imports.clone(),
                local_scopes: vector![module_import.module_scope.local_scope.clone()],
            };
            return Some(FoundBinding::WithEnv(value.clone(), env));
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

    for module_import in &environment.module_imports {
        if let Some(value) = module_import.get_operator(target_name) {
            return Some(value.clone());
        }
    }

    None
}

/*
pub fn add_module_scope(environment: &Environment, new_scope: ModuleScope) -> Environment {
    log::trace!("add_module_scope");
    let mut new_scopes = environment.module_imports.clone();
    new_scopes.push_front(Rc::new(new_scope));

    Environment {
        module_scopes: new_scopes,
        local_scopes: environment.local_scopes.clone(),
    }
}
*/

pub fn add_local_scope(environment: &Environment, new_scope: Scope) -> Environment {
    log::trace!("add_local_scope");
    let mut new_scopes = environment.local_scopes.clone();
    new_scopes.push_front(Rc::new(new_scope));

    Environment {
        module_imports: environment.module_imports.clone(),
        local_scopes: new_scopes,
    }
}

pub fn new_local_scope(environment: &Environment, new_scope: Scope) -> Environment {
    log::trace!("new_local_scope");
    Environment {
        module_imports: environment.module_imports.clone(),
        local_scopes: im::vector![Rc::new(new_scope)],
    }
}
