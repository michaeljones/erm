mod error;
mod extract;
mod indent;
mod mtch;
mod types;
mod whitespace;

use std::convert::TryFrom;
use std::rc::Rc;

use super::ast::*;
use super::lexer::{SrcToken, Token, TokenIter};

pub use self::error::Error;
use self::mtch::{matches, matches_space};

pub type ParseResult = Result<Module, Error>;

pub fn parse(mut iter: &mut TokenIter) -> ParseResult {
    log::trace!("parse");

    // Uncomment to print out whole token stream
    // println!("{:?}", iter.collect::<Vec<_>>());

    matches(&iter.next(), Token::Module)?;
    matches_space(&iter.next())?;
    let name = extract::extract_module_name(&iter.next())?;
    log::trace!("module {:?}", name);
    matches_space(&iter.next())?;
    let exposing = parse_exposing(&mut iter)?;

    // Uncomment to print out whole token stream
    // println!("{:?}", iter.collect::<Vec<_>>());
    //
    whitespace::consume_til_line_start(&mut iter);

    let imports = parse_imports(&mut iter)?;

    whitespace::consume_til_line_start(&mut iter);

    let statements = parse_statements(&mut iter)?;

    if iter.peek() == None {
        Ok(Module {
            name,
            exposing,
            imports,
            statements,
        })
    } else {
        let tokens = iter.map(|token| format!("{:?}", token)).collect();
        Err(Error::TokensRemaining(tokens))
    }
}

fn parse_exposing(mut iter: &mut TokenIter) -> Result<Exposing, Error> {
    matches(&iter.next(), Token::Exposing)?;
    matches_space(&iter.next())?;
    matches(&iter.next(), Token::OpenParen)?;

    let exposing = match iter.peek() {
        Some((Token::Ellipsis, _range)) => {
            iter.next();
            Ok(Exposing::All)
        }
        Some(_) => parse_exposing_details(&mut iter).map(Exposing::List),
        token => Err(Error::UnknownExposing(format!("{:?}", token))),
    }?;

    matches(&iter.next(), Token::CloseParen)?;

    Ok(exposing)
}

fn parse_exposing_details(iter: &mut TokenIter) -> Result<Vec<ExposingDetail>, Error> {
    let mut details = vec![];
    loop {
        match iter.peek() {
            Some((Token::OpenParen, _range)) => {
                iter.next();
                let operator_name = extract::extract_operator(&iter.next())?;
                matches(&iter.next(), Token::CloseParen)?;
                details.push(ExposingDetail::Operator(operator_name.to_string()))
            }
            Some((Token::LowerName(name), _range)) => {
                details.push(ExposingDetail::Name(name.to_string()));
                iter.next();
            }
            token => return Err(Error::UnknownExposing(format!("{:?}", token))),
        }

        if let Some((Token::CloseParen, _range)) = iter.peek() {
            // Break without consuming the CloseParen
            break;
        }

        matches(&iter.next(), Token::Comma)?;
        matches_space(&iter.next())?;
    }

    Ok(details)
}

// Imports
fn parse_imports(iter: &mut TokenIter) -> Result<Vec<Import>, Error> {
    let mut imports = vec![];

    loop {
        if !matches!(iter.peek(), Some((Token::Import, _range))) {
            break;
        }

        matches(&iter.next(), Token::Import)?;
        matches_space(&iter.next())?;
        let module_name = extract::extract_module_name(&iter.next())?;

        let mut exposing = None;

        whitespace::consume_spaces(iter);
        if matches!(iter.peek(), Some((Token::Exposing, _range))) {
            exposing = Some(parse_exposing(iter)?);
        }

        imports.push(Import {
            module_name,
            exposing,
        });

        whitespace::consume_til_line_start(iter);
    }

    Ok(imports)
}

// Statements
fn parse_statements(iter: &mut TokenIter) -> Result<Vec<Rc<Stmt>>, Error> {
    log::trace!("parse_statements: {:?}", iter.peek());
    let mut statements = vec![];

    let base = 0;
    let mut current = 0;

    loop {
        match iter.peek() {
            Some((Token::LowerName(_), _range)) => {
                // Get the name
                let name = extract::extract_lower_name(&iter.next())?;
                whitespace::consume_spaces(iter);

                let statement = if matches!(iter.peek(), Some((Token::Colon, _range))) {
                    let type_annotation = parse_type_annotation(iter, base, current, name.clone())?;
                    current = indent::must_consume_to_matching(iter, base, current)?;

                    let function_name = extract::extract_lower_name(&iter.next())?;
                    whitespace::consume_spaces(iter);

                    if function_name != name {
                        return Err(Error::NameMismatch);
                    }

                    parse_function_or_binding(
                        iter,
                        base,
                        current,
                        function_name,
                        Some(type_annotation),
                    )?
                } else {
                    parse_function_or_binding(iter, base, current, name, None)?
                };

                statements.push(Rc::new(statement));
            }
            Some((Token::Type, _range)) => {
                let statement = types::parse_type_declaration(iter, base, current)?;
                statements.push(Rc::new(statement));
            }
            Some((Token::Infix, _range)) => {
                let statement = parse_infix(iter, base, current)?;
                statements.push(Rc::new(statement));
            }
            Some((token, range)) => {
                log::error!("UnexpectedToken");
                return Err(Error::UnexpectedToken {
                    found: token.to_string(),
                    expected: "Expression token".to_string(),
                    range: range.clone(),
                });
            }
            None => break,
        }

        // TODO: Update/fix/change
        indent::must_consume_to_matching(iter, base, current)?;
    }

    Ok(statements)
}

// Infix operators
fn parse_infix(iter: &mut TokenIter, _base: usize, mut _current: usize) -> Result<Stmt, Error> {
    log::trace!("parse_infix: {:?}", iter.peek());
    matches(&iter.next(), Token::Infix)?;
    whitespace::consume_spaces(iter);

    let associativity = extract::extract_associativity(&iter.next())?;
    whitespace::consume_spaces(iter);

    let precedence = extract_precendence(&iter.next())?;
    whitespace::consume_spaces(iter);

    matches(&iter.next(), Token::OpenParen)?;
    let operator_name = extract::extract_operator(&iter.next())?;
    matches(&iter.next(), Token::CloseParen)?;
    whitespace::consume_spaces(iter);

    matches(&iter.next(), Token::Equals)?;
    whitespace::consume_spaces(iter);

    let function_name = extract::extract_qualified_lower_name(&iter.next())?;

    Ok(Stmt::Infix {
        operator_name: operator_name.to_string(),
        associativity,
        precedence,
        function_name,
    })
}

fn extract_precendence(stream_token: &Option<SrcToken>) -> Result<usize, Error> {
    match stream_token {
        Some((Token::LiteralInteger(int), _range)) => {
            usize::try_from(*int).map_err(|_| Error::NegativePrecendence)
        }
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: Token::UpperName("").to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}

// Type annotations
fn parse_type_annotation(
    iter: &mut TokenIter,
    base: usize,
    current: usize,
    name: LowerName,
) -> Result<TypeAnnotation, Error> {
    log::trace!("parse_type_annotation: {:?}", name);
    matches(&iter.next(), Token::Colon)?;
    whitespace::consume_spaces(iter);

    let type_ = types::parse_type(iter, base, current)?;
    whitespace::consume_spaces(iter);

    Ok(TypeAnnotation {
        // TODO: Don't use lower name for this stuff
        name,
        type_,
    })
}

// Functions & bindings
fn parse_function_or_binding(
    iter: &mut TokenIter,
    base: usize,
    mut current: usize,
    name: LowerName,
    type_annotation: Option<TypeAnnotation>,
) -> Result<Stmt, Error> {
    log::trace!("parse_function_or_binding: {:?}", name);
    let mut args = Vec::new();
    loop {
        if !matches!(iter.peek(), Some((Token::LowerName(_), _range))) {
            break;
        }

        let arg = extract::extract_pattern_name(&iter.next())?;
        args.push(arg);

        whitespace::consume_spaces(iter);
    }

    matches(&iter.next(), Token::Equals)?;

    let indent = indent::consume_to_indented(iter, base, current)?;
    current = indent.extract();

    let (expr, _curr) = parse_expression(iter, current, current)?;

    if args.is_empty() {
        Ok(Stmt::Binding {
            type_annotation,
            name,
            expr: Rc::new(expr),
        })
    } else {
        Ok(Stmt::Function {
            type_annotation,
            name,
            args,
            expr: Rc::new(expr),
        })
    }
}

// Expressions
//
fn parse_expression(
    mut iter: &mut TokenIter,
    base: usize,
    current: usize,
) -> Result<(Expr, usize), Error> {
    log::trace!("parse_expression: {:?}", iter.peek());
    match iter.peek() {
        Some((Token::If, _range)) => parse_if_expression(&mut iter, base, current),
        None => Err(Error::UnexpectedEnd),
        _ => parse_binary_expression(&mut iter, base, current),
    }
}

// Binary Expressions
//
// Shunting yard approach based on:
//   - https://eli.thegreenplace.net/2009/03/20/a-recursive-descent-parser-with-an-infix-expression-evaluator
//   - http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm
//
fn parse_binary_expression(
    mut iter: &mut TokenIter,
    base: usize,
    current: usize,
) -> Result<(Expr, usize), Error> {
    log::trace!("parse_binary_expression: {:?}", iter.peek());
    let (expr, mut current) = parse_var_or_call(&mut iter, base, current)?;

    // We have to keep parsing to look for more parts to this expression but if we find a change in
    // indentation that indicates the end of the scope for this expression then we just want to
    // return the expression we've found so far and allow the level up to deal with the change in
    // scope.
    let indent = indent::consume_to_indented(&mut iter, base, current)?;
    if indent.in_scope() {
        current = indent.extract();
    } else {
        return Ok((expr, indent.extract()));
    }

    let mut operator_stack = Vec::new();
    let mut operand_stack = vec![expr];

    while matches!(iter.peek(), Some((Token::Operator(_), _range))) {
        let operator = extract::extract_operator(&iter.next())?;
        current = indent::must_consume_to_indented(&mut iter, base, current)?;

        process_stacks(operator, &mut operator_stack, &mut operand_stack)?;

        let (right_hand_expr, curr) = parse_var_or_call(&mut iter, base, current)?;
        operand_stack.push(right_hand_expr);
        current = curr;

        // Similar to above, we consume the expression on the right hand side of the operator and
        // then any whitespace afterwards (to reach the next operator if there is one) but if we
        // find that we're no longer in the indentation scope of the expression then we assume
        // we've reached the end of it and continue with processing what we've got so far
        let indent = indent::consume_to_indented(&mut iter, base, current)?;
        if indent.in_scope() {
            current = indent.extract();
        } else {
            break;
        }
    }

    while !operator_stack.is_empty() {
        let operator = operator_stack.pop().ok_or(Error::NoOperator)?;
        let right_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;
        let left_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;

        operand_stack.push(Expr::BinOp {
            operator,
            left: Rc::new(left_hand_expr),
            right: Rc::new(right_hand_expr),
        })
    }

    assert!(operand_stack.len() == 1);
    operand_stack
        .pop()
        .map(|expr| (expr, current))
        .ok_or(Error::NoOperand)
}

fn process_stacks(
    operator: &str,
    mut operator_stack: &mut Vec<String>,
    mut operand_stack: &mut Vec<Expr>,
) -> Result<(), Error> {
    if has_greater_precedence(operator, operator_stack)? {
        operator_stack.push(operator.to_string());
    } else {
        let right_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;
        let left_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;
        let stored_operator = operator_stack.pop().ok_or(Error::NoOperator)?;

        operand_stack.push(Expr::BinOp {
            operator: stored_operator,
            left: Rc::new(left_hand_expr),
            right: Rc::new(right_hand_expr),
        });

        process_stacks(operator, &mut operator_stack, &mut operand_stack)?;
    };

    Ok(())
}

fn has_greater_precedence(operator_a: &str, operator_stack: &[String]) -> Result<bool, Error> {
    if operator_stack.is_empty() {
        Ok(true)
    } else {
        let precedence_a = precedence(operator_a)?;
        let precedence_b = operator_stack
            .last()
            .ok_or(Error::EmptyOperatorStack)
            .and_then(|op| precedence(op))?;

        Ok(precedence_a > precedence_b)
    }
}

// Based on:
//
//   - http://faq.elm-community.org/operators.html
//   - https://github.com/elm-lang/core/blob/master/src/Basics.elm#L72-L90
//
fn precedence(operator: &str) -> Result<usize, Error> {
    match operator {
        "*" | "/" => Ok(7),
        "+" | "-" => Ok(6),
        "++" | "::" => Ok(5),
        "==" | "/=" | ">" | "<" | "<=" | ">=" => Ok(4),
        _ => Err(Error::UnknownOperator(operator.to_string())),
    }
}

/* Parse a single variable or expression that might appear as an argument in a call site. ie.
 * nothing with args unless it is wrapped in parens or anything containing syntax.
 */
fn parse_singular_expression(
    iter: &mut TokenIter,
    base: usize,
    current: usize,
) -> Result<(Expr, usize), Error> {
    log::trace!("parse_singular_expression: {:?}", iter.peek());
    match iter.peek() {
        Some((Token::OpenParen, _range)) => {
            matches(&iter.next(), Token::OpenParen)?;
            let (expr, current) = parse_expression(iter, base, current)?;
            let current = indent::must_consume_to_at_least(iter, base, current)?;
            matches(&iter.next(), Token::CloseParen)?;
            Ok((expr, current))
        }
        // Some((Token::LowerName(_), _range)) => parse_var_or_call(iter, base, current),
        None => Err(Error::UnexpectedEnd),
        _ => parse_contained_expression(iter, base, current),
    }
}

fn parse_contained_expression(
    iter: &mut TokenIter,
    base: usize,
    current: usize,
) -> Result<(Expr, usize), Error> {
    log::trace!("parse_contained_expression: {:?}", iter.peek());
    match iter.peek() {
        Some((Token::LiteralInteger(int), _range)) => {
            let result = Ok((Expr::Integer(*int), current));
            iter.next();
            result
        }
        Some((Token::LiteralFloat(float), _range)) => {
            let result = Ok((Expr::Float(*float), current));
            iter.next();
            result
        }
        Some((Token::LiteralString(string), _range)) => {
            let result = Ok((Expr::String(string.to_string()), current));
            iter.next();
            result
        }
        Some((Token::UpperName("True"), _range)) => {
            let result = Ok((Expr::Bool(true), current));
            iter.next();
            result
        }
        Some((Token::UpperName("False"), _range)) => {
            let result = Ok((Expr::Bool(false), current));
            iter.next();
            result
        }
        Some((Token::LowerName(name), _range)) => {
            let result = Ok((
                Expr::VarName(QualifiedLowerName::from(name.to_string())),
                current,
            ));
            iter.next();
            result
        }
        Some((Token::LowerPath(name), _range)) => {
            let result = Ok((
                Expr::VarName(QualifiedLowerName::from(name.to_string())),
                current,
            ));
            iter.next();
            result
        }
        Some((Token::OpenBracket, _range)) => parse_list_literal(iter, base, current),
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: "Expression token".to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}

/* Parse the contents between [ and ] */
fn parse_list_literal(
    iter: &mut TokenIter,
    base: usize,
    current: usize,
) -> Result<(Expr, usize), Error> {
    log::trace!("parse_list_literal: {:?}", iter.peek());
    matches(&iter.next(), Token::OpenBracket)?;

    let mut expressions = Vec::new();

    loop {
        whitespace::consume_spaces(iter);
        if let Some((Token::CloseBracket, _range)) = iter.peek() {
            break;
        }

        let (expr, _current) = parse_expression(iter, base, current)?;
        expressions.push(Rc::new(expr));

        whitespace::consume_spaces(iter);

        match iter.peek() {
            Some((Token::CloseBracket, _range)) => break,
            Some((Token::Comma, _range)) => {
                matches(&iter.next(), Token::Comma)?;
            }
            Some((token, range)) => {
                log::error!("UnexpectedToken");
                return Err(Error::UnexpectedToken {
                    found: token.to_string(),
                    expected: ", or ]".to_string(),
                    range: range.clone(),
                });
            }
            None => return Err(Error::UnexpectedEnd),
        }
    }

    matches(&iter.next(), Token::CloseBracket)?;

    Ok((Expr::List(expressions), current))
}

/* A single value or a call site with some kind of single token or expression that we assume
 * resolves to a function if there are space separated arguments after it.
 */
fn parse_var_or_call(
    iter: &mut TokenIter,
    base: usize,
    mut current: usize,
) -> Result<(Expr, usize), Error> {
    log::trace!("parse_var_or_call: {:?}", iter.peek());
    let (var_or_func_expr, curr) = parse_singular_expression(iter, base, current)?;
    current = curr;

    // We have to keep parsing to look for more parts to this expression but if we find a change in
    // indentation that indicates the end of the scope for this expression then we just want to
    // return the expression we've found so far and allow the level up to deal with the change in
    // scope.
    let indent = indent::consume_to_indented(iter, base, current)?;
    if indent.in_scope() {
        current = indent.extract();
    } else {
        return Ok((var_or_func_expr, indent.extract()));
    }

    let mut args = Vec::new();

    loop {
        match iter.peek() {
            Some((Token::Operator(_), _range))
            | Some((Token::CloseParen, _range))
            | Some((Token::CloseBracket, _range))
            | Some((Token::Comma, _range))
            | Some((Token::Then, _range))
            | Some((Token::Else, _range)) => {
                // On certain tokens we know we've finish this 'var or call' and so we can exit and
                // let the parse_expression code handle it
                break;
            }
            _ => {}
        }

        let (argument_expr, curr) = parse_singular_expression(iter, base, current)?;
        current = curr;
        args.push(Rc::new(argument_expr));

        // Similar to above, we consume the expression on the right hand side of the operator and
        // then any whitespace afterwards (to reach the next operator if there is one) but if we
        // find that we're no longer in the indentation scope of the expression then we assume
        // we've reached the end of it and continue with processing what we've got so far
        let indent = indent::consume_to_indented(iter, base, current)?;
        if indent.in_scope() {
            current = indent.extract();
        } else {
            break;
        }
    }

    if args.is_empty() {
        Ok((var_or_func_expr, current))
    } else {
        Ok((
            Expr::Call {
                function: Rc::new(var_or_func_expr),
                args,
            },
            current,
        ))
    }
}

fn parse_if_expression(
    mut iter: &mut TokenIter,
    base: usize,
    mut current: usize,
) -> Result<(Expr, usize), Error> {
    log::trace!("parse_if_expression: {:?}", iter.peek());
    matches(&iter.next(), Token::If)?;
    current = indent::must_consume_to_indented(&mut iter, base, current)?;
    let (condition, curr) = parse_expression(&mut iter, current, current)?;
    current = curr;

    current = indent::must_consume_to_matching(&mut iter, base, current)?;
    matches(&iter.next(), Token::Then)?;

    current = indent::must_consume_to_indented(&mut iter, base, current)?;
    let (then_branch, curr) = parse_expression(&mut iter, current, current)?;
    current = curr;

    current = indent::must_consume_to_matching(&mut iter, base, current)?;
    matches(&iter.next(), Token::Else)?;

    current = indent::must_consume_to_indented(&mut iter, base, current)?;
    let (else_branch, current) = parse_expression(&mut iter, current, current)?;

    Ok((
        Expr::If {
            condition: Rc::new(condition),
            then_branch: Rc::new(then_branch),
            else_branch: Rc::new(else_branch),
        },
        current,
    ))
}
