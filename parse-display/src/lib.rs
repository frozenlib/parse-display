use serde::de::Deserializer;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;

pub use parse_display_derive::Display;

pub struct ParseEnumError;
impl Debug for ParseEnumError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "cannot parse enum")
    }
}
impl Display for ParseEnumError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "cannot parse enum")
    }
}
pub fn deserialize_from_str<'de, D, T>(d: D) -> Result<T, D::Error>
where
    T: FromStr,
    D: Deserializer<'de>,
{
    use serde::de::*;
    struct StrVisitor<T> {
        _phantom: PhantomData<fn() -> T>,
    }
    impl<'de, T: FromStr> Visitor<'de> for StrVisitor<T> {
        type Value = T;
        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            write!(formatter, "string")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if let Ok(value) = FromStr::from_str(v) {
                Ok(value)
            } else {
                Err(Error::invalid_value(Unexpected::Str(v), &self))
            }
        }
    }
    d.deserialize_str(StrVisitor {
        _phantom: PhantomData::default(),
    })
}
