use crate::lexer::{Token, TokenIter};

pub fn consume_til_line_start(mut iter: &mut TokenIter) {
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
