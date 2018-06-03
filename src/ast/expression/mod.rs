mod character;
mod string;
mod float;
mod integer;
mod access;
mod variable;
mod core;

pub use ast::expression::core::Expression;

use ast::expression::character::character;
use ast::expression::string::string;
use ast::expression::float::float;
use ast::expression::integer::integer;
use ast::expression::variable::variable;
use ast::expression::access::access;
use ast::helpers::{lo_name, new_line_and_exact_indent, operator, spaces_or_new_line_and_indent};

// use nom;
use nom::{multispace, multispace0, space1};
use nom::types::CompleteStr;

named_args!(tuple(indentation: u32) <CompleteStr, Expression>,
  map!(
    delimited!(
        char!('('),
        separated_list!(
            char!(','),
            delimited!(
                multispace0,
                call!(expression, indentation),
                multispace0
            )
        ),
        char!(')')
    ),
    Expression::Tuple
  )
);

named_args!(list(indentation: u32) <CompleteStr, Expression>,
  map!(
    delimited!(
        char!('['),
        // Not sure why this optional is required
        opt!(
            separated_list!(
                char!(','),
                delimited!(
                    multispace0,
                    call!(expression, indentation),
                    multispace0
                )
            )
        ),
        char!(']')
    ),
    |o| Expression::List(o.unwrap_or(vec![]))
  )
);

named_args!(record(indentation: u32) <CompleteStr, Expression>,
  map!(
    delimited!(
        char!('{'),
        separated_list!(
            char!(','),
            do_parse!(
                multispace0 >>
                name: lo_name >>
                multispace0 >>
                char!('=') >>
                multispace0 >>
                expression: call!(expression, indentation) >>
                multispace0 >>
                ((name, expression))
            )
        ),
        char!('}')
    ),
    Expression::Record
  )
);

named_args!(simplified_record(indentation: u32) <CompleteStr, Expression>,
  map!(
    delimited!(
        char!('{'),
        separated_list!(
            char!(','),
            do_parse!(
                multispace0 >>
                name: lo_name >>
                multispace0 >>
                ((name.clone(), Expression::Variable(vec![name.to_string()])))
            )
        ),
        char!('}')
    ),
    Expression::Record
  )
);

named_args!(record_update(indentation: u32) <CompleteStr, Expression>,
  delimited!(
    char!('{'),
    do_parse!(
      multispace0 >>
      name: lo_name >>
      multispace0 >>
      char!('|') >>
      multispace0 >>
      pairs: separated_list!(
        char!(','),
        do_parse!(
          multispace0 >>
          name: lo_name >>
          multispace0 >>
          char!('=') >>
          multispace0 >>
          expression: call!(expression, indentation) >>
          multispace0 >>
          ((name, expression))
        )
      ) >>
      (Expression::RecordUpdate(name.to_string(), pairs))
    ),
    char!('}')
  )
);

named_args!(parens(indentation: u32) <CompleteStr, Expression>,
    delimited!(
        char!('('),
        call!(expression, indentation),
        char!(')')
    )
);

named_args!(term(indentation: u32) <CompleteStr, Expression>,
  alt!(
      access
    | variable
    // | accessFunction
    | string
    | float
    | integer
    | character
    | call!(parens, indentation)
    | call!(list, indentation)
    | call!(tuple, indentation)
    | call!(record_update, indentation)
    | call!(record, indentation)
    | call!(simplified_record, indentation)
  )
);

named_args!(lambda(indentation: u32) <CompleteStr, Expression>,
  do_parse!(
    char!('\\') >>
    args: separated_nonempty_list!(space1, call!(term, indentation)) >>
    tag!("->") >>
    body: call!(expression, indentation) >>
    (Expression::Lambda(args, Box::new(body)))
  )
);

named_args!(application_or_var(indentation: u32) <CompleteStr, Expression>,
  map_res!(
      separated_list!(
          call!(spaces_or_new_line_and_indent, indentation),
          call!(term, indentation)
      ),
      |v: Vec<Expression>| {
          if v.len() == 0 {
              Err("Empty list".to_string())
          }
          else if v.len() == 1 {
              Ok(v[0].clone())
          } else {
              let mut app = Expression::Application(Box::new(v[0].clone()), Box::new(v[1].clone()));
              for entry in v.iter().skip(2) {
                  app = Expression::Application(Box::new(app), Box::new(entry.clone()))
              }
              Ok(app)
          }
      }
  )
);

named_args!(let_binding(indentation: u32) <CompleteStr, (Expression, Expression)>,
   do_parse!(
       binding: call!(expression, indentation) >>
       call!(spaces_or_new_line_and_indent, indentation) >>
       char!('=') >>
       new_indent: call!(spaces_or_new_line_and_indent, indentation) >>
       expression: call!(expression, new_indent) >>
       (binding, expression)
   )
);

named_args!(let_bindings(indentation: u32) <CompleteStr, Vec<(Expression, Expression)>>,
    separated_nonempty_list!(call!(new_line_and_exact_indent, indentation), call!(let_binding, indentation))
);

named_args!(let_expression(indentation: u32) <CompleteStr, Expression>,
   do_parse!(
       tag!("let") >>
       assignment_indentation: call!(spaces_or_new_line_and_indent, indentation) >>
       assignments: call!(let_bindings, assignment_indentation) >>
       call!(spaces_or_new_line_and_indent, indentation) >>
       tag!("in") >>
       call!(spaces_or_new_line_and_indent, indentation) >>
       expression: call!(expression, indentation) >>
       (Expression::Let(assignments, Box::new(expression)))
   )
);

named_args!(case(indentation: u32) <CompleteStr, (Expression, Expression)>,
   do_parse!(
       matcher: call!(expression, indentation) >>
       call!(spaces_or_new_line_and_indent, indentation) >>
       tag!("->") >>
       new_indent: call!(spaces_or_new_line_and_indent, indentation) >>
       expression: call!(expression, new_indent) >>
       (matcher, expression)
   )
);

named_args!(cases(indentation: u32) <CompleteStr, Vec<(Expression, Expression)>>,
    separated_nonempty_list!(call!(new_line_and_exact_indent, indentation), call!(case, indentation))
);

named_args!(case_expression(indentation: u32) <CompleteStr, Expression>,
   do_parse!(
       tag!("case") >>
       call!(spaces_or_new_line_and_indent, indentation) >>
       expression: call!(expression, indentation) >>
       call!(spaces_or_new_line_and_indent, indentation) >>
       tag!("of") >>
       new_indent: call!(spaces_or_new_line_and_indent, indentation) >>
       cases: call!(cases, new_indent) >>
       (Expression::Case(Box::new(expression), cases))
   )
);

// Application followed by possibly a operator + expression
named_args!(binary(indentation: u32) <CompleteStr, Expression>,
   do_parse!(
     application_or_var: call!(application_or_var, indentation) >>
     operator_exp: opt!(
       do_parse!(
         call!(spaces_or_new_line_and_indent, indentation) >>
         operator: operator >>
         multispace >>
         expression: call!(expression, indentation) >>
         (operator, expression)
       )
     ) >>
     (match operator_exp {
         Some((op, exp)) => Expression::BinOp(
             Box::new(Expression::Variable(vec![op.to_string()])),
             Box::new(application_or_var),
             Box::new(exp)
         ),
         None => application_or_var
     })
  )
);

named_args!(pub expression(indentation: u32) <CompleteStr, Expression>,
  alt!(
         call!(binary, indentation)
       | call!(let_expression, indentation)
       | call!(case_expression, indentation)
    // | if_expression
       | call!(lambda, indentation)
  )
);

#[cfg(test)]
mod tests {

    use ast::expression::*;
    use nom::types::CompleteStr;

    fn var(name: &str) -> Expression {
        Expression::Variable(vec![name.to_string()])
    }

    fn int(text: &str) -> Expression {
        Expression::Integer(text.to_string())
    }

    fn application(a: Expression, b: Expression) -> Expression {
        Expression::Application(Box::new(a), Box::new(b))
    }

    // Tuples

    #[test]
    fn simple_tuple() {
        assert_eq!(
            tuple(CompleteStr("(a,b)"), 0),
            Ok((CompleteStr(""), Expression::Tuple(vec![var("a"), var("b")])))
        );
    }

    #[test]
    fn simple_tuple_with_formatting() {
        assert_eq!(
            tuple(CompleteStr("( a, b )"), 0),
            Ok((CompleteStr(""), Expression::Tuple(vec![var("a"), var("b")])))
        );
    }

    // Lists

    #[test]
    fn empty_list() {
        assert_eq!(
            list(CompleteStr("[]"), 0),
            Ok((CompleteStr(""), Expression::List(vec![])))
        );
    }

    #[test]
    fn simple_list() {
        assert_eq!(
            list(CompleteStr("[a,b]"), 0),
            Ok((CompleteStr(""), Expression::List(vec![var("a"), var("b")])))
        );
    }

    #[test]
    fn simple_list_with_formatting() {
        assert_eq!(
            list(CompleteStr("[ a, b ]"), 0),
            Ok((CompleteStr(""), Expression::List(vec![var("a"), var("b")])))
        );
    }

    #[test]
    fn simple_int_list() {
        assert_eq!(
            list(CompleteStr("[1,2]"), 0),
            Ok((CompleteStr(""), Expression::List(vec![int("1"), int("2")])))
        );
    }

    #[test]
    fn tuple_list() {
        assert_eq!(
            list(CompleteStr("[(a, b), (a, b)]"), 0),
            Ok((
                CompleteStr(""),
                Expression::List(vec![
                    Expression::Tuple(vec![var("a"), var("b")]),
                    Expression::Tuple(vec![var("a"), var("b")]),
                ])
            ))
        );
    }

    // Application or Var

    #[test]
    fn simple_variable() {
        assert_eq!(
            application_or_var(CompleteStr("abc"), 0),
            Ok((CompleteStr(""), var("abc")))
        );
    }

    #[test]
    fn simple_application() {
        assert_eq!(
            application_or_var(CompleteStr("f a"), 0),
            Ok((
                CompleteStr(""),
                Expression::Application(Box::new(var("f")), Box::new(var("a")))
            ))
        );
    }

    #[test]
    fn curried_application() {
        assert_eq!(
            application_or_var(CompleteStr("f a b"), 0),
            Ok((
                CompleteStr(""),
                Expression::Application(
                    Box::new(Expression::Application(
                        Box::new(var("f")),
                        Box::new(var("a"))
                    )),
                    Box::new(var("b"))
                )
            ))
        );
    }

    #[test]
    fn curried_application_with_parens() {
        assert_eq!(
            application_or_var(CompleteStr("(f a) b"), 0),
            Ok((
                CompleteStr(""),
                Expression::Application(
                    Box::new(Expression::Application(
                        Box::new(var("f")),
                        Box::new(var("a"))
                    )),
                    Box::new(var("b"))
                )
            ))
        );
    }

    #[test]
    fn multiline_application() {
        assert_eq!(
            application_or_var(CompleteStr("f\n   a\n b"), 0),
            Ok((
                CompleteStr(""),
                Expression::Application(
                    Box::new(Expression::Application(
                        Box::new(var("f")),
                        Box::new(var("a"))
                    )),
                    Box::new(var("b"))
                )
            ))
        );
    }

    #[test]
    fn multiline_bug() {
        assert_eq!(
            application_or_var(CompleteStr("f\n (==)"), 0),
            Ok((
                CompleteStr(""),
                Expression::Application(Box::new(var("f")), Box::new(var("=="))),
            ))
        );
    }

    #[test]
    fn same_multiline_bug() {
        assert_eq!(
            application_or_var(CompleteStr("f\n \"I like the symbol =\""), 0),
            Ok((
                CompleteStr(""),
                Expression::Application(
                    Box::new(var("f")),
                    Box::new(Expression::String("I like the symbol =".to_string()))
                ),
            ))
        );
    }

    #[test]
    fn constructor_application() {
        assert_eq!(
            application_or_var(CompleteStr("Cons a Nil"), 0),
            Ok((
                CompleteStr(""),
                Expression::Application(
                    Box::new(Expression::Application(
                        Box::new(var("Cons")),
                        Box::new(var("a"))
                    )),
                    Box::new(var("Nil"))
                )
            ))
        );
    }

    #[test]
    fn application_with_record_update() {
        assert_eq!(
            application_or_var(CompleteStr("a  { r | f = 1 } c"), 0),
            Ok((
                CompleteStr(""),
                application(
                    application(
                        var("a"),
                        Expression::RecordUpdate(
                            "r".to_string(),
                            vec![("f".to_string(), int("1"))]
                        )
                    ),
                    var("c")
                )
            ))
        );
    }

    // Record

    #[test]
    fn simple_record() {
        assert_eq!(
            record(CompleteStr("{ a = b }"), 0),
            Ok((
                CompleteStr(""),
                Expression::Record(vec![("a".to_string(), var("b"))])
            ))
        );
    }

    #[test]
    fn simple_record_with_many_fields() {
        assert_eq!(
            record(CompleteStr("{ a = b, b = 2 }"), 0),
            Ok((
                CompleteStr(""),
                Expression::Record(vec![
                    ("a".to_string(), var("b")),
                    ("b".to_string(), int("2")),
                ])
            ))
        );
    }

    #[test]
    fn simple_record_with_many_tuple_fields() {
        assert_eq!(
            record(CompleteStr("{ a = (a, b), b = (a, b) }"), 0),
            Ok((
                CompleteStr(""),
                Expression::Record(vec![
                    ("a".to_string(), Expression::Tuple(vec![var("a"), var("b")])),
                    ("b".to_string(), Expression::Tuple(vec![var("a"), var("b")])),
                ])
            ))
        );
    }

    #[test]
    fn simple_record_with_updated_field() {
        assert_eq!(
            record_update(CompleteStr("{ a | b = 2, c = 3 }"), 0),
            Ok((
                CompleteStr(""),
                Expression::RecordUpdate(
                    "a".to_string(),
                    vec![("b".to_string(), int("2")), ("c".to_string(), int("3"))]
                )
            ))
        );
    }

    #[test]
    fn simple_record_with_advanced_field() {
        assert_eq!(
            record(CompleteStr("{ a = Just 2 }"), 0),
            Ok((
                CompleteStr(""),
                Expression::Record(vec![("a".to_string(), application(var("Just"), int("2")))])
            ))
        );
    }

    #[test]
    fn simple_record_update_with_advanced_field() {
        assert_eq!(
            record_update(CompleteStr("{ a | a = Just 2 }"), 0),
            Ok((
                CompleteStr(""),
                Expression::RecordUpdate(
                    "a".to_string(),
                    vec![("a".to_string(), application(var("Just"), int("2")))],
                )
            ))
        );
    }

    #[test]
    fn simple_record_destructuring_pattern() {
        assert_eq!(
            simplified_record(CompleteStr("{ a, b }"), 0),
            Ok((
                CompleteStr(""),
                Expression::Record(vec![
                    ("a".to_string(), var("a")),
                    ("b".to_string(), var("b")),
                ],)
            ))
        );
    }

    // Binary Ops

    #[test]
    fn simple_binary_op() {
        assert_eq!(
            binary(CompleteStr("x + 1"), 0),
            Ok((
                CompleteStr(""),
                Expression::BinOp(Box::new(var("+")), Box::new(var("x")), Box::new(int("1")),)
            ))
        );
    }

    // Let Expressions

    #[test]
    fn let_single_binding() {
        assert_eq!(
            let_binding(CompleteStr("a = 42"), 0),
            Ok((CompleteStr(""), (var("a"), int("42"))))
        );
    }

    #[test]
    fn let_group_single_binding() {
        assert_eq!(
            let_bindings(CompleteStr("a = 42"), 0),
            Ok((CompleteStr(""), vec![(var("a"), int("42"))]))
        );
    }

    #[test]
    fn let_block_with_single_binding() {
        assert_eq!(
            let_expression(CompleteStr("let a = 42 in a"), 0),
            Ok((
                CompleteStr(""),
                Expression::Let(vec![(var("a"), int("42"))], Box::new(var("a")))
            ))
        );
    }

    #[test]
    fn let_block_bind_to_underscore() {
        assert_eq!(
            let_expression(CompleteStr("let _ = 42 in 24"), 0),
            Ok((
                CompleteStr(""),
                Expression::Let(vec![(var("_"), int("42"))], Box::new(int("24")))
            ))
        );
    }

    #[test]
    fn let_block_can_start_with_a_tag_name() {
        assert_eq!(
            let_expression(CompleteStr("let letter = 1 \n in letter"), 0),
            Ok((
                CompleteStr(""),
                Expression::Let(vec![(var("letter"), int("1"))], Box::new(var("letter")))
            ))
        );
    }

    #[test]
    fn let_block_function_1() {
        assert_eq!(
            let_expression(
                CompleteStr(
                    "let
 f x = x + 1
in
 f 4"
                ),
                0
            ),
            Ok((
                CompleteStr(""),
                Expression::Let(
                    vec![
                        (
                            application(var("f"), var("x")),
                            Expression::BinOp(
                                Box::new(var("+")),
                                Box::new(var("x")),
                                Box::new(int("1")),
                            ),
                        ),
                    ],
                    Box::new(application(var("f"), int("4")))
                )
            ))
        );
    }

    #[test]
    fn let_block_function_2() {
        assert_eq!(
            let_expression(
                CompleteStr(
                    "let
  f x = x + 1
  g x = x + 1
in
  f 4"
                ),
                0
            ),
            Ok((
                CompleteStr(""),
                Expression::Let(
                    vec![
                        (
                            application(var("f"), var("x")),
                            Expression::BinOp(
                                Box::new(var("+")),
                                Box::new(var("x")),
                                Box::new(int("1")),
                            ),
                        ),
                        (
                            application(var("g"), var("x")),
                            Expression::BinOp(
                                Box::new(var("+")),
                                Box::new(var("x")),
                                Box::new(int("1")),
                            ),
                        ),
                    ],
                    Box::new(application(var("f"), int("4")))
                )
            ))
        );
    }

    #[test]
    fn let_block_multiple_bindings() {
        assert_eq!(
            let_expression(
                CompleteStr(
                    "let
  a = 42
  b = a + 1
in
  b"
                ),
                0
            ),
            Ok((
                CompleteStr(""),
                Expression::Let(
                    vec![
                        (var("a"), int("42")),
                        (
                            var("b"),
                            Expression::BinOp(
                                Box::new(var("+")),
                                Box::new(var("a")),
                                Box::new(int("1")),
                            ),
                        ),
                    ],
                    Box::new(var("b"))
                )
            ))
        );
    }

    // Case Expressions

    #[test]
    fn case_simple_statement() {
        assert_eq!(
            case_expression(
                CompleteStr(
                    "case x of
  Nothing ->
    0
  Just y ->
    y"
                ),
                0
            ),
            Ok((
                CompleteStr(""),
                Expression::Case(
                    Box::new(var("x")),
                    vec![
                        (var("Nothing"), int("0")),
                        (application(var("Just"), var("y")), var("y")),
                    ],
                )
            ))
        );
    }

    #[test]
    fn case_binding_to_underscore() {
        assert_eq!(
            case_expression(
                CompleteStr(
                    "case x of
  _ ->
    42"
                ),
                0
            ),
            Ok((
                CompleteStr(""),
                Expression::Case(Box::new(var("x")), vec![(var("_"), int("42"))])
            ))
        );
    }

    #[test]
    fn case_nested() {
        assert_eq!(
            case_expression(
                CompleteStr(
                    "case x of
  a -> a
  b ->
    case y of
      a1 -> a1
      b1 -> b1
  c -> c"
                ),
                0
            ),
            Ok((
                CompleteStr(""),
                Expression::Case(
                    Box::new(var("x")),
                    vec![
                        (var("a"), var("a")),
                        (
                            var("b"),
                            Expression::Case(
                                Box::new(var("y")),
                                vec![(var("a1"), var("a1")), (var("b1"), var("b1"))],
                            ),
                        ),
                        (var("c"), var("c")),
                    ]
                )
            ))
        );
    }
}
