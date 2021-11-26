use super::Error;
use lexer::{Token, TokenIter};

#[derive(Clone)]
pub struct Indentation {
    pub lines: usize,
    pub spaces: usize,
}

impl Indentation {
    pub fn new() -> Self {
        Self {
            lines: 0,
            spaces: 0,
        }
    }

    pub fn at_line_start(&self) -> bool {
        self.spaces == 0
    }

    // Used to make sure that a new indentation exactly matches this one. For situations like case
    // statements where each branch should start at the same indentation
    pub fn matches(&self, other: &Self) -> bool {
        self.spaces == other.spaces
    }

    // Used to make sure that a new indentation is within a previous one. For situations where
    // we've parsed some expression and reached a new token and we want to know if that new token
    // is a continuation of that expression somehow or if it is at a shallower indent, indicating
    // that it is the start of something else
    pub fn indented_from(&self, other: &Self) -> bool {
        self.lines == other.lines || self.spaces > other.spaces
    }

    pub fn consume(&self, iter: &mut TokenIter) -> Indentation {
        log::trace!("consume");
        let mut new = self.clone();

        while let Some((ref token, _range)) = iter.peek() {
            match token {
                Token::NewLine => {
                    new.spaces = 0;
                    new.lines += 1;
                    iter.next();
                }
                Token::Space(ref count) => {
                    new.spaces += count;
                    iter.next();
                }
                _ => {
                    return new;
                }
            }
        }

        self.clone()
    }

    pub fn must_consume_to_line_start(&self, iter: &mut TokenIter) -> Result<(), Error> {
        let mut new = self.clone();

        while let Some((token, range)) = iter.peek() {
            match token {
                Token::NewLine => {
                    new.spaces = 0;
                    new.lines += 1;
                    iter.next();
                }
                Token::Space(count) => {
                    new.spaces += count;
                    iter.next();
                }
                Token::SingleLineComment(_) => {
                    iter.next();
                }
                Token::MultiLineComment(_) => {
                    iter.next();
                }
                _ => {
                    if new.spaces == 0 {
                        return Ok(());
                    } else {
                        return Err(Error::TokenNotAtLineStart(range.clone()));
                    }
                }
            }
        }

        // If we've run out of tokens then it is the next of the file and that is a kind of line
        // start too
        Ok(())
    }

    pub fn must_consume_to_indented(&self, iter: &mut TokenIter) -> Result<Self, Error> {
        log::trace!("must_consume_to_indented");
        let mut new = self.clone();

        while let Some((ref token, range)) = iter.peek() {
            match token {
                Token::NewLine => {
                    new.spaces = 0;
                    new.lines += 1;
                    iter.next();
                }
                Token::Space(count) => {
                    new.spaces += count;
                    iter.next();
                }
                _ => {
                    return if new.indented_from(self) {
                        Ok(new)
                    } else {
                        Err(Error::Indent {
                            range: range.clone(),
                        })
                    };
                }
            }
        }

        Err(Error::UnexpectedEnd)
    }
}
