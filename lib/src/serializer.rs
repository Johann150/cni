use std::cmp::Ordering;

fn format_value(value: String) -> String {
    if value.is_empty() {
        "#empty".to_string()
    } else if value.contains(|c| c == '`' || crate::is_vertical_ws(c) || c == '#' || c == ';') {
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
/// ```ignore <https://github.com/rust-lang/rust/issues/67295>
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
