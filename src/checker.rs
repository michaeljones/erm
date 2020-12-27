pub mod term;

use im::HashMap;
use std::rc::Rc;

use self::term::{Term, Value};
use super::env;
use super::function::{Func, Function};
use super::parser::{Expr, Module, Stmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownBinding(String),
    UnhandledExpression(String),
    FailedToUnify,
    UnknownFunction(String),
    UnknownOperator(String),
    TooManyArguments,
}

pub fn check<'src>(module: &Module<'src>, scopes: &env::Scopes<'src>) -> Result<(), Error> {
    let scope = env::Scope::from_module(&module);
    let scopes = env::add_scope(&scopes, scope);

    // let mut var_generator = VarGenerator::new();
    match env::get_binding(&scopes, "main") {
        Some(expr) => {
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
            let scope = env::Scope::from_module(&module);
            let scopes = vector![Rc::new(scope)];
            let main_term = expression_to_term(&expr, &scopes)?;
            let target_term = Term::Constant(Value::String);
            let subs = HashMap::new();

            unify(&main_term, &target_term, &subs).map(|_| ())
        }
        _ => Err(Error::UnknownBinding("main".to_string())),
    }
}

fn expression_to_term<'a, 'b, 'src>(
    expr: &'a Expr<'src>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
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
        _ => Err(Error::UnhandledExpression(format!("{:?}", expr))),
    }
}

fn binary_expression_to_term<'a, 'b, 'src>(
    operator_name: &'src str,
    left: &Rc<Expr<'src>>,
    right: &Rc<Expr<'src>>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
    if let Some(operator) = env::get_operator(&scopes, operator_name) {
        match env::get_function(&scopes, operator.function_name) {
            Some(Function::UserDefined(stmt_rc)) => match &*stmt_rc {
                Stmt::Function { expr, .. } => expression_to_term(&expr, &scopes),
                _ => Err(Error::UnknownOperator(operator_name.to_string())),
            },
            Some(Function::BuiltIn(func)) => {
                let args = vec![left.clone(), right.clone()];
                built_in_to_term(func, &args, &scopes)
            }
            _ => Err(Error::UnknownFunction(operator.function_name.to_string())),
        }
    } else {
        Err(Error::UnknownOperator(operator_name.to_string()))
    }
}

fn call_to_term<'a, 'b, 'src>(
    function_name: &'src str,
    args: &'a Vec<Rc<Expr<'src>>>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
    match env::get_function(&scopes, function_name) {
        Some(Function::UserDefined(stmt_rc)) => match *stmt_rc {
            Stmt::Function { .. } => Err(Error::UnknownFunction(function_name.to_string())),
            _ => Err(Error::UnknownFunction(function_name.to_string())),
        },

        Some(Function::BuiltIn(func)) => built_in_to_term(func, args, &scopes),
        _ => Err(Error::UnknownFunction(function_name.to_string())),
    }
}

fn built_in_to_term<'a, 'b, 'src>(
    func: Rc<dyn Func>,
    args: &'b Vec<Rc<Expr<'src>>>,
    scopes: &'b env::Scopes<'src>,
) -> Result<Term, Error> {
    let signature = func.term();

    if args.len() >= signature.len() {
        return Err(Error::TooManyArguments);
    }

    let arg_terms = args
        .iter()
        .map(|arg| expression_to_term(&arg, &scopes))
        .collect::<Result<Vec<Term>, Error>>()?;

    let remaining: Vec<&Term> = signature
        .iter()
        .map(Some)
        .zip(arg_terms.iter().map(Some).chain(std::iter::repeat(None)))
        .skip_while(|(sig_term, arg_term)| sig_term == arg_term)
        .flat_map(|(sig_term, _)| sig_term)
        .collect();

    assert!(remaining.len() >= 1);

    if remaining.len() == 1 {
        Ok(remaining[0].clone())
    } else {
        Ok(Term::Function {
            name: "anon".to_string(),
            signature: remaining.iter().cloned().cloned().collect(),
        })
    }
}

type Substitutions<'src> = HashMap<String, &'src Term>;

fn unify<'a, 'src>(
    x: &'src Term,
    y: &'src Term,
    subs: &'a Substitutions<'src>,
) -> Result<Substitutions<'src>, Error> {
    if x == y {
        Ok(subs.clone())
    } else if let Term::Var(name) = x {
        unify_variable(name, x, y, subs)
    } else if let Term::Var(name) = y {
        unify_variable(name, y, x, subs)
    } else {
        Err(Error::FailedToUnify)
    }
}

fn unify_variable<'a, 'src>(
    v_name: &'src String,
    v: &'src Term,
    x: &'src Term,
    subs: &'a Substitutions<'src>,
) -> Result<Substitutions<'src>, Error> {
    if let Some(term) = subs.get(v_name) {
        return unify(term, x, subs);
    }

    if let Term::Var(x_name) = x {
        if let Some(term) = subs.get(x_name) {
            return unify(v, term, subs);
        }
    }

    Ok(subs.update(v_name.to_string(), x))
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_unification<'a, 'src>(
        x: &'src Term,
        y: &'src Term,
        mut subs: &'a Substitutions<'src>,
    ) -> Result<Substitutions<'src>, Error> {
        unify(&x, &y, &mut subs)
    }

    #[test]
    fn constant_and_var() {
        let var = Term::Var("a".to_string());
        let mut subs = HashMap::new();
        let result = test_unification(&Term::Constant(Value::String), &var, &mut subs);

        let mut expected_subs = HashMap::new();
        expected_subs.insert("a".to_string(), &Term::Constant(Value::String));
        assert_eq!(result, Ok(expected_subs));
    }

    #[test]
    fn conflicting_constants() {
        let constant = Term::Constant(Value::Integer);
        let mut subs = HashMap::new();
        let result = test_unification(&Term::Constant(Value::String), &constant, &mut subs);

        assert_eq!(result, Err(Error::FailedToUnify));
    }
}
