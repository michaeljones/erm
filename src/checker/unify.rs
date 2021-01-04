use im::HashMap;

use super::term::Term;

pub type Substitutions<'src> = HashMap<String, &'src Term>;

#[derive(Debug, PartialEq)]
pub enum Error {
    FailedToUnify(String, String),
}

pub fn unify<'a, 'src>(
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
        match (x, y) {
            (Term::Function(x_1, x_rest), Term::Function(y_1, y_rest)) => {
                let subs = unify(x_1, y_1, &subs)?;
                unify(x_rest, y_rest, &subs)
            }
            _ => Err(Error::FailedToUnify(format!("{:?}", x), format!("{:?}", y))),
        }
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
    use super::super::term::Value;
    use super::*;

    fn test_unification<'a, 'src>(
        x: &'src Term,
        y: &'src Term,
        mut subs: &'a Substitutions<'src>,
    ) -> Result<Substitutions<'src>, Error> {
        unify(&x, &y, &mut subs)
    }

    #[test]
    fn conflicting_constants() {
        let mut subs = Substitutions::new();
        let result = test_unification(
            &Term::Constant(Value::String),
            &Term::Constant(Value::Integer),
            &mut subs,
        );

        assert_eq!(
            result,
            Err(Error::FailedToUnify(
                "Constant(String)".to_string(),
                "Constant(Integer)".to_string()
            ))
        );
    }

    #[test]
    fn constant_and_var() {
        let var = Term::Var("a".to_string());
        let mut subs = Substitutions::new();
        let result = test_unification(&Term::Constant(Value::String), &var, &mut subs);

        let mut expected_subs = Substitutions::new();
        expected_subs.insert("a".to_string(), &Term::Constant(Value::String));
        assert_eq!(result, Ok(expected_subs));
    }

    #[test]
    fn var_and_constant() {
        let var = Term::Var("a".to_string());
        let mut subs = Substitutions::new();
        let result = test_unification(&var, &Term::Constant(Value::String), &mut subs);

        let mut expected_subs = Substitutions::new();
        expected_subs.insert("a".to_string(), &Term::Constant(Value::String));
        assert_eq!(result, Ok(expected_subs));
    }

    #[test]
    fn var_and_var() {
        let var_a = Term::Var("a".to_string());
        let var_b = Term::Var("b".to_string());
        let mut subs = Substitutions::new();
        let result = test_unification(&var_a, &var_b, &mut subs);

        let mut expected_subs = Substitutions::new();
        expected_subs.insert("a".to_string(), &var_b);
        assert_eq!(result, Ok(expected_subs));
    }

    #[test]
    fn var_and_function() {
        let var_a = Term::Var("a".to_string());
        let var_b = Term::Function(
            Box::new(Term::Constant(Value::String)),
            Box::new(Term::Constant(Value::String)),
        );
        let mut subs = Substitutions::new();
        let result = test_unification(&var_a, &var_b, &mut subs);

        let mut expected_subs = Substitutions::new();
        expected_subs.insert("a".to_string(), &var_b);
        assert_eq!(result, Ok(expected_subs));
    }

    #[test]
    fn matching_function_and_function() {
        let var_a = Term::Function(
            Box::new(Term::Constant(Value::String)),
            Box::new(Term::Constant(Value::Integer)),
        );

        let var_b = Term::Function(
            Box::new(Term::Constant(Value::String)),
            Box::new(Term::Constant(Value::Integer)),
        );

        let mut subs = Substitutions::new();
        let result = test_unification(&var_a, &var_b, &mut subs);

        let expected_subs = Substitutions::new();
        assert_eq!(result, Ok(expected_subs));
    }

    #[test]
    fn var_function_and_function() {
        let var_a = Term::Function(
            Box::new(Term::Var("a".to_string())),
            Box::new(Term::Constant(Value::String)),
        );

        let var_b = Term::Function(
            Box::new(Term::Constant(Value::String)),
            Box::new(Term::Constant(Value::String)),
        );

        let mut subs = Substitutions::new();
        let result = test_unification(&var_a, &var_b, &mut subs);

        let mut expected_subs = Substitutions::new();
        expected_subs.insert("a".to_string(), &Term::Constant(Value::String));
        assert_eq!(result, Ok(expected_subs));
    }
}
