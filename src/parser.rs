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
use self::mtch::matches;

pub type ParseResult = Result<Module, Error>;

pub fn parse(iter: &mut TokenIter) -> ParseResult {
    log::trace!("parse");

    // Uncomment to print out whole token stream
    // println!("{:?}", iter.collect::<Vec<_>>());

    let base_indent = indent::Indentation::new();

    matches(&iter.next(), Token::Module)?;
    base_indent.must_consume_to_indented(iter)?;

    let name = extract::extract_module_name(&iter.next())?;
    base_indent.must_consume_to_indented(iter)?;

    log::trace!("module {:?}", name);

    let exposing = parse_exposing(iter)?;
    base_indent.must_consume_to_line_start(iter)?;

    let imports = parse_imports(iter)?;
    base_indent.must_consume_to_line_start(iter)?;

    let statements = parse_statements(iter)?;

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

fn parse_exposing(iter: &mut TokenIter) -> Result<Exposing, Error> {
    // Fresh indentation as exposing lines only need to be indented from zero
    let base_indent = indent::Indentation::new();

    matches(&iter.next(), Token::Exposing)?;
    base_indent.must_consume_to_indented(iter)?;

    matches(&iter.next(), Token::OpenParen)?;
    base_indent.must_consume_to_indented(iter)?;

    let exposing = match iter.peek() {
        Some((Token::Ellipsis, _range)) => {
            iter.next();
            Ok(Exposing::All)
        }
        Some(_) => parse_exposing_details(iter).map(Exposing::List),
        token => Err(Error::UnknownExposing(format!("{:?}", token))),
    }?;

    matches(&iter.next(), Token::CloseParen)?;

    Ok(exposing)
}

fn parse_exposing_details(iter: &mut TokenIter) -> Result<Vec<ExposingDetail>, Error> {
    // Fresh indentation as exposing lines only need to be indented from zero
    let base_indent = indent::Indentation::new();

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
        base_indent.must_consume_to_indented(iter)?;
    }

    Ok(details)
}

// Imports
fn parse_imports(iter: &mut TokenIter) -> Result<Vec<Import>, Error> {
    // Fresh indentation as import lines only need to be indented from zero
    let base_indent = indent::Indentation::new();

    let mut imports = vec![];

    loop {
        if !matches!(iter.peek(), Some((Token::Import, _range))) {
            break;
        }

        matches(&iter.next(), Token::Import)?;
        base_indent.must_consume_to_indented(iter)?;

        let module_name = extract::extract_module_name(&iter.next())?;

        let mut exposing = None;

        let next_token_indent = base_indent.consume(iter);

        match iter.peek() {
            Some((Token::Exposing, range)) => {
                if next_token_indent.at_line_start() {
                    log::error!("Indentation of exposing");
                    return Err(Error::Indent {
                        range: range.clone(),
                    });
                }

                exposing = Some(parse_exposing(iter)?);
                base_indent.must_consume_to_line_start(iter)?;
            }
            Some((_, range)) => {
                if !next_token_indent.at_line_start() {
                    return Err(Error::TokenNotAtLineStart(range.clone()));
                }
            }
            None => {}
        }

        imports.push(Import {
            module_name,
            exposing,
        });
    }

    Ok(imports)
}

// Statements
fn parse_statements(iter: &mut TokenIter) -> Result<Vec<Rc<Stmt>>, Error> {
    log::trace!("parse_statements: {:?}", iter.peek());

    // Fresh indentation as statement lines only need to be indented from zero
    // Until we're parsing let-in blocks anyway
    let base_indent = indent::Indentation::new();

    let mut statements = vec![];

    let base = 0;
    let current = 0;

    loop {
        match iter.peek() {
            Some((Token::LowerName(_), _range)) => {
                // Get the name
                let name = extract::extract_lower_name(&iter.next())?;
                whitespace::consume_spaces(iter);

                let statement = if matches!(iter.peek(), Some((Token::Colon, _range))) {
                    let type_annotation = parse_type_annotation(iter, name.clone())?;
                    base_indent.must_consume_to_line_start(iter)?;

                    let function_name = extract::extract_lower_name(&iter.next())?;
                    base_indent.must_consume_to_indented(iter)?;

                    if function_name != name {
                        return Err(Error::NameMismatch);
                    }

                    parse_function_or_binding(
                        iter,
                        function_name,
                        Some(type_annotation),
                        &base_indent,
                    )?
                } else {
                    parse_function_or_binding(iter, name, None, &base_indent)?
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
fn parse_type_annotation(iter: &mut TokenIter, name: LowerName) -> Result<TypeAnnotation, Error> {
    log::trace!("parse_type_annotation: {:?}", name);
    matches(&iter.next(), Token::Colon)?;
    whitespace::consume_spaces(iter);

    let base = 0;
    let current = 0;
    let type_ = types::parse_type(iter, base, current)?;
    whitespace::consume_spaces(iter);

    Ok(TypeAnnotation {
        // TODO: Don't use lower name for this stuff
        name,
        type_,
    })
}

// Functions & bindings
//
// Matches:
//
//   myFunc argA argB = < expr >
//          ^^^^^^^^^^^^^^^^^^^^
//
fn parse_function_or_binding(
    iter: &mut TokenIter,
    name: LowerName,
    type_annotation: Option<TypeAnnotation>,
    base_indent: &indent::Indentation,
) -> Result<Stmt, Error> {
    log::trace!("parse_function_or_binding: {:?}", name);
    let mut args = Vec::new();
    loop {
        if !matches!(iter.peek(), Some((Token::LowerName(_), _range))) {
            break;
        }

        let arg = extract::extract_pattern_name(&iter.next())?;
        args.push(arg);

        base_indent.must_consume_to_indented(iter)?;
    }

    matches(&iter.next(), Token::Equals)?;

    base_indent.must_consume_to_indented(iter)?;

    let (expr, _) = parse_expression(iter, base_indent)?;

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
    base_indent: &indent::Indentation,
) -> Result<(Expr, indent::Indentation), Error> {
    log::trace!("parse_expression: {:?}", iter.peek());
    match iter.peek() {
        Some((Token::If, _range)) => parse_if_expression(&mut iter, base_indent),
        Some((Token::Case, _range)) => parse_case_expression(&mut iter, base_indent),
        Some(_) => parse_binary_expression(&mut iter, base_indent),
        None => Err(Error::UnexpectedEnd),
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
    base_indent: &indent::Indentation,
) -> Result<(Expr, indent::Indentation), Error> {
    log::trace!("parse_binary_expression: {:?}", iter.peek());
    let (expr, next_token_indent) = parse_var_or_call(&mut iter, base_indent)?;

    // We have to keep parsing to look for more parts to this expression but if we find a change in
    // indentation that indicates the end of the scope for this expression then we just want to
    // return the expression we've found so far and allow the level up to deal with the change in
    // scope.
    if !next_token_indent.within(&base_indent) {
        log::trace!("exiting parse_binary_expression: {:?}", iter.peek());
        return Ok((expr, next_token_indent));
    }

    let mut operator_stack = Vec::new();
    let mut operand_stack = vec![expr];

    let next_token_indent = loop {
        if !matches!(iter.peek(), Some((Token::Operator(_), _range))) {
            break next_token_indent;
        }

        let operator = extract::extract_operator(&iter.next())?;
        base_indent.must_consume_to_indented(iter)?;

        process_stacks(operator, &mut operator_stack, &mut operand_stack)?;

        let (right_hand_expr, next_token_indent) = parse_var_or_call(&mut iter, base_indent)?;
        operand_stack.push(right_hand_expr);

        // Similar to above, we consume the expression on the right hand side of the operator and
        // then any whitespace afterwards (to reach the next operator if there is one) but if we
        // find that we're no longer in the indentation scope of the expression then we assume
        // we've reached the end of it and continue with processing what we've got so far
        if !next_token_indent.within(&base_indent) {
            log::trace!("exiting parse_binary_expression: {:?}", iter.peek());
            break next_token_indent;
        }
    };

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
        .ok_or(Error::NoOperand)
        .map(|expr| (expr, next_token_indent))
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
    base_indent: &indent::Indentation,
) -> Result<(Expr, indent::Indentation), Error> {
    log::trace!("parse_singular_expression: {:?}", iter.peek());
    let expr = match iter.peek() {
        Some((Token::OpenParen, _range)) => {
            matches(&iter.next(), Token::OpenParen)?;
            let (expr, _) = parse_expression(iter, base_indent)?;
            base_indent.must_consume_to_indented(iter)?;

            matches(&iter.next(), Token::CloseParen)?;

            Ok(expr)
        }
        Some((Token::OpenBracket, _range)) => parse_list_literal(iter, base_indent),
        None => Err(Error::UnexpectedEnd),
        _ => parse_contained_expression(iter),
    }?;

    let next_token_indent = base_indent.consume(iter);
    Ok((expr, next_token_indent))
}

fn parse_contained_expression(iter: &mut TokenIter) -> Result<Expr, Error> {
    log::trace!("parse_contained_expression: {:?}", iter.peek());
    match iter.peek() {
        Some((Token::LiteralInteger(int), _range)) => {
            let result = Ok(Expr::Integer(*int));
            iter.next();
            result
        }
        Some((Token::LiteralFloat(float), _range)) => {
            let result = Ok(Expr::Float(*float));
            iter.next();
            result
        }
        Some((Token::LiteralString(string), _range)) => {
            let result = Ok(Expr::String(string.to_string()));
            iter.next();
            result
        }
        Some((Token::UpperName("True"), _range)) => {
            let result = Ok(Expr::Bool(true));
            iter.next();
            result
        }
        Some((Token::UpperName("False"), _range)) => {
            let result = Ok(Expr::Bool(false));
            iter.next();
            result
        }
        Some((Token::LowerName(name), _range)) => {
            let result = Ok(Expr::VarName(QualifiedLowerName::from(name.to_string())));
            iter.next();
            result
        }
        Some((Token::LowerPath(name), _range)) => {
            let result = Ok(Expr::VarName(QualifiedLowerName::from(name.to_string())));
            iter.next();
            result
        }
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
    base_indent: &indent::Indentation,
) -> Result<Expr, Error> {
    log::trace!("parse_list_literal: {:?}", iter.peek());
    matches(&iter.next(), Token::OpenBracket)?;

    let mut expressions = Vec::new();

    loop {
        base_indent.must_consume_to_indented(iter)?;

        if let Some((Token::CloseBracket, _range)) = iter.peek() {
            break;
        }

        let (expr, _) = parse_expression(iter, base_indent)?;
        expressions.push(Rc::new(expr));

        base_indent.must_consume_to_indented(iter)?;

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

    Ok(Expr::List(expressions))
}

/* A single value or a call site with some kind of single token or expression that we assume
 * resolves to a function if there are space separated arguments after it.
 */
fn parse_var_or_call(
    iter: &mut TokenIter,
    base_indent: &indent::Indentation,
) -> Result<(Expr, indent::Indentation), Error> {
    log::trace!("parse_var_or_call: {:?}", iter.peek());
    let (var_or_func_expr, mut next_token_indent) = parse_singular_expression(iter, base_indent)?;

    // If the next token is within our base indent then we assume we have more of the expression to
    // parse but if it is at a shallower indent then we assume it is a separate entity
    if !next_token_indent.within(&base_indent) {
        log::trace!("exiting parse_var_or_call: {:?}", iter.peek());
        return Ok((var_or_func_expr, next_token_indent));
    }

    let mut args = Vec::new();

    next_token_indent = loop {
        match iter.peek() {
            Some((Token::Operator(_), _))
            | Some((Token::CloseParen, _))
            | Some((Token::CloseBracket, _))
            | Some((Token::Comma, _))
            | Some((Token::Then, _))
            | Some((Token::Else, _))
            | Some((Token::Of, _))
            | Some((Token::RightArrow, _))
            | None => {
                // On certain tokens we know we've finish this 'var or call' and so we can exit and
                // let the parse_expression code handle it
                break next_token_indent;
            }
            _ => {}
        }

        let (argument_expr, next_token_indent) = parse_singular_expression(iter, base_indent)?;
        args.push(Rc::new(argument_expr));

        // Similar to above, we consume the expression on the right hand side of the operator and
        // then any whitespace afterwards (to reach the next operator if there is one) but if we
        // find that we're no longer in the indentation scope of the expression then we assume
        // we've reached the end of it and continue with processing what we've got so far
        if !next_token_indent.within(&base_indent) {
            break next_token_indent;
        }
    };

    if args.is_empty() {
        Ok((var_or_func_expr, next_token_indent))
    } else {
        Ok((
            Expr::Call {
                function: Rc::new(var_or_func_expr),
                args,
            },
            next_token_indent,
        ))
    }
}

fn parse_if_expression(
    iter: &mut TokenIter,
    base_indent: &indent::Indentation,
) -> Result<(Expr, indent::Indentation), Error> {
    log::trace!("parse_if_expression: {:?}", iter.peek());
    matches(&iter.next(), Token::If)?;
    base_indent.must_consume_to_indented(iter)?;

    let (condition, _) = parse_expression(iter, base_indent)?;
    base_indent.must_consume_to_indented(iter)?;

    matches(&iter.next(), Token::Then)?;
    base_indent.must_consume_to_indented(iter)?;

    let (then_branch, _) = parse_expression(iter, base_indent)?;
    base_indent.must_consume_to_indented(iter)?;

    matches(&iter.next(), Token::Else)?;
    base_indent.must_consume_to_indented(iter)?;

    let (else_branch, next_token_indent) = parse_expression(iter, base_indent)?;

    Ok((
        Expr::If {
            condition: Rc::new(condition),
            then_branch: Rc::new(then_branch),
            else_branch: Rc::new(else_branch),
        },
        next_token_indent,
    ))
}

fn parse_case_expression(
    iter: &mut TokenIter,
    base_indent: &indent::Indentation,
) -> Result<(Expr, indent::Indentation), Error> {
    log::trace!("parse_case_expression: {:?}", iter.peek());
    matches(&iter.next(), Token::Case)?;
    base_indent.must_consume_to_indented(iter)?;

    let (expr, _) = parse_expression(iter, base_indent)?;
    base_indent.must_consume_to_indented(iter)?;

    matches(&iter.next(), Token::Of)?;
    let branch_indent = base_indent.must_consume_to_indented(iter)?;

    let mut branches = vec![];

    let next_token_indent = loop {
        match iter.peek() {
            None => {
                // If there are no more tokens then we've finished parsing the possible cases
                break branch_indent;
            }
            _ => {}
        }

        let pattern = parse_pattern(iter)?;
        branch_indent.must_consume_to_indented(iter)?;

        matches(&iter.next(), Token::RightArrow)?;
        branch_indent.must_consume_to_indented(iter)?;

        let (expr, next_token_indent) = parse_expression(iter, &branch_indent)?;
        branches.push((pattern, expr));

        if next_token_indent.matches(&branch_indent) {
            continue;
        } else {
            break next_token_indent;
        }
    };

    Ok((
        Expr::Case {
            expr: Rc::new(expr),
            branches,
        },
        next_token_indent,
    ))
}

fn parse_pattern(iter: &mut TokenIter) -> Result<Pattern, Error> {
    match iter.peek() {
        Some((Token::UpperName("True"), _range)) => {
            let result = Ok(Pattern::Bool(true));
            iter.next();
            result
        }
        Some((Token::UpperName("False"), _range)) => {
            let result = Ok(Pattern::Bool(false));
            iter.next();
            result
        }
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: "Expression token".to_string(),
                range: range.clone(),
            })
        }
        None => {
            log::error!("UnexpectedEnd");
            Err(Error::UnexpectedEnd)
        }
    }
}
