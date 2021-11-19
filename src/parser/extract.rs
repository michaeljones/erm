use crate::ast::*;
use crate::lexer::{SrcToken, Token};

use super::error::Error;

pub fn extract_module_name(stream_token: &Option<SrcToken>) -> Result<ModuleName, Error> {
    log::trace!("extract_module_name: {:?}", stream_token);
    match stream_token {
        Some((Token::UpperPath(name), _range)) => {
            Ok(name.split('.').map(|str| str.to_string()).collect())
        }
        Some((Token::UpperName(name), _range)) => {
            Ok(name.split('.').map(|str| str.to_string()).collect())
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

pub fn extract_qualified_upper_name(
    stream_token: &Option<SrcToken>,
) -> Result<QualifiedUpperName, Error> {
    log::trace!("extract_qualified_upper_name: {:?}", stream_token);
    match stream_token {
        Some((Token::UpperPath(name), _range)) => QualifiedUpperName::from(name).ok_or_else(|| {
            log::error!("Unable to create upper name");
            Error::Unknown
        }),
        Some((Token::UpperName(name), _range)) => QualifiedUpperName::from(name).ok_or_else(|| {
            log::error!("Unable to create upper name");
            Error::Unknown
        }),
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

pub fn extract_upper_name(stream_token: &Option<SrcToken>) -> Result<UpperName, Error> {
    log::trace!("extract_upper_name: {:?}", stream_token);
    match stream_token {
        Some((Token::UpperName(name), _range)) => Ok(UpperName(name.to_string())),
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

pub fn extract_qualified_lower_name(
    stream_token: &Option<SrcToken>,
) -> Result<QualifiedLowerName, Error> {
    log::trace!("extract_qualified_lower_name: {:?}", stream_token);
    match stream_token {
        Some((Token::LowerName(name), _range)) => Ok(QualifiedLowerName::from(name.to_string())),
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: Token::LowerName("").to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}

pub fn extract_lower_name(stream_token: &Option<SrcToken>) -> Result<LowerName, Error> {
    log::trace!("extract_lower_name: {:?}", stream_token);
    match stream_token {
        Some((Token::LowerName(name), _range)) => Ok(LowerName(name.to_string())),
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: Token::LowerName("").to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}

pub fn extract_operator<'a>(stream_token: &Option<SrcToken<'a>>) -> Result<&'a str, Error> {
    log::trace!("extract_operator: {:?}", stream_token);
    match stream_token {
        Some((Token::Operator(op), _range)) => Ok(op),
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: Token::Operator("").to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}

pub fn extract_pattern_name(stream_token: &Option<SrcToken>) -> Result<Pattern, Error> {
    log::trace!("extract_pattern_name: {:?}", stream_token);
    match stream_token {
        Some((Token::LowerName(name), _range)) => Ok(Pattern::Name(name.to_string())),
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: Token::LowerName("").to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}

pub fn extract_associativity(stream_token: &Option<SrcToken>) -> Result<Associativity, Error> {
    log::trace!("extract_associativity: {:?}", stream_token);
    match stream_token {
        Some((Token::LowerName("left"), _range)) => Ok(Associativity::Left),
        Some((Token::LowerName("right"), _range)) => Ok(Associativity::Right),
        Some((Token::LowerName("non"), _range)) => Ok(Associativity::Non),
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: "LowerName with 'left', 'right', or 'non".to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}
