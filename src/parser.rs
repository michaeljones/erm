use lexer::{Range, SrcToken, Token};

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
        function: &'a str,
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
    TokensRemaining,
    NoOperand,
    NoOperator,
}

type TokenIter<'a> = std::iter::Peekable<logos::SpannedIter<'a, Token<'a>>>;

pub fn parse<'a>(mut iter: &mut TokenIter<'a>) -> Result<Module<'a>, Error> {
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

    if iter.next() == None {
        Ok(Module {
            name,
            imports,
            statements,
        })
    } else {
        Err(Error::TokensRemaining)
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

#[derive(Clone)]
enum IndentStatus {
    Inherited(usize),
    Fresh(usize),
}

impl IndentStatus {
    fn add(&mut self, count: usize) -> Self {
        // Only add spaces if we've seen a new line
        match self {
            IndentStatus::Fresh(curr) => IndentStatus::Fresh(*curr + count),
            _ => self.clone(),
        }
    }

    fn indented_from(&self, base: usize) -> bool {
        match self {
            IndentStatus::Inherited(_) => true,
            IndentStatus::Fresh(curr) => curr > &base,
        }
    }

    fn matching(&self, base: usize) -> bool {
        match self {
            IndentStatus::Inherited(_) => true,
            IndentStatus::Fresh(curr) => curr == &base,
        }
    }

    fn extract(&self) -> usize {
        match self {
            IndentStatus::Fresh(curr) => *curr,
            IndentStatus::Inherited(curr) => *curr,
        }
    }
}

#[derive(Clone)]
enum IndentScope {
    In(usize),
    Out(usize),
}

impl IndentScope {
    fn in_scope(&self) -> bool {
        match self {
            IndentScope::In(_) => true,
            IndentScope::Out(_) => false,
        }
    }
    fn extract(&self) -> usize {
        match self {
            IndentScope::In(curr) => *curr,
            IndentScope::Out(curr) => *curr,
        }
    }
}

fn consume_to_indented<'b>(
    iter: &mut TokenIter<'b>,
    base: usize,
    start: usize,
) -> Result<IndentScope, Error> {
    let mut current = IndentStatus::Inherited(start);

    while let Some((ref token, _range)) = iter.peek() {
        match token {
            Token::NewLine => {
                current = IndentStatus::Fresh(0);
                iter.next();
            }
            Token::Space(count) => {
                current = current.add(*count);
                iter.next();
            }
            _ => {
                return if current.indented_from(base) {
                    Ok(IndentScope::In(current.extract()))
                } else {
                    Ok(IndentScope::Out(current.extract()))
                };
            }
        }
    }

    Ok(IndentScope::In(0))
}

fn must_consume_to_indented<'b>(
    iter: &mut TokenIter<'b>,
    base: usize,
    start: usize,
) -> Result<usize, Error> {
    let mut current = IndentStatus::Inherited(start);

    while let Some((ref token, range)) = iter.peek() {
        match token {
            Token::NewLine => {
                current = IndentStatus::Fresh(0);
                iter.next();
            }
            Token::Space(count) => {
                current = current.add(*count);
                iter.next();
            }
            _ => {
                return if current.indented_from(base) {
                    Ok(current.extract())
                } else {
                    Err(Error::Indent {
                        range: range.clone(),
                    })
                };
            }
        }
    }

    Ok(0)
}

fn consume_to_matching<'b>(
    iter: &mut TokenIter<'b>,
    base: usize,
    start: usize,
) -> Result<usize, Error> {
    let mut current = IndentStatus::Inherited(start);

    while let Some((ref token, range)) = iter.peek() {
        match token {
            Token::NewLine => {
                current = IndentStatus::Fresh(0);
                iter.next();
            }
            Token::Space(count) => {
                current = current.add(*count);
                iter.next();
            }
            _ => {
                return if current.matching(base) {
                    Ok(current.extract())
                } else {
                    Err(Error::Indent {
                        range: range.clone(),
                    })
                };
            }
        }
    }

    Ok(0)
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

        let indent = consume_to_indented(&mut iter, base, current)?;
        current = indent.extract();

        let (expr, _current) = parse_expression(&mut iter, current, current)?;

        if args.is_empty() {
            statements.push(Stmt::Binding { name, expr });
        } else {
            statements.push(Stmt::Function { name, args, expr });
        }

        // TODO: Update/fix/change
        consume_til_line_start(&mut iter);
    }

    Ok(statements)
}

// Expressions
//
// Shunting yard approach based on:
//   - https://eli.thegreenplace.net/2009/03/20/a-recursive-descent-parser-with-an-infix-expression-evaluator
//   - http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm
//
fn parse_expression<'a, 'b>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    current: usize,
) -> Result<(Expr<'a>, usize), Error> {
    let (expr, mut current) = parse_singular_expression(&mut iter, base, current)?;

    // We have to keep parsing to look for more parts to this expression but if we find a change in
    // indentation that indicates the end of the scope for this expression then we just want to
    // return the expression we've found so far and allow the level up to deal with the change in
    // scope.
    let indent = consume_to_indented(&mut iter, base, current)?;
    if indent.in_scope() {
        current = indent.extract();
    } else {
        return Ok((expr, indent.extract()));
    }

    let mut operator_stack = Vec::new();
    let mut operand_stack = vec![expr];

    while matches!(iter.peek(), Some((Token::Operator(_), _range))) {
        let operator = extract_operator(&iter.next())?;
        current = must_consume_to_indented(&mut iter, base, current)?;

        process_stacks(operator, &mut operator_stack, &mut operand_stack)?;

        let (right_hand_expr, curr) = parse_singular_expression(&mut iter, base, current)?;
        current = curr;
        current = must_consume_to_indented(&mut iter, base, current)?;
        operand_stack.push(right_hand_expr);
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
    if has_greater_precendence(operator, &operator_stack) {
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

fn has_greater_precendence<'a>(operator_a: &'a str, operator_stack: &Vec<&'a str>) -> bool {
    if operator_stack.is_empty() {
        true
    } else {
        let precedence_a = precendence(operator_a);
        let precedence_b = operator_stack.last().map(|op| precendence(op)).unwrap_or(0);

        precedence_a > precedence_b
    }
}

// Based on:
//
//   - http://faq.elm-community.org/operators.html
//   - https://github.com/elm-lang/core/blob/master/src/Basics.elm#L72-L90
//
fn precendence<'a>(operator: &'a str) -> usize {
    match operator {
        "*" | "/" => 7,
        "+" | "-" => 6,
        "++" | "::" => 5,
        _ => 0,
    }
}

fn parse_singular_expression<'a, 'b>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    current: usize,
) -> Result<(Expr<'a>, usize), Error> {
    match iter.peek() {
        Some((Token::If, _range)) => parse_if_expression(&mut iter, base, current),
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

fn parse_if_expression<'a, 'b>(
    mut iter: &mut TokenIter<'a>,
    base: usize,
    mut current: usize,
) -> Result<(Expr<'a>, usize), Error> {
    matches(&iter.next(), Token::If)?;
    current = must_consume_to_indented(&mut iter, base, current)?;
    let (condition, curr) = parse_expression(&mut iter, current, current)?;
    current = curr;

    current = consume_to_matching(&mut iter, base, current)?;
    matches(&iter.next(), Token::Then)?;

    current = must_consume_to_indented(&mut iter, base, current)?;
    let (then_branch, curr) = parse_expression(&mut iter, current, current)?;
    current = curr;

    current = consume_to_matching(&mut iter, base, current)?;
    matches(&iter.next(), Token::Else)?;

    current = must_consume_to_indented(&mut iter, base, current)?;
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