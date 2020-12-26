mod indent;

use lexer::{Range, SrcToken, Token, TokenIter};

#[derive(Debug)]
pub struct Module<'a> {
    name: &'a str,
    imports: Vec<Import<'a>>,
    pub statements: Vec<Stmt<'a>>,
}

#[derive(Debug)]
pub struct Import<'a> {
    module_name: &'a str,
}

#[derive(Debug)]
pub enum Stmt<'a> {
    Binding {
        name: &'a str,
        expr: Expr<'a>,
    },
    Function {
        name: &'a str,
        args: Vec<Pattern<'a>>,
        expr: Expr<'a>,
    },
}

#[derive(Debug)]
pub enum Pattern<'a> {
    Name(&'a str),
}

#[derive(Debug)]
pub enum Expr<'a> {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(&'a str),
    List(Vec<Expr<'a>>),
    BinOp {
        operator: &'a str,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
    },
    If {
        condition: Box<Expr<'a>>,
        then_branch: Box<Expr<'a>>,
        else_branch: Box<Expr<'a>>,
    },
    Call {
        function_name: &'a str,
        args: Vec<Expr<'a>>,
    },
    VarName(&'a str),
}

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedToken {
        expected: String,
        found: String,
        range: Range,
    },
    UnexpectedEnd,
    Indent {
        range: Range,
    },
    TokensRemaining(Vec<String>),
    NoOperand,
    NoOperator,
    EmptyOperatorStack,
    UnknownOperator(String),
}

pub type ParseResult<'src> = Result<Module<'src>, Error>;

pub fn parse<'src>(mut iter: &mut TokenIter<'src>) -> ParseResult<'src> {
    matches(&iter.next(), Token::Module)?;
    matches_space(&iter.next())?;
    let name = extract_upper_name(&iter.next())?;
    matches_space(&iter.next())?;
    matches(&iter.next(), Token::Exposing)?;
    matches_space(&iter.next())?;
    matches(&iter.next(), Token::OpenParen)?;
    matches(&iter.next(), Token::Ellipsis)?;
    matches(&iter.next(), Token::CloseParen)?;

    consume_til_line_start(&mut iter);

    let imports = parse_imports(&mut iter)?;

    consume_til_line_start(&mut iter);

    let statements = parse_statements(&mut iter)?;

    if iter.peek() == None {
        Ok(Module {
            name,
            imports,
            statements,
        })
    } else {
        let tokens = iter.map(|token| format!("{:?}", token)).collect();
        Err(Error::TokensRemaining(tokens))
    }
}

fn consume_til_line_start<'a>(mut iter: &mut TokenIter<'a>) {
    while let Some((token, _range)) = iter.peek() {
        match token {
            Token::NewLine => {
                iter.next();
                consume_spaces(&mut iter);
            }
            _ => return,
        }
    }
}

fn consume_spaces(iter: &mut TokenIter) {
    while matches!(iter.peek(), Some((Token::Space(_), _range))) {
        iter.next();
    }
}

// Imports
fn parse_imports<'a>(mut iter: &mut TokenIter<'a>) -> Result<Vec<Import<'a>>, Error> {
    let mut imports = vec![];

    loop {
        if !matches!(iter.peek(), Some((Token::Import, _range))) {
            break;
        }

        matches(&iter.next(), Token::Import)?;
        matches_space(&iter.next())?;
        let module_name = extract_upper_name(&iter.next())?;

        imports.push(Import { module_name });

        consume_til_line_start(&mut iter);
    }

    Ok(imports)
}

// Statements
fn parse_statements<'a>(mut iter: &mut TokenIter<'a>) -> Result<Vec<Stmt<'a>>, Error> {
    let mut statements = vec![];

    loop {
        if !matches!(iter.peek(), Some((Token::LowerName(_), _range))) {
            break;
        }

        let name = extract_var_name(&iter.next())?;
        consume_spaces(&mut iter);

        let mut args = Vec::new();
        loop {
            if !matches!(iter.peek(), Some((Token::LowerName(_), _range))) {
                break;
            }

            let arg = extract_pattern_name(&iter.next())?;
            args.push(arg);

            consume_spaces(&mut iter);
        }

        matches(&iter.next(), Token::Equals)?;

        let base = 0;
        let mut current = 0;

        let indent = indent::consume_to_indented(&mut iter, base, current)?;
        current = indent.extract();

        let (expr, curr) = parse_expression(&mut iter, current, current)?;
        current = curr;

        if args.is_empty() {
            statements.push(Stmt::Binding { name, expr });
        } else {
            statements.push(Stmt::Function { name, args, expr });
        }

        // TODO: Update/fix/change
        indent::must_consume_to_matching(&mut iter, base, current)?;
    }

    Ok(statements)
}

// Expressions
//
fn parse_expression<'a, 'b>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    current: usize,
) -> Result<(Expr<'a>, usize), Error> {
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
fn parse_binary_expression<'a, 'b>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    current: usize,
) -> Result<(Expr<'a>, usize), Error> {
    let (expr, mut current) = parse_singular_expression(&mut iter, base, current)?;

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
        let operator = extract_operator(&iter.next())?;
        current = indent::must_consume_to_indented(&mut iter, base, current)?;

        process_stacks(operator, &mut operator_stack, &mut operand_stack)?;

        let (right_hand_expr, curr) = parse_singular_expression(&mut iter, base, current)?;
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

    while operator_stack.len() > 0 {
        let operator = operator_stack.pop().ok_or(Error::NoOperator)?;
        let right_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;
        let left_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;

        operand_stack.push(Expr::BinOp {
            operator,
            left: Box::new(left_hand_expr),
            right: Box::new(right_hand_expr),
        })
    }

    assert!(operand_stack.len() == 1);
    operand_stack
        .pop()
        .map(|expr| (expr, current))
        .ok_or(Error::NoOperand)
}

fn process_stacks<'a>(
    operator: &'a str,
    mut operator_stack: &mut Vec<&'a str>,
    mut operand_stack: &mut Vec<Expr<'a>>,
) -> Result<(), Error> {
    if has_greater_precendence(operator, &operator_stack)? {
        operator_stack.push(operator);
    } else {
        let right_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;
        let left_hand_expr = operand_stack.pop().ok_or(Error::NoOperand)?;
        let stored_operator = operator_stack.pop().ok_or(Error::NoOperator)?;

        operand_stack.push(Expr::BinOp {
            operator: stored_operator,
            left: Box::new(left_hand_expr),
            right: Box::new(right_hand_expr),
        });

        process_stacks(operator, &mut operator_stack, &mut operand_stack)?;
    };

    Ok(())
}

fn has_greater_precendence<'a>(
    operator_a: &'a str,
    operator_stack: &Vec<&'a str>,
) -> Result<bool, Error> {
    if operator_stack.is_empty() {
        Ok(true)
    } else {
        let precedence_a = precendence(operator_a)?;
        let precedence_b = operator_stack
            .last()
            .ok_or(Error::EmptyOperatorStack)
            .and_then(|op| precendence(op))?;

        Ok(precedence_a > precedence_b)
    }
}

// Based on:
//
//   - http://faq.elm-community.org/operators.html
//   - https://github.com/elm-lang/core/blob/master/src/Basics.elm#L72-L90
//
fn precendence<'a>(operator: &'a str) -> Result<usize, Error> {
    match operator {
        "*" | "/" => Ok(7),
        "+" | "-" => Ok(6),
        "++" | "::" => Ok(5),
        "==" | "/=" | ">" | "<" | "<=" | ">=" => Ok(4),
        _ => Err(Error::UnknownOperator(operator.to_string())),
    }
}

fn parse_singular_expression<'a, 'b>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    current: usize,
) -> Result<(Expr<'a>, usize), Error> {
    match iter.peek() {
        Some((Token::OpenParen, _range)) => {
            matches(&iter.next(), Token::OpenParen)?;
            let (expr, current) = parse_expression(&mut iter, base, current)?;
            let current = indent::must_consume_to_at_least(&mut iter, base, current)?;
            matches(&iter.next(), Token::CloseParen)?;
            Ok((expr, current))
        }
        Some((Token::LowerName(_), _range)) => parse_var_or_call(&mut iter, base, current),
        None => Err(Error::UnexpectedEnd),
        _ => parse_contained_expression(&mut iter, base, current),
    }
}

fn parse_contained_expression<'a, 'b>(
    iter: &mut TokenIter<'a>,
    _base: usize,
    current: usize,
) -> Result<(Expr<'a>, usize), Error> {
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
            let result = Ok((Expr::String(string), current));
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
            let result = Ok((Expr::VarName(name), current));
            iter.next();
            result
        }
        Some((token, range)) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: "Expression token".to_string(),
            range: range.clone(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn parse_var_or_call<'a>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    mut current: usize,
) -> Result<(Expr<'a>, usize), Error> {
    let name = extract_var_name(&iter.next())?;

    // We have to keep parsing to look for more parts to this expression but if we find a change in
    // indentation that indicates the end of the scope for this expression then we just want to
    // return the expression we've found so far and allow the level up to deal with the change in
    // scope.
    let indent = indent::consume_to_indented(&mut iter, base, current)?;
    if indent.in_scope() {
        current = indent.extract();
    } else {
        return Ok((Expr::VarName(name), indent.extract()));
    }

    let mut args = Vec::new();

    loop {
        match iter.peek() {
            Some((Token::Operator(_), _range)) | Some((Token::CloseParen, _range)) => {
                // If we've found an operator or closed paren then we want to exit and let the
                // parse_expression code handle it
                break;
            }
            _ => {}
        }

        let (argument_expr, curr) = parse_singular_expression(&mut iter, base, current)?;
        current = curr;
        args.push(argument_expr);

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

    if args.is_empty() {
        Ok((Expr::VarName(name), current))
    } else {
        Ok((
            Expr::Call {
                function_name: name,
                args,
            },
            current,
        ))
    }
}

fn parse_if_expression<'a>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    mut current: usize,
) -> Result<(Expr<'a>, usize), Error> {
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
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        },
        current,
    ))
}

fn matches<'a>(stream_token: &Option<SrcToken<'a>>, match_token: Token<'a>) -> Result<(), Error> {
    match stream_token {
        Some((token, range)) => {
            if token == &match_token {
                Ok(())
            } else {
                Err(Error::UnexpectedToken {
                    found: token.to_string(),
                    expected: match_token.to_string(),
                    range: range.clone(),
                })
            }
        }
        None => Err(Error::UnexpectedEnd),
    }
}

fn matches_space<'a>(stream_token: &Option<SrcToken<'a>>) -> Result<(), Error> {
    match stream_token {
        Some((Token::Space(_), _range)) => Ok(()),
        Some((token, range)) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: "Space".to_string(),
            range: range.clone(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn extract_upper_name<'a>(stream_token: &Option<SrcToken<'a>>) -> Result<&'a str, Error> {
    match stream_token {
        Some((Token::UpperName(name), _range)) => Ok(name),
        Some((token, range)) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: Token::UpperName("").to_string(),
            range: range.clone(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn extract_var_name<'a>(stream_token: &Option<SrcToken<'a>>) -> Result<&'a str, Error> {
    match stream_token {
        Some((Token::LowerName(name), _range)) => Ok(name),
        Some((token, range)) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: Token::LowerName("").to_string(),
            range: range.clone(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn extract_operator<'a>(stream_token: &Option<SrcToken<'a>>) -> Result<&'a str, Error> {
    match stream_token {
        Some((Token::Operator(op), _range)) => Ok(op),
        Some((token, range)) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: Token::Operator("").to_string(),
            range: range.clone(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn extract_pattern_name<'a>(stream_token: &Option<SrcToken<'a>>) -> Result<Pattern<'a>, Error> {
    match stream_token {
        Some((Token::LowerName(name), _range)) => Ok(Pattern::Name(name)),
        Some((token, range)) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: Token::LowerName("").to_string(),
            range: range.clone(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}
