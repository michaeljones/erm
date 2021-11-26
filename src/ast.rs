use std::rc::Rc;

pub type ModuleName = Vec<String>;

#[derive(Debug)]
pub struct Module {
    pub name: ModuleName,
    pub exposing: Exposing,
    pub imports: Vec<Import>,
    pub statements: Vec<Rc<Stmt>>,
}

pub fn with_default_imports(module: &Module) -> Module {
    log::trace!("with_default_imports");
    let mut imports = Import::prelude();
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
    pub module_name: Vec<String>,
    pub exposing: Option<Exposing>,
}

impl Import {
    // How Elm determines whether to include the prelude:
    // https://github.com/elm/compiler/blob/770071accf791e8171440709effe71e78a9ab37c/compiler/src/Parse/Module.hs#L80
    pub fn prelude() -> Vec<Import> {
        log::trace!("prelude");
        vec![
            Import {
                module_name: vec!["Basics".to_string()],
                exposing: Some(Exposing::List(vec![ExposingDetail::Operator(
                    "+".to_string(),
                )])),
            },
            Import {
                module_name: vec!["String".to_string()],
                exposing: None,
            },
            Import {
                module_name: vec!["List".to_string()],
                exposing: None,
            },
            Import {
                module_name: vec!["Maybe".to_string()],
                exposing: Some(Exposing::List(vec![ExposingDetail::Type(
                    UpperName("Maybe".to_string()),
                    TypeState::Open,
                )])),
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
    Type(UpperName, TypeState),
    Operator(String),
    Name(String),
}

#[derive(Debug, Clone)]
pub enum TypeState {
    Open,
    Closed,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UpperName(pub String);

#[derive(Debug, PartialEq, Clone)]
pub struct LowerName(pub String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct QualifiedLowerName {
    pub modules: Vec<String>,
    pub access: Vec<String>,
}

impl QualifiedLowerName {
    pub fn simple(name: String) -> Self {
        Self {
            modules: Vec::new(),
            access: vec![name],
        }
    }

    pub fn from(name: String) -> Self {
        let segments = name.split('.');
        let (modules, access) = segments
            .into_iter()
            .map(|str| str.to_string())
            .partition(|name| name.starts_with(|ch| ('A'..='Z').contains(&ch)));

        Self { modules, access }
    }

    pub fn as_string(&self) -> String {
        self.modules
            .iter()
            .cloned()
            .chain(self.access.iter().cloned())
            .collect::<Vec<String>>()
            .join(".")
    }

    pub fn without_module(&self) -> Self {
        Self {
            modules: vec![],
            access: self.access.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct QualifiedUpperName {
    pub modules: Vec<String>,
    pub access: String,
}

impl QualifiedUpperName {
    pub fn from(name: &str) -> Option<Self> {
        let mut segments: Vec<String> = name
            .split('.')
            .into_iter()
            .map(|str| str.to_string())
            .collect();

        let last = segments.pop();

        last.map(|access| Self {
            modules: segments,
            access,
        })
    }

    pub fn as_string(&self) -> String {
        let mut string = self
            .modules
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join(".");

        if string.is_empty() {
            self.access.clone()
        } else {
            string.push_str(&format!(".{}", self.access));
            string
        }
    }

    pub fn without_module(&self) -> Self {
        Self {
            modules: vec![],
            access: self.access.clone(),
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Binding {
        type_annotation: Option<TypeAnnotation>,
        name: LowerName,
        expr: Rc<Expr>,
    },
    Function {
        type_annotation: Option<TypeAnnotation>,
        name: LowerName,
        args: Vec<Pattern>,
        expr: Rc<Expr>,
    },
    Infix {
        operator_name: String,
        associativity: Associativity,
        precedence: usize,
        function_name: QualifiedLowerName,
    },
    Type {
        name: UpperName,
        args: Vec<LowerName>,
        constructors: Vec<Type>,
    },
}

#[derive(Debug)]
pub struct TypeAnnotation {
    pub name: LowerName,
    pub type_: Type,
}

// Based on: https://github.com/elm-in-elm/compiler/blob/master/src/Elm/Data/Type.elm
#[derive(Debug)]
pub enum Type {
    Var(LowerName),
    Bool,
    Int,
    Float,
    Char,
    String,
    Unit,
    List(Box<Type>),
    Function {
        from: Box<Type>,
        to: Box<Type>,
    },
    UserDefined {
        name: QualifiedUpperName,
        args: Vec<Type>,
    },
}

#[derive(Clone, Debug)]
pub enum Associativity {
    Left,
    Right,
    Non,
}

// Based on: https://github.com/elm-in-elm/compiler/blob/master/src/Elm/AST/Canonical.elm#L97-L111
#[derive(Debug, Clone)]
pub enum Pattern {
    Anything,
    Bool(bool),
    Integer(i32),
    Name(String),
}

impl Pattern {
    // TODO: This feels wrong now that we have more than just 'Name'
    pub fn names(&self) -> Vec<String> {
        match self {
            Pattern::Anything => vec![],
            Pattern::Bool(_) => vec![],
            Pattern::Integer(_) => vec![],
            Pattern::Name(name) => vec![name.to_string()],
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
    Case {
        expr: Rc<Expr>,
        branches: Vec<(Pattern, Expr)>,
    },
    Call {
        function: Rc<Expr>,
        args: Vec<Rc<Expr>>,
    },
    VarName(QualifiedLowerName),
}
