use crate::lexer::{SrcToken, Token};

use super::error::Error;

pub fn matches<'a>(
    stream_token: &Option<SrcToken<'a>>,
    match_token: Token<'a>,
) -> Result<(), Error> {
    match stream_token {
        Some((token, range)) => {
            if token == &match_token {
                Ok(())
            } else {
                log::error!(
                    "UnexpectedToken. Expected: {:?} Found: {:?} \n\n {:?}",
                    match_token,
                    token,
                    backtrace::Backtrace::new()
                );
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

pub fn matches_space(stream_token: &Option<SrcToken>) -> Result<(), Error> {
    match stream_token {
        Some((Token::Space(_), _range)) => Ok(()),
        Some((token, range)) => {
            log::error!("UnexpectedToken");
            Err(Error::UnexpectedToken {
                found: token.to_string(),
                expected: "Space".to_string(),
                range: range.clone(),
            })
        }
        None => Err(Error::UnexpectedEnd),
    }
}
