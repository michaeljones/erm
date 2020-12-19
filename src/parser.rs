use lexer::Token;

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
    Function { name: &'a str, expr: Expr<'a> },
}

#[derive(Debug)]
pub enum Expr<'a> {
    Bool(bool),
    Integer(i32),
    Float(f32),
    String(&'a str),
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
}

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedToken { expected: String, found: String },
    ExpectedSpace(String),
    UnexpectedEnd,
    Indent,
    TokensRemaining,
    NoOperand,
    NoOperator,
}

type TokenIter<'a> = std::iter::Peekable<logos::Lexer<'a, Token<'a>>>;

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
    while let Some(token) = iter.peek() {
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
    while matches!(iter.peek(), Some(Token::Space(_))) {
        iter.next();
    }
}

// Imports
fn parse_imports<'a>(mut iter: &mut TokenIter<'a>) -> Result<Vec<Import<'a>>, Error> {
    let mut imports = vec![];

    loop {
        if !matches!(iter.peek(), Some(Token::Import)) {
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

struct Context {
    base_indent: usize,
    current_indent: usize,
}

impl Context {
    pub fn new() -> Context {
        Context {
            base_indent: 0,
            current_indent: 0,
        }
    }

    pub fn consume_white_space<'a>(&mut self, iter: &mut TokenIter<'a>) -> Result<(), Error> {
        while let Some(ref token) = iter.peek() {
            match token {
                Token::NewLine => {
                    iter.next();
                    self.current_indent = 0
                }
                Token::Space(ref count) => {
                    self.current_indent += count;
                    iter.next();
                }
                _ => {
                    if self.current_indent > self.base_indent {
                        return Ok(());
                    } else {
                        return Err(Error::Indent);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn child(&self) -> Context {
        Context {
            base_indent: self.base_indent,
            current_indent: self.current_indent,
        }
    }
}

// Statements
fn parse_statements<'a>(mut iter: &mut TokenIter<'a>) -> Result<Vec<Stmt<'a>>, Error> {
    let mut statements = vec![];

    loop {
        if !matches!(iter.peek(), Some(Token::LowerName(_))) {
            break;
        }

        let name = extract_var_name(&iter.next())?;
        matches_space(&iter.next())?;
        matches(&iter.next(), Token::Equals)?;
        let mut context = Context::new();
        context.consume_white_space(&mut iter)?;

        let expr = parse_expression(&mut iter, &mut context)?;

        statements.push(Stmt::Function { name, expr });

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
fn parse_expression<'a>(
    mut iter: &mut TokenIter<'a>,
    context: &mut Context,
) -> Result<Expr<'a>, Error> {
    let expr = parse_singular_expression(&mut iter, &mut context.child())?;
    context.consume_white_space(&mut iter)?;

    let mut operator_stack = Vec::new();
    let mut operand_stack = vec![expr];

    while matches!(iter.peek(), Some(Token::Operator(_))) {
        let operator = extract_operator(&iter.next())?;
        context.consume_white_space(&mut iter)?;

        process_stacks(operator, &mut operator_stack, &mut operand_stack)?;

        let right_hand_expr = parse_singular_expression(&mut iter, &mut context.child())?;
        context.consume_white_space(&mut iter)?;
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
    operand_stack.pop().ok_or(Error::NoOperand)
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

fn parse_singular_expression<'a>(
    mut iter: &mut TokenIter<'a>,
    context: &mut Context,
) -> Result<Expr<'a>, Error> {
    match iter.peek() {
        Some(Token::If) => parse_if_expression(&mut iter, &mut context.child()),
        Some(Token::LiteralInteger(int)) => {
            let result = Ok(Expr::Integer(*int));
            iter.next();
            result
        }
        Some(Token::LiteralFloat(float)) => {
            let result = Ok(Expr::Float(*float));
            iter.next();
            result
        }
        Some(Token::LiteralString(string)) => {
            let result = Ok(Expr::String(string));
            iter.next();
            result
        }
        Some(Token::UpperName("True")) => {
            let result = Ok(Expr::Bool(true));
            iter.next();
            result
        }
        Some(Token::UpperName("False")) => {
            let result = Ok(Expr::Bool(false));
            iter.next();
            result
        }
        Some(token) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: "Expression token".to_string(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn parse_if_expression<'a>(
    mut iter: &mut TokenIter<'a>,
    context: &mut Context,
) -> Result<Expr<'a>, Error> {
    matches(&iter.next(), Token::If)?;
    matches_space(&iter.next())?;
    let condition = parse_expression(&mut iter, &mut context.child())?;
    matches(&iter.next(), Token::Then)?;
    let then_branch = parse_expression(&mut iter, &mut context.child())?;
    matches(&iter.next(), Token::Else)?;
    let else_branch = parse_expression(&mut iter, &mut context.child())?;

    Ok(Expr::If {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    })
}

fn matches<'a>(stream_token: &Option<Token<'a>>, match_token: Token<'a>) -> Result<(), Error> {
    match stream_token {
        Some(token) => {
            if token == &match_token {
                Ok(())
            } else {
                Err(Error::UnexpectedToken {
                    found: token.to_string(),
                    expected: match_token.to_string(),
                })
            }
        }
        None => Err(Error::UnexpectedEnd),
    }
}

fn matches_space<'a>(stream_token: &Option<Token<'a>>) -> Result<(), Error> {
    match stream_token {
        Some(Token::Space(_)) => Ok(()),
        Some(token) => Err(Error::ExpectedSpace(token.to_string())),
        None => Err(Error::UnexpectedEnd),
    }
}

fn extract_upper_name<'a>(stream_token: &Option<Token<'a>>) -> Result<&'a str, Error> {
    match stream_token {
        Some(Token::UpperName(name)) => Ok(name),
        Some(token) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: Token::UpperName("").to_string(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn extract_var_name<'a>(stream_token: &Option<Token<'a>>) -> Result<&'a str, Error> {
    match stream_token {
        Some(Token::LowerName(name)) => Ok(name),
        Some(token) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: Token::LowerName("").to_string(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn extract_operator<'a>(stream_token: &Option<Token<'a>>) -> Result<&'a str, Error> {
    match stream_token {
        Some(Token::Operator(op)) => Ok(op),
        Some(token) => Err(Error::UnexpectedToken {
            found: token.to_string(),
            expected: Token::Operator("").to_string(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}
