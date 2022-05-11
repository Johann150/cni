use super::{Deserializer, Tree};
use crate::error::{Error, Kind, Result};
use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    forward_to_deserialize_any,
};

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
