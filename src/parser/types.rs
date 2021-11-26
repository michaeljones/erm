use crate::ast::*;
use crate::lexer::{Token, TokenIter};

use super::error::Error;
use super::extract;
use super::indent;
use super::mtch::matches;

pub fn parse_type_declaration(
    iter: &mut TokenIter,
    base_indent: &indent::Indentation,
) -> Result<Stmt, Error> {
    log::trace!("parse_type_declaration: {:?}", iter.peek());
    matches(&iter.next(), Token::Type)?;
    base_indent.must_consume_to_indented(iter)?;

    let name = extract::extract_upper_name(&iter.next())?;
    base_indent.must_consume_to_indented(iter)?;

    let mut args = vec![];

    loop {
        if matches!(&iter.peek(), Some((Token::Equals, _range))) {
            break;
        };

        let type_variable = extract::extract_lower_name(&iter.next())?;
        args.push(type_variable);
        base_indent.must_consume_to_indented(iter)?;
    }

    matches(&iter.next(), Token::Equals)?;
    base_indent.must_consume_to_indented(iter)?;

    let first_constructor = parse_type(iter, &base_indent)?;
    base_indent.must_consume_to_indented(iter)?;

    let mut constructors = vec![first_constructor];

    loop {
        if !matches!(iter.peek(), Some((Token::Bar, _range))) {
            break;
        }
        matches(&iter.next(), Token::Bar)?;
        base_indent.must_consume_to_indented(iter)?;

        let constructor = parse_type(iter, &base_indent)?;
        constructors.push(constructor);
        base_indent.must_consume_to_indented(iter)?;
    }

    Ok(Stmt::Type {
        name,
        args,
        constructors,
    })
}

pub fn parse_type(iter: &mut TokenIter, base_indent: &indent::Indentation) -> Result<Type, Error> {
    log::trace!("parse_type: {:?}", iter.peek());
    let mut type_ = parse_single_type(iter, &base_indent)?;
    base_indent.must_consume_to_indented(iter)?;

    loop {
        match iter.peek() {
            Some((Token::RightArrow, _range)) => {
                matches(&iter.next(), Token::RightArrow)?;
                base_indent.must_consume_to_indented(iter)?;

                let next_type = parse_single_type(iter, &base_indent)?;
                base_indent.must_consume_to_indented(iter)?;
                type_ = Type::Function {
                    from: Box::new(type_),
                    to: Box::new(next_type),
                }
            }
            Some((Token::CloseParen, _range)) => {
                break;
            }
            _ => {
                break;
            }
        }
    }

    Ok(type_)
}

// Parse up to the next "->" (RightArrow)
fn parse_single_type(
    iter: &mut TokenIter,
    base_indent: &indent::Indentation,
) -> Result<Type, Error> {
    log::trace!("parse_single_type: {:?}", iter.peek());

    match iter.peek() {
        Some((Token::UpperName(_), _range)) => parse_explicit_type(iter, &base_indent),
        Some((Token::UpperPath(_), _range)) => parse_explicit_type(iter, &base_indent),
        Some((Token::LowerName(_), _range)) => {
            let name = extract::extract_lower_name(&iter.next())?;
            Ok(Type::Var(name))
        }
        Some((token, range)) => Err(Error::UnexpectedToken {
            expected: "Not sure".to_string(),
            found: token.to_string(),
            range: range.clone(),
        }),
        None => Err(Error::UnexpectedEnd),
    }
}

fn parse_explicit_type(
    iter: &mut TokenIter,
    base_indent: &indent::Indentation,
) -> Result<Type, Error> {
    log::trace!("parse_explicit_type: {:?}", iter.peek());
    let name = extract::extract_qualified_upper_name(&iter.next())?;
    let indent = base_indent.consume(iter);
    if indent.indented_from(&base_indent) {
        // current = indent.extract();
    } else {
        return convert_name_to_type(name, vec![]);
    }

    let mut args = vec![];
    loop {
        if matches!(
            iter.peek(),
            Some((Token::CloseParen, _range))
                | Some((Token::RightArrow, _range))
                | Some((Token::Bar, _range))
        ) {
            break;
        }

        let arg_type = match iter.peek() {
            Some((Token::UpperName(_), _range)) => {
                let name = extract::extract_qualified_upper_name(&iter.next())?;
                convert_name_to_type(name, vec![])
            }
            Some((Token::LowerName(_), _range)) => {
                let name = extract::extract_lower_name(&iter.next())?;
                Ok(Type::Var(name))
            }
            Some((Token::OpenParen, _range)) => {
                matches(&iter.next(), Token::OpenParen)?;
                let type_ = parse_type(iter, &base_indent)?;
                matches(&iter.next(), Token::CloseParen)?;
                Ok(type_)
            }
            Some((token, range)) => Err(Error::UnexpectedToken {
                expected: "Not sure".to_string(),
                found: token.to_string(),
                range: range.clone(),
            }),
            None => Err(Error::UnexpectedEnd),
        }?;

        args.push(arg_type);

        let next_indent = base_indent.consume(iter);
        if next_indent.indented_from(&base_indent) {
            // current = indent.extract();
        } else {
            break;
        }
    }

    convert_name_to_type(name, args)
}

fn convert_name_to_type(name: QualifiedUpperName, mut args: Vec<Type>) -> Result<Type, Error> {
    log::trace!("convert_name_to_type: {:?} {:?}", name, args);
    let full_name = name.as_string();
    match full_name.as_str() {
        "Int" => Ok(Type::Int),
        "Float" => Ok(Type::Float),
        "Char" => Ok(Type::Char),
        "String" => Ok(Type::String),
        "List" => {
            if args.len() == 1 {
                args.pop()
                    .map(|arg| Type::List(Box::new(arg)))
                    .ok_or_else(|| {
                        log::error!(
                            "Failed to pop from array with one entry: {:?} {:?}",
                            name,
                            args
                        );
                        Error::Unknown
                    })
            } else {
                log::error!("List with too many or too few args: {:?} {:?}", name, args);
                Err(Error::Unknown)
            }
        }
        _ => Ok(Type::UserDefined { name, args }),
    }
}
