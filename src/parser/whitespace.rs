use crate::lexer::{Token, TokenIter};

// Consumes space and comments
pub fn consume_spaces(iter: &mut TokenIter) {
    loop {
        match iter.peek() {
            Some((Token::Space(_), _range))
            | Some((Token::SingleLineComment(_), _range))
            | Some((Token::MultiLineComment(_), _range)) => {
                iter.next();
            }
            _ => {
                break;
            }
        }
    }
}
