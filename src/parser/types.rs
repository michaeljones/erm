use crate::ast::*;
use crate::lexer::{Token, TokenIter};

use super::error::Error;
use super::extract;
use super::indent;
use super::mtch::matches;
use super::whitespace::*;

pub fn parse_type(iter: &mut TokenIter, base: usize, current: usize) -> Result<Type, Error> {
    log::trace!("parse_type: {:?}", iter.peek());
    match iter.peek() {
        Some((Token::UpperName(_), _range)) => {
            let mut type_ = parse_single_type(iter, base, current)?;
            consume_spaces(iter);

            loop {
                if !matches!(iter.peek(), Some((Token::RightArrow, _range))) {
                    break;
                }
                matches(&iter.next(), Token::RightArrow)?;
                consume_spaces(iter);

                let next_type = parse_single_type(iter, base, current)?;
                consume_spaces(iter);
                type_ = Type::Function {
                    from: Box::new(type_),
                    to: Box::new(next_type),
                }
            }

            Ok(type_)
        }
        _ => Err(Error::Unknown),
    }
}

// Parse up to the next "->" (RightArrow)
fn parse_single_type(iter: &mut TokenIter, base: usize, current: usize) -> Result<Type, Error> {
    log::trace!("parse_single_type: {:?}", iter.peek());
    let name = extract::extract_qualified_upper_name(&iter.next())?;
    let indent = indent::consume_to_indented(iter, base, current)?;
    if indent.in_scope() {
        // current = indent.extract();
    } else {
        return convert_name_to_type(name, vec![]);
    }

    let mut args = vec![];
    loop {
        if matches!(
            iter.peek(),
            Some((Token::RightArrow, _range)) | Some((Token::Bar, _range))
        ) {
            break;
        }

        let arg_name = extract::extract_qualified_upper_name(&iter.next())?;
        let arg_type = convert_name_to_type(arg_name, vec![])?;
        args.push(arg_type);

        let indent = indent::consume_to_indented(iter, base, current)?;
        if indent.in_scope() {
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
