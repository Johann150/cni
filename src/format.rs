use std::cmp::Ordering;

fn format_value(value: String) -> String {
    if value.contains(|c| c == '`' || crate::is_vertical_ws(c) || crate::is_comment(c)) {
        // This has to be stored as a raw value.
        format!("`{}`", value.replace("`", "``"))
    } else {
        // normal value
        value
    }
}

/// Turn a key/value store into CNI format text. Accepts a wide range of keys,
/// values and map types.
/// The output will contain as few section headers as possible, but if a key
/// consists of multiple parts separated by a dot, the first one will always be
/// used for the section name
///
/// ```
/// let mut map = std::collections::HashMap::new();
/// map.insert("a", "b");
///
/// assert_eq!(
///     &cni_format::to_str(map),
///     "a = b\n"
/// );
///
/// assert_eq!(
///     &cni_format::to_str(vec![
///         ("a.b", "c"),
///     ]),
///     "[a]\nb = c\n"
/// );
/// ```
pub fn to_str<I, K, V>(data: I) -> String
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: ToString,
{
    let mut data = data
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect::<Vec<_>>();

    // use special ordering to ensure fewest number of section headings:
    // - sort by keys
    // - keys without dots first
    // - then sort alphabetically grouped in (sub)sections
    data.sort_unstable_by(|(a, _), (b, _)| {
        a.as_ref()
            .contains('.')
            .cmp(&b.as_ref().contains('.'))
            .then(
                a.as_ref()
                    .split('.')
                    .zip(b.as_ref().split('.'))
                    .fold(Ordering::Equal, |acc, (a, b)| acc.then(a.cmp(b))),
            )
    });

    let mut section = String::new();
    let mut buf = String::new();

    for (key, value) in data {
        let key = key.as_ref();

        let key = if let Some(pos) = key.find('.') {
            let (new_section, key) = key.split_at(pos);
            if section != new_section {
                buf.push_str(&format!("[{}]\n", new_section));
                section = new_section.to_string();
            }
            &key[1..] // remove dot
        } else {
            key
        };
        buf.push_str(&format!("{} = {}\n", key, format_value(value)));
    }

    buf
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn section() {
        let mut map = HashMap::new();
        map.insert("a.b", "c");

        assert_eq!(&crate::to_str(map), "[a]\nb = c\n");
    }

    #[test]
    fn multi_section() {
        let mut map = HashMap::new();
        map.insert("a.b.c".to_string(), "d".to_string());

        assert_eq!(&crate::to_str(map), "[a]\nb.c = d\n");
    }

    #[test]
    fn section_nonalphabetical() {
        let mut map = BTreeMap::new();
        map.insert("a.b", "with section header");
        map.insert("ccc", "without section header");

        assert_eq!(
            &crate::to_str(map),
            "ccc = without section header\n[a]\nb = with section header\n"
        );
    }

    #[test]
    fn multi_value() {
        assert_eq!(
            &crate::to_str(vec![("a", "b"), ("c", "d"),]),
            "a = b\nc = d\n"
        );
    }

    #[test]
    fn value_backtick() {
        let mut map = BTreeMap::new();
        map.insert("a", "backtick`d");

        assert_eq!(&crate::to_str(map), "a = `backtick``d`\n");
    }

    #[test]
    fn value_vertical_whitespace() {
        assert_eq!(
            &crate::to_str(vec![("a", "multi\nline")]),
            "a = `multi\nline`\n"
        );

        assert_eq!(
            &crate::to_str(vec![("a", "multi\r\nline")]),
            "a = `multi\r\nline`\n"
        );

        assert_eq!(
            &crate::to_str(vec![("a", "multi\u{b}line")]),
            "a = `multi\u{b}line`\n"
        );
    }

    #[test]
    fn value_comment_symbol() {
        assert_eq!(
            &crate::to_str(vec![("a", "sharp#sign")]),
            "a = `sharp#sign`\n"
        );

        assert_eq!(
            &crate::to_str(vec![("a", "semi;colon")]),
            "a = `semi;colon`\n"
        );
    }
}
