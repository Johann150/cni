//! This is a parser library for the [CNI configuration format (CoNfiguration Initialization format)](https://github.com/libuconf/cni/) by libuconf.
//! The implementation is fully compliant with the `core` and `ini` part of the specification and with both extensions `flexspace` and `tabulation`.
//!
//! You can use the library like this:
//! ```
//! # use std::collections::HashMap;
//! let cni = r"
//! [section]
//! key = value
//! rkey = `raw value with `` escaped`
//! ";
//!
//! let parsed = cni::parse(&cni).expect("could not parse CNI");
//!
//! let mut result = HashMap::new();
//! result.insert("section.key".to_string(), "value".to_string());
//! result.insert("section.rkey".to_string(), "raw value with ` escaped".to_string());
//!
//! assert_eq!(parsed,result);
//! ```

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[cfg(test)]
mod tests;

fn is_vertical_ws(c: char) -> bool {
    // Perl's / Raku's "\v"
    matches!(
        c,
        '\n' | '\u{B}' | '\u{C}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

fn skip_ws(chars: &mut Peekable<Chars>) {
    while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
        chars.next();
    }
}

fn parse_key(chars: &mut Peekable<Chars>) -> Result<String, &'static str> {
    // because chars is a &mut, the iterator is not moved and can still be used
    // after this call
    let mut key = String::new();

    while matches!(
        chars.peek(),
        Some('0'..='9') | Some('a'..='z') | Some('A'..='Z') | Some('-') | Some('_') | Some('.')
    ) {
        key.push(chars.next().unwrap());
    }

    if key.starts_with('.') || key.ends_with('.') {
        // key cannot start or end with a dot
        Err("invalid key")
    } else {
        Ok(key)
    }
}

fn parse_value(chars: &mut Peekable<Chars>) -> Result<String, &'static str> {
    let mut value = String::new();

    if chars.peek() == Some(&'`') {
        // raw value
        chars.next(); // consume backtick
        loop {
            if chars.peek() == Some(&'`') {
                // check if this is an escaped backtick
                chars.next();
                if chars.peek() == Some(&'`') {
                    // escaped backtick
                    chars.next();
                    value.push('`');
                } else {
                    // end of the value
                    break;
                }
            } else if let Some(c) = chars.next() {
                value.push(c);
            } else {
                // current value must have been a None
                return Err("unterminated raw value");
            }
        }
    } else {
        // normal value: no comment starting character but white space, but not vertical space
        while matches!(chars.peek(), Some(&c) if c != '#' && c != ';' && !( c.is_whitespace() && is_vertical_ws(c) ))
        {
            value.push(chars.next().unwrap());
        }
        value = value.trim().to_string();
    }

    Ok(value)
}

/// Parses CNI format text and returns the resulting key-value store.
///
/// Section names are prepended to the key separated by a dot.
///
/// For more information see the [crate level documentation](index.html).
pub fn parse(text: &str) -> Result<HashMap<String, String>, &'static str> {
    let mut chars = text.chars().peekable();
    let mut map = HashMap::new();
    let mut section = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace()
        /* same as Perl's and Raku's \s */
        {
            chars.next(); // consume whitespace
        } else if c == '#' || c == ';' {
            // comment, continue until next vertical whitespace or EOF
            while matches!(chars.next(), Some(c) if !is_vertical_ws(c)) {}
        } else if c == '[' {
            // section heading
            chars.next(); // consume [
            skip_ws(&mut chars);
            section = parse_key(&mut chars)?;
            skip_ws(&mut chars);
            if chars.next() != Some(']') {
                return Err("expected \"]\"");
            }
        } else {
            // this should be a key
            let mut key = parse_key(&mut chars)?;
            // this key cannot be empty
            if key.is_empty() {
                return Err("expected key");
            }

            if !section.is_empty() {
                // prepend section name
                key = format!("{}.{}", section, key);
            }

            skip_ws(&mut chars);
            if chars.next() != Some('=') {
                return Err("expected \"=\"");
            }
            skip_ws(&mut chars);

            let value = parse_value(&mut chars)?;

            map.insert(key, value);
        }
    }

    Ok(map)
}
