use std::iter::Peekable;
use std::str::Chars;

/// Wrapper around an iterator that keeps track of the current line and column
/// position to produce proper diagnostics.
pub(crate) struct Iter<'src> {
    /// The internal iterator.
    iter: Peekable<Chars<'src>>,

    pub line: usize,
    pub col: usize,
}

impl<'src> Iter<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            iter: src.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }
}

impl<'src> Iterator for Iter<'src> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().filter(|c| {
            if crate::is_vertical_ws(c) {
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
