use std::iter::Peekable;

/// Wrapper around an iterator that keeps track of the current line and column
/// position to produce proper diagnostics.
pub(crate) struct Iter<I: Iterator> {
    /// The internal iterator.
    iter: Peekable<I>,

    pub line: usize,
    pub col: usize,
}

impl<I: Iterator<Item = char>> Iter<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            line: 1,
            col: 1,
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }
}

impl<I: Iterator<Item = char>> Iterator for Iter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().filter(|&c| {
            if c == '\r' && self.iter.peek() == Some(&'\n') {
                // deal with CRLF line endings
                self.col += 1;
            } else if crate::is_vertical_ws(c) {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }

            // never filter out, this is just a convenient way to do something
            // if there is a character
            true
        })
    }
}
