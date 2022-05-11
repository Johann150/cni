use crate::error::{Error, Kind};
use serde::{
    de::{Deserializer, Visitor},
    forward_to_deserialize_any,
};

/// A newtype around String to implement Deserialize on.
/// Also keeps track of the starting location of the value.
#[derive(Clone, Debug)]
pub(super) struct Value(pub String, pub usize, pub usize);

macro_rules! deserialize {
    ($deser:ident, $visit:ident, $err:ident) => {
        fn $deser<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self.0.parse() {
                Ok(int) => visitor.$visit(int),
                Err(err) => Err(Error {
                    line: self.1,
                    col: self.2,
                    kind: Kind::$err(err),
                }),
            }
        }
    };
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

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

    forward_to_deserialize_any! { string str seq tuple tuple_struct map struct enum identifier }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.make_ascii_lowercase();
        match self.0.as_str() {
            "1" | "+" | "true" | "yes" | "on" | "up" => visitor.visit_bool(true),
            "0" | "-" | "false" | "no" | "off" | "down" => visitor.visit_bool(false),
            _ => Err(Error {
                line: self.1,
                col: self.2,
                kind: Kind::Bool,
            }),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.0.chars().count() == 1 {
            visitor.visit_char(self.0.chars().next().unwrap())
        } else {
            Err(Error {
                line: self.1,
                col: self.2,
                kind: Kind::Char,
            })
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.0.into())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.0.into())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.0.is_empty() {
            visitor.visit_unit()
        } else {
            Err(Error {
                line: self.1,
                col: self.2,
                kind: Kind::Unit,
            })
        }
    }
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }
}
