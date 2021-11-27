pub mod term;
pub mod unify;

use std::rc::Rc;

use self::term::{Term, Value};
use super::ast::{self, Expr, Module, Pattern, Stmt};
use super::bindings::Binding;
use super::env::{self, FoundBinding};
use super::project;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownBinding(String),
    UnhandledExpression(String),
    UnifyError(unify::Error),
    UnknownFunction(ast::QualifiedLowerName),
    UnknownOperator(String),
    UnknownVarName(String),
    UnknownPattern(String),
    ArgumentMismatch(u32),
    TooManyArguments,
    Broken(&'static str),
    ScopeError(env::Error),
    ImpossiblyEmptyList,
    ImpossiblyEmptyCase,
    Unknown,
}

pub struct Context {
    pub next_unique_id: u32,
}

impl Context {
    pub fn new() -> Self {
        Self { next_unique_id: 1 }
    }

    pub fn unique_var(&mut self) -> Term {
        let id = self.next_unique_id;
        self.next_unique_id += 1;

        Term::Var(format!("var-{}", id))
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check(
    _module: &Module,
    environment: &env::Environment,
    _settings: &project::Settings,
) -> Result<(), Error> {
    log::trace!("check");

    let main_name = ast::QualifiedLowerName::simple("main".to_string());
    let mut context = Context::default();

    // let mut var_generator = VarGenerator::new();
    match environment.get_binding(&main_name) {
        Ok(FoundBinding::WithEnv(Binding::UserFunc(stmt_rc), _env)) => match &*stmt_rc {
            Stmt::Function { args, expr, .. } => {
                let mut bindings = env::Bindings::new();
                for arg in args {
                    for name in arg.names() {
                        bindings.insert(
                            ast::QualifiedLowerName {
                                modules: Vec::new(),
                                access: vec![name.clone()],
                            },
                            Binding::UserArg(Term::Var(name.clone())),
                        );
                    }
                }

                let scope = env::Scope::from_bindings(bindings);
                let environment = env::add_local_scope(environment, scope);

                let body_term = expression_to_term(expr, &mut context, &environment)?;
                let main_term = Term::Function(
                    Box::new(Term::Type(
                        "List".to_string(),
                        vec![Term::Constant(Value::String)],
                    )),
                    Box::new(body_term),
                );

                let target_term = Term::Function(
                    Box::new(Term::Type(
                        "List".to_string(),
                        vec![Term::Constant(Value::String)],
                    )),
                    Box::new(Term::Constant(Value::String)),
                );

                let subs = unify::Substitutions::new();
                unify::unify(&main_term, &target_term, &subs)
                    .map(|_| ())
                    .map_err(Error::UnifyError)
            }
            _ => Err(Error::UnknownBinding("main".to_string())),
        },
        entry => {
            println!("entry {:?}", entry);
            Err(Error::UnknownBinding("main".to_string()))
        }
    }
}

fn expression_to_term(
    expr: &Expr,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    log::trace!("expression_to_term: {:?}", expr);
    match expr {
        Expr::Bool(_) => Ok(Term::Constant(Value::Bool)),
        Expr::Integer(_) => Ok(Term::Constant(Value::Integer)),
        Expr::String(_) => Ok(Term::Constant(Value::String)),
        Expr::Call { function, args } => call_to_term(function, args, context, environment),
        Expr::BinOp {
            operator,
            left,
            right,
        } => binary_expression_to_term(operator, left, right, context, environment),
        Expr::VarName(name) =>
        // Want to be able to fetch 'x' from the scope where 'x' is an typed or untyped
        // argument to the function that we might be in the scope of
        {
            match environment.get_binding(name) {
                Ok(FoundBinding::BuiltInFunc(name)) => {
                    let built_in_func =
                        env::get_built_in(&name).ok_or(Error::UnknownFunction(name))?;
                    // TODO: Don't resolve with fake args - just resolve directly to a term definition
                    // for a function
                    // let args = Vec::new();
                    Ok(built_in_func.term())
                }
                Ok(FoundBinding::WithEnv(Binding::UserBinding(expr), env)) => {
                    expression_to_term(&expr, context, &env)
                }
                Ok(FoundBinding::WithEnv(Binding::UserFunc(stmt), _env)) => match &*stmt {
                    Stmt::Function { args, expr, .. } => {
                        let mut bindings = env::Bindings::new();
                        for arg in args {
                            for name in arg.names() {
                                bindings.insert(
                                    ast::QualifiedLowerName {
                                        modules: Vec::new(),
                                        access: vec![name.clone()],
                                    },
                                    Binding::UserArg(context.unique_var()),
                                );
                            }
                        }
                        let scope = env::Scope::from_bindings(bindings);
                        // TODO: The called function should probably not have the scope of the callee but
                        // rather than scope of where it was parsed
                        let environment = env::add_local_scope(environment, scope);

                        // TODO: Might infer substitutions from this work that we should return and make
                        // available
                        let body_term = expression_to_term(expr, context, &environment)?;

                        let mut signature_term = body_term;
                        for arg in args.iter().rev() {
                            signature_term = Term::Function(
                                Box::new(pattern_to_term(arg, context, &environment)?),
                                Box::new(signature_term),
                            )
                        }

                        // log::error!("Error");
                        Ok(signature_term)
                    }
                    result => {
                        log::error!("{:#?}", result);
                        Err(Error::UnknownVarName(name.as_string()))
                    }
                },
                Ok(FoundBinding::WithEnv(Binding::UserArg(term), _env)) => Ok(term),
                result => {
                    log::error!("{:#?}", result);
                    Err(Error::UnknownVarName(name.as_string()))
                }
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => if_expression_to_term(condition, then_branch, else_branch, context, environment),
        Expr::Case { expr, branches } => {
            case_expression_to_term(expr, branches, context, environment)
        }
        Expr::List(expressions) => list_to_term(expressions.to_vec(), context, environment),
        _ => Err(Error::UnhandledExpression(format!("{:?}", expr))),
    }
}

fn binary_expression_to_term(
    operator_name: &str,
    left: &Rc<Expr>,
    right: &Rc<Expr>,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    log::trace!("binary_expression_to_term");
    if let Some(operator) = env::get_operator(environment, operator_name) {
        // TODO: Make sure we get the function that corresponds to the same scope as the operator
        // otherwise we might get another function
        match operator.binding {
            Binding::UserFunc(stmt_rc) => match &*stmt_rc {
                Stmt::Function { .. } => {
                    // TODO: Figure out how to get from this function def to a usable signature for
                    // checking against with the args that we have
                    //
                    // We want to create a list of terms representing the signature. The last entry
                    // is the result of evaluating the expression which forms the body of the
                    // function. If the function has any arguments then we want to figure out terms
                    // for those to and join them to the return type to form the signature
                    //
                    // As we're processing a binary expression and the attached signature then we
                    // expect/assume there are two arguments to the function

                    // TODO: Check with the args that we have
                    // let args = vec![left.clone(), right.clone()];
                    // resolve_function_term(&signature, &args, &environment)
                    Err(Error::UnknownFunction(operator.function_name))
                }
                _ => Err(Error::UnknownFunction(operator.function_name)),
            },
            Binding::UserBinding(expr_rc) => {
                let signature_term = expression_to_term(&expr_rc, context, environment)?;
                let left_term = expression_to_term(left, context, environment)?;
                let right_term = expression_to_term(right, context, environment)?;
                let arg_terms = [left_term, right_term];
                // println!("About to resolve for {:#?}", expr_rc);
                resolve_function_and_args(&signature_term, &arg_terms, environment)
            }
            _ => Err(Error::UnknownFunction(operator.function_name)),
        }
    } else {
        Err(Error::UnknownOperator(operator_name.to_string()))
    }
}

fn call_to_term(
    function: &Rc<Expr>,
    call_args: &[Rc<Expr>],
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    log::trace!("call_to_term");
    let function_term = expression_to_term(function, context, environment)?;

    let arg_terms = call_args
        .iter()
        .map(|arg| expression_to_term(arg, context, environment))
        .collect::<Result<Vec<Term>, Error>>()?;

    // println!("About to resolve for builtin {:?}", function_name);
    resolve_function_and_args(&function_term, &arg_terms, environment)
}

/* Takes a function signature expressed as terms and arguments expressed as terms and applies the
 * arguments to the signature to resolve down to a shorter signature or a single non-function term
 */
fn resolve_function_and_args(
    signature_term: &Term,
    arg_terms: &[Term],
    environment: &env::Environment,
) -> Result<Term, Error> {
    log::trace!(
        "resolve_function_and_args: {:?} {:?}",
        signature_term,
        arg_terms
    );
    match signature_term {
        Term::Function(from, to) => match arg_terms.split_first() {
            Some((first, [])) => {
                let subs = unify::Substitutions::new();
                match unify::unify(first, &**from, &subs) {
                    Ok(_subs) => Ok((**to).clone()),
                    Err(err) => Err(Error::UnifyError(err)),
                }
            }
            Some((first, rest)) => {
                let subs = unify::Substitutions::new();
                match unify::unify(first, &**from, &subs) {
                    Ok(_subs) => resolve_function_and_args(to, rest, environment),
                    Err(err) => Err(Error::UnifyError(err)), // TODO: If they don't already match then we want to try to unify them - in
                                                             // particular to detect if one is more general than the other and that they can
                                                             // therefore be brought together by narrowing the more general one down and fixing
                                                             // it to match the more specific one
                                                             // println!("first {:#?}", first);
                                                             // println!("from {:#?}", from);
                                                             // Err(Error::Broken("arg didn't match function slot"))
                }
            }
            None => Err(Error::Broken("no more args")),
        },
        term => {
            log::error!("{:?}", term);
            Err(Error::Broken("signature is not a function"))
        }
    }
}

fn if_expression_to_term(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    log::trace!("if_expression_to_term");
    // Infer condition
    let condition_term = expression_to_term(condition, context, environment)?;

    // Unify condition
    let subs = unify::Substitutions::new();
    let _subs = unify::unify(&condition_term, &Term::Constant(Value::Bool), &subs)
        .map_err(Error::UnifyError)?;

    // Infer then_branch
    let then_branch_term = expression_to_term(then_branch, context, environment)?;

    // Infer else_branch
    let else_branch_term = expression_to_term(else_branch, context, environment)?;

    // TODO: Unify instead of equality check
    if then_branch_term == else_branch_term {
        Ok(then_branch_term)
    } else {
        Err(Error::Broken("else & then don't match"))
    }
}

fn case_expression_to_term(
    expr: &Expr,
    branches: &[(Pattern, Expr)],
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    log::trace!("case_expression_to_term");
    let expr_term = expression_to_term(expr, context, environment)?;
    let subs = unify::Substitutions::new();

    let mut branch_expr_term = None;

    for (pattern, branch_expr) in branches {
        let pattern_term = pattern_to_term(pattern, context, environment)?;
        unify::unify(&expr_term, &pattern_term, &subs).map_err(Error::UnifyError)?;

        branch_expr_term = Some(expression_to_term(branch_expr, context, environment)?);
    }

    branch_expr_term.ok_or(Error::ImpossiblyEmptyCase)
}

fn pattern_to_term(
    pattern: &Pattern,
    context: &mut Context,
    _environment: &env::Environment,
) -> Result<Term, Error> {
    match pattern {
        Pattern::Anything => Ok(context.unique_var()),
        Pattern::Bool(_) => Ok(Term::Constant(Value::Bool)),
        Pattern::Integer(_) => Ok(Term::Constant(Value::Integer)),
        Pattern::Name(name) => Ok(term::Term::Var(name.to_string())),
    }
}

fn list_to_term(
    expressions: Vec<Rc<Expr>>,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    log::trace!("list_to_term");
    if expressions.is_empty() {
        Ok(Term::Type("List".to_string(), vec![context.unique_var()]))
    } else {
        let terms: Vec<Term> = expressions
            .iter()
            .map(|expr| expression_to_term(expr, context, environment))
            .collect::<Result<_, _>>()?;

        // Unify terms by comparing each item with its neighbour and making sure there are no
        // issues unifying them with a consistent set of subs
        let (_subs, term) = terms
            .iter()
            .fold(Err(Error::ImpossiblyEmptyList), |acc, term| match acc {
                Err(Error::ImpossiblyEmptyList) => Ok((unify::Substitutions::new(), term)),
                Err(err) => Err(err),
                Ok((subs, last_term)) => unify::unify(term, last_term, &subs)
                    .map(|subs| (subs, term))
                    .map_err(Error::UnifyError),
            })?;

        // TODO: What is the best term to actually include from the list? The most basic? The most
        // general?

        Ok(Term::Type("List".to_string(), vec![term.clone()]))
    }
}
