use std::collections::HashMap;
use std::str::Chars;

#[cfg(test)]
mod tests;

fn is_vertical_ws(c: char) -> bool {
	// Perl's / Raku's "\v"
	matches!(c, '\n' | '\u{B}' | '\u{C}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

fn skip_ws(chars: &mut Chars) {
	let mut p = chars.peekable();
	while matches!(p.peek(), Some(c) if c.is_whitespace()) {
		p.next();
	}
}

fn parse_key(chars: &mut Chars) -> Result<String, &'static str> {
	// because chars is a &mut, the iterator is not moved and can still be used
	// after this call
	let mut p = chars.peekable();
	let mut key = String::new();

	while matches!(p.peek(), Some('0'..='9')|Some('a'..='z')|Some('A'..='Z')|Some('.')) {
		key.push(p.next().unwrap());
	}

	if key.starts_with('.') || key.ends_with('.') {
		// key cannot start or end with a dot
		Err("invalid key")
	} else if key.is_empty() {
		Err("expected key")
	} else {
		Ok(key)
	}
}

fn parse_value(chars: &mut Chars) -> Result<String, &'static str> {
	let mut p = chars.peekable();
	let mut value = String::new();

	while matches!(p.peek(), Some(c) if c.is_alphanumeric()||(c.is_whitespace()&&!is_vertical_ws(*c))) {
		value.push(p.next().unwrap());
	}

	Ok(value.trim().to_string())
}

pub fn parse(text: &str) -> Result<HashMap<String, String>, &'static str> {
	let mut chars = text.chars();
	let mut map = HashMap::new();
	let mut section = String::new();

	while let Some(c) = chars.next() {
		if c.is_whitespace() /* same as Perl's and Raku's \s */ {
			continue;
		} else if c == '#' || c == ';' {
			// comment, continue until next vertical whitespace or EOF
			while matches!(chars.next(), Some(c) if !is_vertical_ws(c)) {}
		} else if c == '[' {
			skip_ws(&mut chars);
			section = parse_key(&mut chars)?;
			skip_ws(&mut chars);
			if chars.next() != Some(']') {
				return Err("expected \"]\"")
			}
		} else {
			// this should be a key
			let key = parse_key(&mut chars)?;
			// prepend section name
			let key = section.chars().chain(key.chars()).collect();

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
