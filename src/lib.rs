#![forbid(unsafe_code)]
#![warn(
    invalid_html_tags,
    keyword_idents,
    missing_docs,
    non_ascii_idents,
    trivial_casts,
    trivial_numeric_casts,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    clippy::cargo,
    clippy::pedantic
)]
//! This is a parser library for the
//! [CNI configuration format (**C**o**N**figuration **I**nitialization format)][CNI]
//! by libuconf.
//! # CNI standard compliance
//! The implementation is fully compliant with the `core` and
//! `ini` part of the specification and with the extension `more-keys`.
//!
//! [CNI]: https://github.com/libuconf/cni/
//!
//! # Examples
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
//! let parsed = cni_format::from_str(&cni).expect("could not parse CNI");
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
//! # #[cfg(feature = "api")]
//! {
//!     let mut section: HashMap<String, String> = HashMap::new();
//!     section.insert("key".to_string(), "value".to_string());
//!     section.insert("rkey".to_string(), "raw value with ` escaped".to_string());
//!     section.insert("subsection.key".to_string(), "look, whitespace!".to_string());
//!
//!     // use trait that adds CNI related functionality
//!     use cni_format::CniExt;
//!
//!     // filter out values in section "section"
//!     assert_eq!(parsed.sub_tree("section"), section);
//! }
//!
//! // You can get child nodes from one section only, excluding subsections.
//! # #[cfg(feature = "api")]
//! {
//!     let mut section: HashMap<String, String> = HashMap::new();
//!     section.insert("key".to_string(), "value".to_string());
//!     section.insert("rkey".to_string(), "raw value with ` escaped".to_string());
//!
//!     // use trait that adds CNI related functionality
//!     use cni_format::CniExt;
//!
//!     // filter out values in section "section", but not in subsections
//!     assert_eq!(parsed.sub_leaves("section"), section);
//! }
//! ```

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "api", test, doctest, doc))]
mod api;
#[cfg(any(feature = "api", test, doctest, doc))]
pub use api::{CniExt, SectionFilter};

#[cfg(any(feature = "serializer", test, doctest, doc))]
mod serializer;
#[cfg(any(feature = "serializer", test, doctest, doc))]
pub use serializer::to_str;

/// implements Perl's / Raku's "\v", i.e. vertical white space
fn is_vertical_ws(c: char) -> bool {
    matches!(
        c,
        '\n' | '\u{B}' | '\u{C}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

fn is_comment(c: char) -> bool {
    c == '#' || ((cfg!(feature = "ini") || cfg!(test)) && c == ';')
}

fn is_key(c: char) -> bool {
    if cfg!(feature = "more-keys") || cfg!(test) {
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
/// [`from_str`].
pub struct CniParser<I: Iterator> {
    /// The iterator stores the current position.
    iter: Peekable<I>,
    /// The current section name.
    section: String,
}

impl<I: Iterator<Item = char>> CniParser<I> {
    /// Creates a new `CniParser` that will parse the given CNI format text.
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            section: String::new(),
        }
    }

    /// Skips whitespace.
    fn skip_ws(&mut self) {
        while matches!(
            self.iter.peek(),
            Some(c) if c.is_whitespace()
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
            Some(&c) if is_comment(c)
        ) {
            // continue until next vertical whitespace or EOF
            while matches!(self.iter.next(), Some(c) if !is_vertical_ws(c)) {}
        }
    }

    fn parse_key(&mut self) -> Result<String, &'static str> {
        let mut key = String::new();

        while matches!(self.iter.peek(), Some(&c) if is_key(c)) {
            key.push(self.iter.next().unwrap());
        }

        if key.starts_with('.') || key.ends_with('.') {
            // key cannot start or end with a dot
            Err("invalid key")
        } else {
            Ok(key)
        }
    }

    fn parse_value(&mut self) -> Result<String, &'static str> {
        // since raw values might have escaped backtics, they have to
        // be constructed as Strings and cannot be a reference.
        let mut value = String::new();

        if let Some('`') = self.iter.peek() {
            // raw value
            self.iter.next(); // consume backtick
            loop {
                if let Some('`') = self.iter.peek() {
                    // check if this is an escaped backtick
                    self.iter.next();
                    if let Some('`') = self.iter.peek() {
                        // escaped backtick
                        self.iter.next();
                        value.push('`');
                    } else {
                        // end of the value
                        break;
                    }
                } else if let Some(c) = self.iter.next() {
                    value.push(c);
                } else {
                    // current value must have been a None
                    return Err("unterminated raw value");
                }
            }
        } else {
            // normal value: no comment starting character but white space, but not vertical space
            while matches!(self.iter.peek(), Some(&c) if !is_comment(c) && !( c.is_whitespace() && is_vertical_ws(c) ))
            {
                value.push(self.iter.next().unwrap());
            }
            // leading or trailing whitespace cannot be part of the value
            value = value.trim().to_string();
        }

        Ok(value)
    }
}

impl<'a> From<&'a str> for CniParser<Chars<'a>> {
    /// Create a `CniParser` from a string slice.
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    fn from(text: &'a str) -> Self {
        Self::new(text.chars())
    }
}

impl<I: Iterator<Item = char>> Iterator for CniParser<I> {
    type Item = Result<(String, String), &'static str>;

    /// Try to parse until the next key/value pair.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.skip_ws();
            // we should be at start of a line now
            let c = *self.iter.peek()?;
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
                    Ok(key) => self.section = key.to_string(),
                    Err(e) => return Some(Err(e)),
                };
                self.skip_ws();
                if self.iter.next().map_or(true, |c| c != ']') {
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
                    Ok(key) if self.section.is_empty() => key,
                    Ok(key) => format!("{}.{}", self.section, key),
                    Err(e) => return Some(Err(e)),
                };

                self.skip_ws();
                if self.iter.next().map_or(true, |c| c != '=') {
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
///
/// # Errors
/// Returns an `Err` if the given text is not in a valid CNI format. The `Err`
/// will contain a message explaining the error.
pub fn from_str(text: &str) -> Result<HashMap<String, String>, &'static str> {
    CniParser::from(text).collect()
}
