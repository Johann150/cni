#[cfg(test)]
mod test;

use crate::error::{Error, Kind, Result};
use cni_format::{CniExt, CniParser};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
enum Tree {
    Map(HashMap<String, Tree>),
    /// stringized value and starting position as line, column (counting from 1)
    Value(String, usize, usize),
}

#[derive(Debug)]
pub struct Deserializer {
    keys: Vec<String>,
    vals: Vec<Tree>,
    end: Option<(usize, usize)>,
}

impl Deserializer {
    fn new(map: HashMap<String, Tree>) -> Self {
        let end = map
            .values()
            .filter_map(|v| {
                if let Tree::Value(_, line, col) = v {
                    Some((*line, *col))
                } else {
                    None
                }
            })
            .max();
        let (keys, vals): (Vec<_>, Vec<_>) = map.into_iter().unzip();

        Self { keys, vals, end }
    }

    fn next(&mut self) -> Result<(String, usize, usize)> {
        if let Some(Tree::Value(value, line, col)) = self.vals.pop() {
            Ok((value, line, col))
        } else {
            Err(Error {
                line: self.end.map_or(0, |x| x.0),
                col: self.end.map_or(0, |x| x.1),
                kind: Kind::ExpectedValues,
            })
        }
    }
}

// actual deserialisation logic
mod r#impl;

pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: Deserialize<'de>,
{
    use std::collections::hash_map::Entry;

    let mut parser: CniParser<std::str::Chars<'de>> = s.into();
    let mut data = HashMap::new();

    while let Some(result) = parser.next() {
        let (key, val) = result?;
        // can unwrap here because the parser must have returned a Ok result
        let (line, col) = parser.last_pos().unwrap();
        let val = (val, line, col);

        // the format itself allows this, but handle duplicate keys as an error
        // because it might have unintended consequences
        match data.entry(key) {
            Entry::Vacant(e) => e.insert(val),
            Entry::Occupied(e) => {
                return Err(Error {
                    line,
                    col,
                    kind: Kind::DuplicateKey(e.remove_entry().0),
                })
            }
        };
    }

    // the whole file is a struct/map so to represent that
    // put the whole tree into a tree with an empty key
    let mut obj = HashMap::new();
    obj.insert(String::new(), to_tree(data));
    T::deserialize(&mut Deserializer::new(obj))
}

fn to_tree(data: HashMap<String, (String, usize, usize)>) -> Tree {
    let mut map = data
        .sub_leaves("")
        .into_iter()
        .map(|(key, (val, line, col))| (key, Tree::Value(val, line, col)))
        .collect::<HashMap<_, _>>();
    map.extend(data.section_leaves("").into_iter().map(|sect| {
        let tree = to_tree(data.sub_tree(&sect));
        (sect, tree)
    }));

    Tree::Map(map)
}
