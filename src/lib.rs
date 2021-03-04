//! This is a parser library for the
//! [CNI configuration format (CoNfiguration Initialization format)][CNI]
//! by libuconf. The implementation is fully compliant with the `core` and
//! `ini` part of the specification and with the extension `more-keys`.
//!
//! [CNI]: https://github.com/libuconf/cni/
//!
//! You can use the library like this:
//! ```
//! use std::collections::HashMap;
//!
//! let cni = r"
//! [section]
//! key = value
//! rkey = `raw value with `` escaped`
//! subsection.key = look, whitespace!
//! ";
//!
//! let parsed = cni::from_str(&cni).expect("could not parse CNI");
//!
//! // You can get everything, section names will be prepended to key names.
//! {
//!     let mut result: HashMap<String, String> = HashMap::new();
//!     result.insert("section.key".to_string(), "value".to_string());
//!     result.insert("section.rkey".to_string(), "raw value with ` escaped".to_string());
//!     result.insert("section.subsection.key".to_string(), "look, whitespace!".to_string());
//!
//!     assert_eq!(parsed, result);
//! }
//!
//! // You can get values from one section only.
//! {
//!     let mut section: HashMap<String, String> = HashMap::new();
//!     section.insert("key".to_string(), "value".to_string());
//!     section.insert("rkey".to_string(), "raw value with ` escaped".to_string());
//!     section.insert("subsection.key".to_string(), "look, whitespace!".to_string());
//!
//!     // use trait that adds CNI related functionality
//!     use cni::api::Cni;
//!
//!     // filter out values in section "section"
//!     assert_eq!(parsed.in_section("section"), section);
//! }
//!
//! // You can get child nodes from one section only, excluding subsections.
//! {
//!     let mut section: HashMap<String, String> = HashMap::new();
//!     section.insert("key".to_string(), "value".to_string());
//!     section.insert("rkey".to_string(), "raw value with ` escaped".to_string());
//!
//!     // use trait that adds CNI related functionality
//!     use cni::api::Cni;
//!
//!     // filter out values in section "section", but not in subsections
//!     assert_eq!(parsed.children_in_section("section"), section);
//! }
//! ```

use std::collections::HashMap;
use std::iter::Peekable;
use std::ops::Range;
use std::str::CharIndices;

#[cfg(test)]
mod tests;

pub mod api;

/// implements Perl's / Raku's "\v", i.e. vertical white space
fn is_vertical_ws(c: char) -> bool {
    matches!(
        c,
        '\n' | '\u{B}' | '\u{C}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

fn is_comment(c: char) -> bool {
    c == '#' || (cfg!(feature = "ini") && c == ';')
}

fn is_key(c: char) -> bool {
    if cfg!(feature = "more-keys") {
        !matches!(c, '[' | ']' | '=' | '`') && !is_comment(c) && !c.is_whitespace()
    } else {
        matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '-' | '_' | '.')
    }
}

/// An iterator that visits all key/value pairs in declaration order, even
/// key/value pairs that will be overwritten by later statements.
///
/// Calling `next` on this iterator after receiving a `Some(Err(_))` causes
/// undefined behaviour.
///
/// If you just want to access the resulting key/value store, take a look at
/// [`parse`].
pub struct CniParser<'source> {
    source: &'source str,
    /// The iterator stores the current position.
    iter: Peekable<CharIndices<'source>>,
    /// The current section name.
    section: Range<usize>,
}

impl<'a> CniParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            iter: source.char_indices().peekable(),
            section: 0..0,
        }
    }

    /// Skips whitespace.
    fn skip_ws(&mut self) {
        while matches!(
            self.iter.peek(),
            Some(&(_, c)) if c.is_whitespace()
        ) {
            self.iter.next();
        }
    }

    fn skip_comment(&mut self) {
        // skip any whitespace
        self.skip_ws();
        // if we arrive at a comment symbol now, skip the comment after it
        // otherwise do not because we might have also skipped over line ends
        if matches!(
            self.iter.peek(),
            Some(&(_, c)) if is_comment(c)
        ) {
            // continue until next vertical whitespace or EOF
            while matches!(self.iter.next(), Some((_, c)) if !is_vertical_ws(c)) {}
        }
    }

    fn parse_key(&mut self) -> Result<Range<usize>, &'static str> {
        let start = self.iter.peek().unwrap().0;
        let mut end = start;

        while matches!(self.iter.peek(), Some(&(_, c)) if is_key(c)) {
            end += self.iter.next().unwrap().1.len_utf8();
        }

        let key = &self.source[start..end];

        if key.starts_with('.') || key.ends_with('.') {
            // key cannot start or end with a dot
            Err("invalid key")
        } else {
            Ok(start..end)
        }
    }

    fn parse_value(&mut self) -> Result<String, &'static str> {
        // since raw values might have escaped backtics, they have to
        // be constructed as Strings and cannot be a reference.
        let mut value = String::new();

        if let Some(&(_, '`')) = self.iter.peek() {
            // raw value
            self.iter.next(); // consume backtick
            loop {
                if let Some((_, '`')) = self.iter.peek() {
                    // check if this is an escaped backtick
                    self.iter.next();
                    if let Some((_, '`')) = self.iter.peek() {
                        // escaped backtick
                        self.iter.next();
                        value.push('`');
                    } else {
                        // end of the value
                        break;
                    }
                } else if let Some((_, c)) = self.iter.next() {
                    value.push(c);
                } else {
                    // current value must have been a None
                    return Err("unterminated raw value");
                }
            }
        } else {
            // normal value: no comment starting character but white space, but not vertical space
            while matches!(self.iter.peek(), Some(&(_, c)) if !is_comment(c) && !( c.is_whitespace() && is_vertical_ws(c) ))
            {
                value.push(self.iter.next().unwrap().1);
            }
            // leading or trailing whitespace cannot be part of the value
            value = value.trim().to_string();
        }

        Ok(value)
    }
}

impl Iterator for CniParser<'_> {
    type Item = Result<(String, String), &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.skip_ws();
            // we should be at start of a line now
            let (_, c) = *self.iter.peek()?;
            if is_vertical_ws(c) {
                // empty line
                self.iter.next();
                continue;
            } else if is_comment(c) {
                self.skip_comment();
            } else if c == '[' {
                // section heading
                self.iter.next(); // consume [
                self.skip_ws();
                // this key can be empty
                match self.parse_key() {
                    Ok(key) => self.section = key,
                    Err(e) => return Some(Err(e)),
                };
                self.skip_ws();
                if self.iter.next().map_or(true, |(_, c)| c != ']') {
                    return Some(Err("expected \"]\""));
                }
                self.skip_comment();
            } else {
                // this should be a key/value pair

                // parse key, prepend it with section name if present
                let key = match self.parse_key() {
                    // this key cannot be empty
                    Ok(key) if key.is_empty() => return Some(Err("expected key")),
                    // do not prepend an empty section
                    Ok(key) if self.section.is_empty() => self.source[key].to_string(),
                    Ok(key) => format!(
                        "{}.{}",
                        &self.source[self.section.clone()],
                        &self.source[key]
                    ),
                    Err(e) => return Some(Err(e)),
                };

                self.skip_ws();
                if self.iter.next().map_or(true, |(_, c)| c != '=') {
                    return Some(Err("expected \"=\""));
                }
                self.skip_ws();

                let value = match self.parse_value() {
                    Ok(key) => key,
                    Err(e) => return Some(Err(e)),
                };

                self.skip_comment();

                break Some(Ok((key, value)));
            }
        }
    }
}

/// Parses CNI format text and returns the resulting key/value store.
///
/// This just constructs a [`CniParser`] and collects it.
///
/// For more information see the [crate level documentation](index.html).
pub fn from_str(text: &str) -> Result<HashMap<String, String>, &'static str> {
    CniParser::new(text).collect()
}
