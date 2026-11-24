//! [`DataValue`], the value a plugin works with.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// A generic data item: what a plugin reads from and returns to Drift.
///
/// WIT cannot express a recursive type, so on the wire a value is the flattened
/// [`data-value`](crate::bindings::drift::plugins::data_values::DataValue) record. Convert with
/// [`DataValue::pack`] and [`DataValue::unpack`], or `TryFrom`.
///
/// Map keys are not ordered. Packing sorts them, so the same value always packs to the same bytes.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum DataValue {
    /// No value.
    #[default]
    Null,
    /// Keys to values.
    Map(HashMap<String, DataValue>),
    /// An ordered list of values.
    List(Vec<DataValue>),
    /// Text.
    String(String),
    /// A signed integer.
    Integer(i64),
    /// An unsigned integer.
    UInteger(u64),
    /// A double precision floating point number.
    Double(f64),
    /// A boolean.
    Boolean(bool),
    /// Raw bytes.
    Bytes(Vec<u8>),
    /// An error, carried as a value.
    Error(String),
}

impl DataValue {
    /// The value for `key`, if this is a map that has it.
    pub fn get(&self, key: &str) -> Option<&DataValue> {
        match self {
            DataValue::Map(map) => map.get(key),
            _ => None,
        }
    }

    /// The map, if this is one.
    pub fn as_map(&self) -> Option<&HashMap<String, DataValue>> {
        match self {
            DataValue::Map(map) => Some(map),
            _ => None,
        }
    }

    /// The map, mutably, if this is one.
    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<String, DataValue>> {
        match self {
            DataValue::Map(map) => Some(map),
            _ => None,
        }
    }

    /// The list, if this is one.
    pub fn as_list(&self) -> Option<&Vec<DataValue>> {
        match self {
            DataValue::List(list) => Some(list),
            _ => None,
        }
    }

    /// The list, mutably, if this is one.
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<DataValue>> {
        match self {
            DataValue::List(list) => Some(list),
            _ => None,
        }
    }

    /// The text, if this is a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            DataValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// The number, if this is a signed integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            DataValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// The number, if this is an unsigned integer.
    pub fn as_uinteger(&self) -> Option<u64> {
        match self {
            DataValue::UInteger(i) => Some(*i),
            _ => None,
        }
    }

    /// The number, if this is a double.
    pub fn as_double(&self) -> Option<f64> {
        match self {
            DataValue::Double(d) => Some(*d),
            _ => None,
        }
    }

    /// The boolean, if this is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            DataValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// The bytes, if this is a byte array.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            DataValue::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// If this is [`DataValue::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, DataValue::Null)
    }

    /// If this is [`DataValue::Error`].
    pub fn is_error(&self) -> bool {
        matches!(self, DataValue::Error(_))
    }

    /// A name for the kind of value this is, for messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            DataValue::Null => "Null",
            DataValue::Map(_) => "Map",
            DataValue::List(_) => "List",
            DataValue::String(_) => "String",
            DataValue::Integer(_) => "Integer",
            DataValue::UInteger(_) => "Unsigned-Integer",
            DataValue::Double(_) => "Double",
            DataValue::Boolean(_) => "Boolean",
            DataValue::Bytes(_) => "Bytes",
            DataValue::Error(_) => "Error",
        }
    }
}

/// Text for people. Maps are written in key order, bytes as lossy UTF-8, and null as nothing.
impl Display for DataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataValue::Null => Ok(()),
            DataValue::Map(map) => {
                let mut entries: Vec<_> = map.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));
                write!(f, "{{")?;
                for (index, (k, v)) in entries.into_iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            DataValue::List(list) => {
                write!(f, "[")?;
                for (index, v) in list.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            DataValue::String(s) | DataValue::Error(s) => write!(f, "{s}"),
            DataValue::Integer(i) => write!(f, "{i}"),
            DataValue::UInteger(i) => write!(f, "{i}"),
            DataValue::Double(d) => write!(f, "{d}"),
            DataValue::Boolean(b) => write!(f, "{b}"),
            DataValue::Bytes(b) => write!(f, "{}", String::from_utf8_lossy(b)),
        }
    }
}

impl From<String> for DataValue {
    fn from(value: String) -> Self {
        DataValue::String(value)
    }
}

impl From<&String> for DataValue {
    fn from(value: &String) -> Self {
        DataValue::String(value.clone())
    }
}

impl From<&str> for DataValue {
    fn from(value: &str) -> Self {
        DataValue::String(value.to_string())
    }
}

impl From<Cow<'_, str>> for DataValue {
    fn from(value: Cow<'_, str>) -> Self {
        DataValue::String(value.into_owned())
    }
}

impl From<i64> for DataValue {
    fn from(value: i64) -> Self {
        DataValue::Integer(value)
    }
}

impl From<i32> for DataValue {
    fn from(value: i32) -> Self {
        DataValue::Integer(value.into())
    }
}

impl From<i16> for DataValue {
    fn from(value: i16) -> Self {
        DataValue::Integer(value.into())
    }
}

impl From<u64> for DataValue {
    fn from(value: u64) -> Self {
        DataValue::UInteger(value)
    }
}

impl From<u32> for DataValue {
    fn from(value: u32) -> Self {
        DataValue::UInteger(value.into())
    }
}

impl From<u16> for DataValue {
    fn from(value: u16) -> Self {
        DataValue::UInteger(value.into())
    }
}

impl From<usize> for DataValue {
    fn from(value: usize) -> Self {
        DataValue::UInteger(value as u64)
    }
}

impl From<f64> for DataValue {
    fn from(value: f64) -> Self {
        DataValue::Double(value)
    }
}

impl From<bool> for DataValue {
    fn from(value: bool) -> Self {
        DataValue::Boolean(value)
    }
}

impl From<&[u8]> for DataValue {
    fn from(value: &[u8]) -> Self {
        DataValue::Bytes(value.to_vec())
    }
}

impl<T: Into<DataValue>> From<Option<T>> for DataValue {
    fn from(value: Option<T>) -> Self {
        value.map_or(DataValue::Null, Into::into)
    }
}

impl<T: Into<DataValue>> From<Vec<T>> for DataValue {
    fn from(value: Vec<T>) -> Self {
        DataValue::List(value.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<DataValue>> From<HashMap<String, T>> for DataValue {
    fn from(value: HashMap<String, T>) -> Self {
        DataValue::Map(value.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_match_the_variant_only() {
        let map = DataValue::Map(HashMap::from([("a".to_string(), DataValue::Integer(1))]));
        assert_eq!(map.get("a"), Some(&DataValue::Integer(1)));
        assert_eq!(map.get("b"), None);
        assert_eq!(DataValue::Integer(1).get("a"), None);

        assert_eq!(DataValue::Integer(1).as_integer(), Some(1));
        assert_eq!(DataValue::Integer(1).as_uinteger(), None);
        assert_eq!(DataValue::UInteger(1).as_uinteger(), Some(1));
        assert_eq!(DataValue::from("x").as_string(), Some("x"));
        assert_eq!(DataValue::Bytes(vec![1]).as_bytes(), Some(&[1u8][..]));
        assert!(DataValue::default().is_null());
        assert!(DataValue::Error("boom".into()).is_error());
    }

    #[test]
    fn conversions_pick_signed_or_unsigned_by_type() {
        assert_eq!(DataValue::from(-1i32), DataValue::Integer(-1));
        assert_eq!(DataValue::from(1u32), DataValue::UInteger(1));
        assert_eq!(DataValue::from(1usize), DataValue::UInteger(1));
        assert_eq!(DataValue::from(None::<String>), DataValue::Null);
        assert_eq!(
            DataValue::from(vec!["a", "b"]),
            DataValue::List(vec![DataValue::from("a"), DataValue::from("b")])
        );
    }

    #[test]
    fn display_orders_map_keys() {
        let value = DataValue::from(HashMap::from([
            ("b".to_string(), 2i64),
            ("a".to_string(), 1i64),
        ]));
        assert_eq!(value.to_string(), "{a: 1, b: 2}");
        assert_eq!(DataValue::from(vec![1i64, 2]).to_string(), "[1, 2]");
    }
}
