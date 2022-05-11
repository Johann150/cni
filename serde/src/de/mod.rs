#[cfg(test)]
mod test;

use crate::error::{Error, Kind, Result};
use cni_format::{CniExt, CniParser};
use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    forward_to_deserialize_any, Deserialize,
};
use std::collections::HashMap;
use std::str::Chars;

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
}

macro_rules! deserialize {
    ($deser:ident, $visit:ident, $err:ident) => {
        fn $deser<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let (value, line, col) = self.next()?;
            match value.parse() {
                Ok(int) => visitor.$visit(int),
                Err(err) => Err(Error {
                    line,
                    col,
                    kind: Kind::$err(err),
                }),
            }
        }
    };
}

impl Deserializer {
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

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    forward_to_deserialize_any! { string str tuple tuple_struct map struct seq enum }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.vals.pop() {
            Some(Tree::Map(map)) => visitor.visit_map(&mut Deserializer::new(map)),
            Some(Tree::Value(val, ..)) => visitor.visit_string(val),
            None => Err(Error {
                line: self.end.map_or(0, |x| x.0),
                col: self.end.map_or(0, |x| x.1),
                kind: Kind::ExpectedValues,
            }),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Some(Tree::Value(..)) = self.vals.pop() {
            visitor.visit_unit()
        } else {
            Err(Error {
                line: self.end.map_or(0, |x| x.0),
                col: self.end.map_or(0, |x| x.1),
                kind: Kind::ExpectedValues,
            })
        }
    }

    deserialize!(deserialize_i8, visit_i8, Int);
    deserialize!(deserialize_i16, visit_i16, Int);
    deserialize!(deserialize_i32, visit_i32, Int);
    deserialize!(deserialize_i64, visit_i64, Int);
    deserialize!(deserialize_u8, visit_u8, Int);
    deserialize!(deserialize_u16, visit_u16, Int);
    deserialize!(deserialize_u32, visit_u32, Int);
    deserialize!(deserialize_u64, visit_u64, Int);
    deserialize!(deserialize_f32, visit_f32, Float);
    deserialize!(deserialize_f64, visit_f64, Float);

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (mut val, line, col) = self.next()?;

        val.make_ascii_lowercase();
        match val.as_str() {
            "1" | "+" | "true" | "yes" | "on" | "up" => visitor.visit_bool(true),
            "0" | "-" | "false" | "no" | "off" | "down" => visitor.visit_bool(false),
            _ => Err(Error {
                line,
                col,
                kind: Kind::Bool,
            }),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Some(Tree::Value(value, line, col)) = self.vals.pop() {
            if value.chars().count() == 1 {
                visitor.visit_char(value.chars().next().unwrap())
            } else {
                Err(Error {
                    line,
                    col,
                    kind: Kind::Char,
                })
            }
        } else {
            Err(Error {
                line: self.end.map_or(0, |x| x.0),
                col: self.end.map_or(0, |x| x.1),
                kind: Kind::ExpectedValues,
            })
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.next()?.0.into())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.next()?.0.into())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next()?.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (val, line, col) = self.next()?;
        if val.is_empty() {
            visitor.visit_unit()
        } else {
            Err(Error {
                line,
                col,
                kind: Kind::Unit,
            })
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // using pop means the elements will be iterated from back to front
        // but since they came from a hashmap with indeterminate order it
        // does not matter anyway
        if let Some(key) = self.keys.pop() {
            visitor.visit_string(key)
        } else {
            Err(Error {
                line: self.end.map_or(0, |x| x.0),
                col: self.end.map_or(0, |x| x.1),
                kind: Kind::ExpectedValues,
            })
        }
    }
}

impl<'de> MapAccess<'de> for Deserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // have to force the deserialize to deserialize an identifier
        use serde::{de::IntoDeserializer, Deserializer};

        struct KeyVisitor;

        impl Visitor<'_> for KeyVisitor {
            type Value = String;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a key")
            }

            fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(s.to_owned())
            }
        }

        let ident = self
            .deserialize_identifier(KeyVisitor)
            .and_then(|ident| seed.deserialize(ident.into_deserializer()));
        match ident {
            Ok(x) => Ok(Some(x)),
            Err(Error {
                kind: Kind::ExpectedValues,
                ..
            }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
}

pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut parser: CniParser<Chars<'de>> = s.into();
    let mut data = HashMap::new();

    while let Some(result) = parser.next() {
        let result = result?;
        // can unwrap here because the parser must have returned a Ok result
        let pos = parser.last_pos().unwrap();
        let val = (result.1, pos.0, pos.1);

        // the format itself allows this, but handle duplicate keys as an error
        // because it might have unintended consequences
        if data.contains_key(&result.0) {
            return Err(Error {
                line: pos.0,
                col: pos.1,
                kind: Kind::DuplicateKey(result.0),
            });
        } else {
            data.insert(result.0, val);
        }
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
        .map(|(key, (val, line, col))| (key.to_string(), Tree::Value(val, line, col)))
        .collect::<HashMap<_, _>>();
    map.extend(data.section_leaves("").into_iter().map(|sect| {
        let tree = to_tree(data.sub_tree(&sect));
        (sect, tree)
    }));

    Tree::Map(map)
}
