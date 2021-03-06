pub mod term;
pub mod unify;

use std::rc::Rc;

use self::term::{Term, Value};
use super::env;
use super::function::Binding;
use super::parser::{Expr, Module, Stmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownBinding(String),
    UnhandledExpression(String),
    UnifyError(unify::Error),
    UnknownFunction(String, u32),
    UnknownOperator(String),
    ArgumentMismatch(u32),
    TooManyArguments,
    Broken(&'static str, u32),
}

pub fn check<'src>(module: &Module<'src>, scopes: &env::Scopes<'src>) -> Result<(), Error> {
    let scope = env::Scope::from_module(&module);
    let scopes = env::add_scope(&scopes, scope);

    // let mut var_generator = VarGenerator::new();
    match env::get_binding(&scopes, "main") {
        Some(Binding::UserBinding(expr)) => {
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

            let main_term = expression_to_term(&expr, &scopes)?;
            let target_term = Term::Constant(Value::String);
            let subs = unify::Substitutions::new();

            // println!("main_term {:#?}", main_term);

            unify::unify(&main_term, &target_term, &subs)
                .map(|_| ())
                .map_err(Error::UnifyError)
        }
        Some(Binding::UserFunc(stmt_rc)) => match &*stmt_rc {
            Stmt::Function { expr, args, .. } => {
                println!("matched function");

                let mut bindings = env::Bindings::new();
                for arg in args {
                    for name in arg.names() {
                        bindings.insert(name.clone(), Binding::UserArg(Term::Var(name.clone())));
                    }
                }
                let scope = env::Scope::from_bindings(bindings);
                // TODO: The called function should probably not have the scope of the callee but
                // rather than scope of where it was parsed
                let scopes = env::add_scope(&scopes, scope);

                let body_term = expression_to_term(&expr, &scopes)?;
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

fn expression_to_term<'a, 'b, 'src>(
    expr: &'a Expr<'src>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
    println!("expr {:#?}", expr);
    match expr {
        Expr::Bool(_) => Ok(Term::Constant(Value::Bool)),
        Expr::Integer(_) => Ok(Term::Constant(Value::Integer)),
        Expr::String(_) => Ok(Term::Constant(Value::String)),
        Expr::Call {
            function_name,
            args,
        } => call_to_term(function_name, args, &scopes),
        Expr::BinOp {
            operator,
            left,
            right,
        } => binary_expression_to_term(operator, left, right, &scopes),
        Expr::VarName(name) =>
        // Want to be able to fetch 'x' from the scope where 'x' is an typed or untyped
        // argument to the function that we might be in the scope of
        {
            match env::get_binding(&scopes, name) {
                Some(Binding::BuiltInFunc(func)) => {
                    // TODO: Don't resolve with fake args - just resolve directly to a term definition
                    // for a function
                    // let args = Vec::new();
                    Ok(func.term())
                }
                Some(Binding::UserArg(term)) => Ok(term.clone()),
                _ => Err(Error::UnhandledExpression("var name lookup".to_string())),
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => if_expression_to_term(&condition, &then_branch, &else_branch, &scopes),
        _ => Err(Error::UnhandledExpression(format!("{:?}", expr))),
    }
}

fn binary_expression_to_term<'a, 'b, 'src>(
    operator_name: &'src str,
    left: &Rc<Expr<'src>>,
    right: &Rc<Expr<'src>>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
    // println!("{:#?}", scopes);
    if let Some(operator) = env::get_operator(&scopes, operator_name) {
        // TODO: Make sure we get the function that corresponds to the same scope as the operator
        // otherwise we might get another function
        match env::get_binding(&scopes, operator.function_name) {
            Some(Binding::UserFunc(stmt_rc)) => match &*stmt_rc {
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
                    // resolve_function_term(&signature, &args, &scopes)
                    Err(Error::UnknownFunction(
                        operator.function_name.to_string(),
                        line!(),
                    ))
                }
                _ => Err(Error::UnknownFunction(
                    operator.function_name.to_string(),
                    line!(),
                )),
            },
            Some(Binding::UserBinding(expr_rc)) => {
                let signature_term = expression_to_term(&expr_rc, &scopes)?;
                let left_term = expression_to_term(left, &scopes)?;
                let right_term = expression_to_term(right, &scopes)?;
                let arg_terms = [left_term, right_term];
                println!("About to resolve for {:#?}", expr_rc);
                dbg!(resolve_function_and_args(
                    &signature_term,
                    &arg_terms,
                    &scopes
                ))
            }
            Some(Binding::BuiltInFunc(_func)) => {
                // let args = vec![left.clone(), right.clone()];
                // built_in_to_term(func, &args, &scopes)
                Err(Error::Broken(
                    "no support for binary with built-in",
                    line!(),
                ))
            }
            _ => Err(Error::UnknownFunction(
                operator.function_name.to_string(),
                line!(),
            )),
        }
    } else {
        Err(Error::UnknownOperator(operator_name.to_string()))
    }
}

fn call_to_term<'a, 'b, 'src>(
    function_name: &'src str,
    call_args: &'a Vec<Rc<Expr<'src>>>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
    match env::get_binding(&scopes, function_name) {
        Some(Binding::UserFunc(stmt_rc)) => match &*stmt_rc {
            Stmt::Function {
                name: _,
                args,
                expr,
            } => {
                // We want to take the args and create a scope out of them where if the body
                // expression has one of them then we can fetch the term for it. It would be nice
                // if this could be built into the 'scopes' interface but we use the scopes for
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
                        bindings.insert(name.clone(), Binding::UserArg(Term::Var(name.clone())));
                    }
                }
                let scope = env::Scope::from_bindings(bindings);
                // TODO: The called function should probably not have the scope of the callee but
                // rather than scope of where it was parsed
                let scopes = env::add_scope(&scopes, scope);

                // TODO: Might infer substitutions from this work that we should return and make
                // available
                let body_term = expression_to_term(&expr, &scopes)?;

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
                    .map(|arg| expression_to_term(&arg, &scopes))
                    .collect::<Result<Vec<Term>, Error>>()?;

                // Result the arguments against the signature
                dbg!(resolve_function_and_args(
                    &signature_term,
                    &arg_terms,
                    &scopes
                ))
            }
            _ => Err(Error::UnknownFunction(function_name.to_string(), line!())),
        },
        Some(Binding::BuiltInFunc(func)) => {
            let signature_term = func.term();

            let arg_terms = call_args
                .iter()
                .map(|arg| expression_to_term(&arg, &scopes))
                .collect::<Result<Vec<Term>, Error>>()?;

            println!("About to resolve for builtin {:?}", function_name);
            dbg!(resolve_function_and_args(
                &signature_term,
                &arg_terms,
                &scopes
            ))
        }
        value => {
            println!("value {:#?}", value);
            Err(Error::UnknownFunction(function_name.to_string(), line!()))
        }
    }
}

/* Takes a function signature expressed as terms and arguments expressed as terms and applies the
 * arguments to the signature to resolve down to a shorter signature or a single non-function term
 */
fn resolve_function_and_args<'a, 'b, 'src>(
    signature_term: &Term,
    arg_terms: &[Term],
    scopes: &'b env::Scopes<'src>,
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
                    Ok(_subs) => resolve_function_and_args(to, rest, &scopes),
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

fn if_expression_to_term<'a, 'b, 'src>(
    condition: &'a Expr<'src>,
    then_branch: &'a Expr<'src>,
    else_branch: &'a Expr<'src>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
    // Infer condition
    let condition_term = expression_to_term(condition, &scopes)?;

    // Unify condition
    let subs = unify::Substitutions::new();
    let _subs = unify::unify(&condition_term, &Term::Constant(Value::Bool), &subs)
        .map_err(Error::UnifyError)?;

    // Infer then_branch
    let then_branch_term = expression_to_term(then_branch, &scopes)?;

    // Infer else_branch
    let else_branch_term = expression_to_term(else_branch, &scopes)?;

    // TODO: Unify instead of equality check
    if then_branch_term == else_branch_term {
        Ok(then_branch_term)
    } else {
        Err(Error::Broken("else & then don't match", line!()))
    }
}
