pub mod term;
pub mod unify;

use std::rc::Rc;

use self::term::{Term, Value};
use super::ast::{self, Expr, Module, Stmt};
use super::bindings::Binding;
use super::env::{self, FoundBinding};
use super::project;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownBinding(String),
    UnhandledExpression(String, u32),
    UnifyError(unify::Error),
    UnknownFunction(ast::LowerName, u32),
    UnknownOperator(String),
    UnknownVarName(String, u32),
    ArgumentMismatch(u32),
    TooManyArguments,
    Broken(&'static str, u32),
    ScopeError(env::Error),
    ImpossiblyEmptyList,
}

pub struct Context {
    pub next_unique_id: u32,
}

impl Context {
    pub fn new() -> Context {
        Context { next_unique_id: 1 }
    }

    pub fn unique_var(&mut self) -> Term {
        let id = self.next_unique_id;
        self.next_unique_id += 1;

        Term::Var(format!("var-{}", id))
    }
}

pub fn check(
    _module: &Module,
    environment: &env::Environment,
    _settings: &project::Settings,
) -> Result<(), Error> {
    log::trace!("check");

    let main_name = ast::LowerName::simple("main".to_string());
    let mut context = Context::new();

    // let mut var_generator = VarGenerator::new();
    match env::get_binding(&environment, &main_name) {
        Ok(FoundBinding::WithEnv(Binding::UserBinding(expr), _env)) => {
            // Generate Term version of Expr tree
            //
            // Unify Term version of Expr tree with a simple Term of:
            //
            //   Constant(Value::String)
            //
            // if "main" is a binding and
            //
            //   Function { name: "main", args: vec![Constant(Value::List(Value::String))]
            //
            // if "main" is a function.
            //
            // TODO: Inspect any explicitly written type on main and resolve against that?

            let main_term = expression_to_term(&expr, &mut context, &environment)?;
            let target_term = Term::Constant(Value::String);
            let subs = unify::Substitutions::new();

            // println!("main_term {:#?}", main_term);

            unify::unify(&main_term, &target_term, &subs)
                .map(|_| ())
                .map_err(Error::UnifyError)
        }
        Ok(FoundBinding::WithEnv(Binding::UserFunc(stmt_rc), _env)) => match &*stmt_rc {
            Stmt::Function { expr, args, .. } => {
                let mut bindings = env::Bindings::new();
                for arg in args {
                    for name in arg.names() {
                        bindings.insert(
                            ast::LowerName {
                                modules: Vec::new(),
                                access: vec![name.clone()],
                            },
                            Binding::UserArg(Term::Var(name.clone())),
                        );
                    }
                }

                let scope = env::Scope::from_bindings(bindings);
                let environment = env::add_local_scope(&environment, scope);

                let body_term = expression_to_term(&expr, &mut context, &environment)?;
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
    println!("expr {:#?}", expr);
    match expr {
        Expr::Bool(_) => Ok(Term::Constant(Value::Bool)),
        Expr::Integer(_) => Ok(Term::Constant(Value::Integer)),
        Expr::String(_) => Ok(Term::Constant(Value::String)),
        Expr::Call {
            function_name,
            args,
        } => call_to_term(function_name, args, context, &environment),
        Expr::BinOp {
            operator,
            left,
            right,
        } => binary_expression_to_term(operator, left, right, context, &environment),
        Expr::VarName(name) =>
        // Want to be able to fetch 'x' from the scope where 'x' is an typed or untyped
        // argument to the function that we might be in the scope of
        {
            match env::get_binding(&environment, &name) {
                Ok(FoundBinding::BuiltInFunc(func)) => {
                    // TODO: Don't resolve with fake args - just resolve directly to a term definition
                    // for a function
                    // let args = Vec::new();
                    Ok(func.term())
                }
                Ok(FoundBinding::WithEnv(Binding::UserBinding(expr), env)) => {
                    expression_to_term(&expr, context, &env)
                }
                Ok(FoundBinding::WithEnv(Binding::UserArg(term), _env)) => Ok(term.clone()),
                result => {
                    // println!("Environment: {:#?}", environment);
                    println!("Result: {:#?}", result);
                    Err(Error::UnknownVarName(name.to_string(), line!()))
                }
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => if_expression_to_term(
            &condition,
            &then_branch,
            &else_branch,
            context,
            &environment,
        ),
        Expr::List(expressions) => list_to_term(&expressions, context, &environment),
        _ => Err(Error::UnhandledExpression(format!("{:?}", expr), line!())),
    }
}

fn binary_expression_to_term(
    operator_name: &str,
    left: &Rc<Expr>,
    right: &Rc<Expr>,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    // println!("{:#?}", environment);
    if let Some(operator) = env::get_operator(&environment, operator_name) {
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
                    Err(Error::UnknownFunction(operator.function_name, line!()))
                }
                _ => Err(Error::UnknownFunction(operator.function_name, line!())),
            },
            Binding::UserBinding(expr_rc) => {
                let signature_term = expression_to_term(&expr_rc, context, &environment)?;
                let left_term = expression_to_term(left, context, &environment)?;
                let right_term = expression_to_term(right, context, &environment)?;
                let arg_terms = [left_term, right_term];
                println!("About to resolve for {:#?}", expr_rc);
                dbg!(resolve_function_and_args(
                    &signature_term,
                    &arg_terms,
                    &environment
                ))
            }
            _ => Err(Error::UnknownFunction(operator.function_name, line!())),
        }
    } else {
        Err(Error::UnknownOperator(operator_name.to_string()))
    }
}

fn call_to_term(
    function_name: &ast::LowerName,
    call_args: &Vec<Rc<Expr>>,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    match env::get_binding(&environment, function_name) {
        Ok(FoundBinding::BuiltInFunc(func)) => {
            let signature_term = func.term();

            let arg_terms = call_args
                .iter()
                .map(|arg| expression_to_term(&arg, context, &environment))
                .collect::<Result<Vec<Term>, Error>>()?;

            println!("About to resolve for builtin {:?}", function_name);
            dbg!(resolve_function_and_args(
                &signature_term,
                &arg_terms,
                &environment
            ))
        }
        Ok(FoundBinding::WithEnv(Binding::UserFunc(stmt_rc), _env)) => match &*stmt_rc {
            Stmt::Function {
                name: _,
                args,
                expr,
            } => {
                // We want to take the args and create a scope out of them where if the body
                // expression has one of them then we can fetch the term for it. It would be nice
                // if this could be built into the 'environment' interface but we use the
                // environment for
                // evaluating our ast as well so it doesn't make much sense to be able to return
                // something that is just a term. On the other hand we also fetch built in
                // functions from the scope and then ask those for their terms so there is
                // precendent.
                //
                // In that case we'd add some kind of binding to the scope such that we could
                // insert our argument object along with its current term (which is unknown as we
                // don't support types for them yet.)
                let mut bindings = env::Bindings::new();
                for arg in args {
                    for name in arg.names() {
                        bindings.insert(
                            ast::LowerName {
                                modules: Vec::new(),
                                access: vec![name.clone()],
                            },
                            Binding::UserArg(Term::Var(name.clone())),
                        );
                    }
                }
                let scope = env::Scope::from_bindings(bindings);
                // TODO: The called function should probably not have the scope of the callee but
                // rather than scope of where it was parsed
                let environment = env::add_local_scope(&environment, scope);

                // TODO: Might infer substitutions from this work that we should return and make
                // available
                let body_term = expression_to_term(&expr, context, &environment)?;

                println!("body_term {:?}", body_term);

                // Figure out signature for function by creating nested function terms using the
                // function signature arguments
                let mut signature_term = body_term;
                for arg in args.iter().rev() {
                    signature_term = Term::Function(Box::new(arg.term()), Box::new(signature_term))
                }

                // Having got the signature for the function that we're calling
                // Create terms for the args being passed to the function
                let arg_terms = call_args
                    .iter()
                    .map(|arg| expression_to_term(&arg, context, &environment))
                    .collect::<Result<Vec<Term>, Error>>()?;

                // Result the arguments against the signature
                dbg!(resolve_function_and_args(
                    &signature_term,
                    &arg_terms,
                    &environment
                ))
            }
            _ => Err(Error::UnknownFunction(function_name.clone(), line!())),
        },
        Ok(FoundBinding::WithEnv(Binding::UserBinding(expr), _env)) => {
            println!("expr {:#?}", expr);
            match &*expr {
                Expr::VarName(lower_name) => {
                    call_to_term(lower_name, call_args, context, environment)
                }
                _ => Err(Error::UnknownFunction(function_name.clone(), line!())),
            }
        }
        value => {
            println!("value {:#?}", value);
            println!("value {:#?}", environment.module_imports);
            Err(Error::UnknownFunction(function_name.clone(), line!()))
        }
    }
}

/* Takes a function signature expressed as terms and arguments expressed as terms and applies the
 * arguments to the signature to resolve down to a shorter signature or a single non-function term
 */
fn resolve_function_and_args<'a, 'b, 'src>(
    signature_term: &Term,
    arg_terms: &[Term],
    environment: &'b env::Environment,
) -> Result<Term, Error> {
    println!("signature_term {:#?}", signature_term);
    println!("arg_terms {:#?}", arg_terms);
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
                // TODO: Unify instead of equality check
                let subs = unify::Substitutions::new();
                match unify::unify(first, &**from, &subs) {
                    Ok(_subs) => resolve_function_and_args(to, rest, &environment),
                    Err(err) => Err(Error::UnifyError(err)), // TODO: If they don't already match then we want to try to unify them - in
                                                             // particular to detect if one is more general than the other and that they can
                                                             // therefore be brought together by narrowing the more general one down and fixing
                                                             // it to match the more specific one
                                                             // println!("first {:#?}", first);
                                                             // println!("from {:#?}", from);
                                                             // Err(Error::Broken("arg didn't match function slot", line!()))
                }
            }
            None => Err(Error::Broken("no more args", line!())),
        },
        _ => Err(Error::Broken("signature is not a function", line!())),
    }
}

fn if_expression_to_term(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
    // Infer condition
    let condition_term = expression_to_term(condition, context, &environment)?;

    // Unify condition
    let subs = unify::Substitutions::new();
    let _subs = unify::unify(&condition_term, &Term::Constant(Value::Bool), &subs)
        .map_err(Error::UnifyError)?;

    // Infer then_branch
    let then_branch_term = expression_to_term(then_branch, context, &environment)?;

    // Infer else_branch
    let else_branch_term = expression_to_term(else_branch, context, &environment)?;

    // TODO: Unify instead of equality check
    if then_branch_term == else_branch_term {
        Ok(then_branch_term)
    } else {
        Err(Error::Broken("else & then don't match", line!()))
    }
}

fn list_to_term(
    expressions: &Vec<Rc<Expr>>,
    context: &mut Context,
    environment: &env::Environment,
) -> Result<Term, Error> {
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
