// TODO: Rename to AST?
use std::rc::Rc;

use super::checker::term;

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub exposing: Exposing,
    pub imports: Vec<Import>,
    pub statements: Vec<Rc<Stmt>>,
}

pub fn with_default_imports(module: &Module) -> Module {
    log::trace!("with_default_imports");
    let mut imports = Import::prelude().clone();
    imports.append(&mut module.imports.clone());

    Module {
        name: module.name.clone(),
        exposing: module.exposing.clone(),
        imports,
        statements: module.statements.clone(),
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub module_name: String,
    pub exposing: Option<Exposing>,
}

impl Import {
    pub fn prelude() -> Vec<Import> {
        log::trace!("prelude");
        vec![
            Import {
                module_name: "Basics".to_string(),
                exposing: Some(Exposing::List(vec![ExposingDetail::Operator(
                    "+".to_string(),
                )])),
            },
            Import {
                module_name: "String".to_string(),
                exposing: None,
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub enum Exposing {
    All,
    List(Vec<ExposingDetail>),
}

#[derive(Debug, Clone)]
pub enum ExposingDetail {
    Operator(String),
    Name(String),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct LowerName {
    pub modules: Vec<String>,
    pub access: Vec<String>,
}

impl LowerName {
    pub fn simple(name: String) -> LowerName {
        LowerName {
            modules: Vec::new(),
            access: vec![name],
        }
    }

    pub fn to_string(&self) -> String {
        self.modules
            .iter()
            .cloned()
            .chain(self.access.iter().cloned())
            .collect()
            .join(".")
    }
}

#[derive(Debug)]
pub enum Stmt {
    Binding {
        name: String,
        expr: Rc<Expr>,
    },
    Function {
        name: String,
        args: Vec<Pattern>,
        expr: Rc<Expr>,
    },
    Infix {
        operator_name: String,
        associativity: Associativity,
        precedence: usize,
        function_name: LowerName,
    },
}

#[derive(Clone, Debug)]
pub enum Associativity {
    Left,
    Right,
    Non,
}

#[derive(Debug)]
pub enum Pattern {
    Name(String),
}

impl Pattern {
    pub fn names(&self) -> Vec<String> {
        match self {
            Pattern::Name(name) => vec![name.to_string()],
        }
    }

    pub fn term(&self) -> term::Term {
        match self {
            Pattern::Name(name) => term::Term::Var(name.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(String),
    List(Vec<Rc<Expr>>),
    BinOp {
        operator: String,
        left: Rc<Expr>,
        right: Rc<Expr>,
    },
    If {
        condition: Rc<Expr>,
        then_branch: Rc<Expr>,
        else_branch: Rc<Expr>,
    },
    Call {
        function_name: LowerName,
        args: Vec<Rc<Expr>>,
    },
    VarName(String),
}
