use super::Error;
use lexer::{Token, TokenIter};

#[derive(Clone)]
pub enum IndentStatus {
    Inherited,
    Fresh,
}

#[derive(Clone)]
struct Indent {
    count: usize,
    status: IndentStatus,
}

impl Indent {
    fn inherited(count: usize) -> Self {
        Indent {
            status: IndentStatus::Inherited,
            count,
        }
    }

    fn fresh(count: usize) -> Self {
        Indent {
            status: IndentStatus::Fresh,
            count,
        }
    }

    fn add(&mut self, count: usize) -> Self {
        // Only add spaces if we've seen a new line
        match self.status {
            IndentStatus::Fresh => Indent {
                count: self.count + count,
                status: IndentStatus::Fresh,
            },
            _ => self.clone(),
        }
    }

    fn indented_from(&self, base: usize) -> bool {
        match self.status {
            IndentStatus::Inherited => true,
            IndentStatus::Fresh => self.count > base,
        }
    }

    fn matching(&self, base: usize) -> bool {
        match self.status {
            IndentStatus::Inherited => true,
            IndentStatus::Fresh => self.count == base,
        }
    }

    fn at_least(&self, base: usize) -> bool {
        match self.status {
            IndentStatus::Inherited => true,
            IndentStatus::Fresh => self.count >= base,
        }
    }

    fn extract(&self) -> usize {
        self.count
    }
}

#[derive(Clone)]
pub enum IndentScope {
    In(usize),
    Out(usize),
}

impl IndentScope {
    pub fn in_scope(&self) -> bool {
        match self {
            IndentScope::In(_) => true,
            IndentScope::Out(_) => false,
        }
    }

    pub fn extract(&self) -> usize {
        match self {
            IndentScope::In(curr) => *curr,
            IndentScope::Out(curr) => *curr,
        }
    }
}

/// For when we want to continue parsing the current expression if we find an indented line but if
/// we move out to a shallower indent then we can see that and move to the next part of the parsing
pub fn consume_to_indented(
    iter: &mut TokenIter,
    base: usize,
    start: usize,
) -> Result<IndentScope, Error> {
    let mut current = Indent::inherited(start);

    while let Some((ref token, _range)) = iter.peek() {
        match token {
            Token::NewLine => {
                current = Indent::fresh(0);
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

    // In this situation the iterator is returning None so we're at the end of the file/content and
    // so we're definitely 'out' of the previous scope. It might be sensible for as to introduce a
    // different IndentScope value for this
    Ok(IndentScope::Out(0))
}

/// For when the code is only valid if we continue on the same line at an indent on the next line.
/// For something like "if <expr> then". It isn't valid for <expr> to be at the same indent at the
/// if-keyword.
pub fn must_consume_to_indented(
    iter: &mut TokenIter,
    base: usize,
    start: usize,
) -> Result<usize, Error> {
    let mut current = Indent::inherited(start);

    while let Some((ref token, range)) = iter.peek() {
        match token {
            Token::NewLine => {
                current = Indent::fresh(0);
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

/// For when the code can continue on the same line or a more indented one. This might be the
/// closing parenthesis which can be at the same indent at the opening one or more indented.
pub fn must_consume_to_at_least(
    iter: &mut TokenIter,
    base: usize,
    start: usize,
) -> Result<usize, Error> {
    let mut current = Indent::inherited(start);

    while let Some((ref token, range)) = iter.peek() {
        match token {
            Token::NewLine => {
                current = Indent::fresh(0);
                iter.next();
            }
            Token::Space(count) => {
                current = current.add(*count);
                iter.next();
            }
            _ => {
                return if current.at_least(base) {
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

/// For when the code must be on the same line or at the expected indentation. eg. the 'if' and
/// 'else' keywords in an if-statement should be at the same indentation (or on the same line.)
pub fn must_consume_to_matching(
    iter: &mut TokenIter,
    base: usize,
    start: usize,
) -> Result<usize, Error> {
    let mut current = Indent::inherited(start);

    while let Some((ref token, range)) = iter.peek() {
        match token {
            Token::NewLine => {
                current = Indent::fresh(0);
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
