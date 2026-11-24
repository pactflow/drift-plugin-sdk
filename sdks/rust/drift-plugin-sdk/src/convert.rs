//! Converting between [`DataValue`] and the WIT `data-value` record.
//!
//! WIT has no recursive types, so a value crosses the boundary flattened. The record holds the root
//! value and a list of nodes. A list or map refers to its children by their position in that list,
//! as a `u16`. The format is:
//!
//! * A scalar is the root value, and there are no nodes.
//! * A list or map is the root value, with its children in the nodes. Nodes are laid out in
//!   pre-order: a node comes first, then everything beneath it, then its next sibling. Packing an
//!   empty list or map gives an empty node list, not none.
//! * Map entries are in key order, compared as UTF-8 bytes.
//! * Lists and maps nest at most [`MAX_DEPTH`] deep, so that converting a value cannot exhaust the
//!   stack.
//!
//! The `conformance/data-values` vectors in the repository pin this down, and every SDK runs them.

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::bindings::drift::plugins::data_values::{
    DataValue as WitDataValue, DataValueInner, StringValue,
};
use crate::value::DataValue;

/// The most lists and maps a value can nest, counting the root as one.
///
/// It is the recursion limit `serde_json` uses, so a value parsed from JSON is never over it. A value
/// that is deeper cannot be packed, and a `data-value` that is deeper cannot be unpacked.
pub const MAX_DEPTH: usize = 128;

/// Why a value could not be packed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackError {
    /// Nodes are addressed by a `u16`, so a value can have at most 65,536 nodes beneath its root.
    TooManyNodes,
    /// The value nests lists and maps more than [`MAX_DEPTH`] deep.
    TooDeep,
}

impl Display for PackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PackError::TooManyNodes => {
                write!(f, "the value has more than 65,536 nodes below its root")
            }
            PackError::TooDeep => {
                write!(
                    f,
                    "the value nests lists and maps more than {MAX_DEPTH} deep"
                )
            }
        }
    }
}

impl std::error::Error for PackError {}

/// Why a `data-value` could not be unpacked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnpackError {
    /// An index is past the end of the node list, or there is no node list.
    IndexOutOfRange {
        /// The index that was used.
        index: u16,
        /// How many nodes there are.
        nodes: usize,
    },
    /// A node refers to itself or to a node before it. A packed value never does, and following such
    /// an index could loop for ever.
    IndexNotForward {
        /// The index that was used.
        index: u16,
        /// The node that used it.
        parent: u16,
    },
    /// Lists and maps nest more than [`MAX_DEPTH`] deep.
    TooDeep,
}

impl Display for UnpackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnpackError::IndexOutOfRange { index, nodes } => {
                write!(
                    f,
                    "node index {index} is out of range: there are {nodes} nodes"
                )
            }
            UnpackError::IndexNotForward { index, parent } => {
                write!(
                    f,
                    "node {parent} refers to node {index}, which is not after it"
                )
            }
            UnpackError::TooDeep => {
                write!(f, "lists and maps nest more than {MAX_DEPTH} deep")
            }
        }
    }
}

impl std::error::Error for UnpackError {}

impl DataValue {
    /// Flattens this value into the form that crosses the plugin boundary.
    ///
    /// Fails with [`PackError::TooManyNodes`] when there are too many nodes to address, and with
    /// [`PackError::TooDeep`] when lists and maps nest more than [`MAX_DEPTH`] deep.
    pub fn pack(&self) -> Result<WitDataValue, PackError> {
        let mut nodes = Vec::new();
        let value = match self {
            DataValue::Null => DataValueInner::Null,
            DataValue::String(s) => DataValueInner::Str(s.into()),
            DataValue::Integer(i) => DataValueInner::Int(*i),
            DataValue::UInteger(i) => DataValueInner::Uint(*i),
            DataValue::Double(f) => DataValueInner::Double(*f),
            DataValue::Boolean(b) => DataValueInner::Boolean(*b),
            DataValue::Bytes(b) => DataValueInner::Bytes(b.clone()),
            DataValue::Error(e) => DataValueInner::Error(e.into()),
            DataValue::List(_) | DataValue::Map(_) => {
                let inner = pack_children(self, &mut nodes, 1)?;
                return Ok(WitDataValue {
                    value: inner,
                    nodes: Some(nodes),
                });
            }
        };
        Ok(WitDataValue { value, nodes: None })
    }

    /// Rebuilds a value from the form that crosses the plugin boundary.
    ///
    /// Text that is not valid in its encoding is decoded lossily. Lists and maps may nest at most
    /// [`MAX_DEPTH`] deep. See [`UnpackError`] for what is rejected.
    pub fn unpack(value: &WitDataValue) -> Result<DataValue, UnpackError> {
        let nodes = value.nodes.as_deref().unwrap_or(&[]);
        unpack_inner(&value.value, nodes, None, 0)
    }
}

/// Packs a list or map, whose children go into `nodes`. Returns the node that describes it. `depth`
/// is how many lists and maps it is inside, counting itself.
fn pack_children(
    value: &DataValue,
    nodes: &mut Vec<DataValueInner>,
    depth: usize,
) -> Result<DataValueInner, PackError> {
    if depth > MAX_DEPTH {
        return Err(PackError::TooDeep);
    }
    match value {
        DataValue::List(list) => {
            let mut indices = Vec::with_capacity(list.len());
            for child in list {
                indices.push(pack_node(child, nodes, depth)?);
            }
            Ok(DataValueInner::List(indices))
        }
        DataValue::Map(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut entries = Vec::with_capacity(keys.len());
            for key in keys {
                entries.push((key.into(), pack_node(&map[key], nodes, depth)?));
            }
            Ok(DataValueInner::Map(entries))
        }
        _ => unreachable!("only a list or a map has children"),
    }
}

/// Adds `value`, then everything beneath it, to `nodes`. Returns the index it was given. `depth` is
/// how many lists and maps contain it.
fn pack_node(
    value: &DataValue,
    nodes: &mut Vec<DataValueInner>,
    depth: usize,
) -> Result<u16, PackError> {
    let index = u16::try_from(nodes.len()).map_err(|_| PackError::TooManyNodes)?;
    // Reserve the slot first so the node precedes its children.
    nodes.push(DataValueInner::Null);
    let inner = match value {
        DataValue::Null => DataValueInner::Null,
        DataValue::String(s) => DataValueInner::Str(s.into()),
        DataValue::Integer(i) => DataValueInner::Int(*i),
        DataValue::UInteger(i) => DataValueInner::Uint(*i),
        DataValue::Double(f) => DataValueInner::Double(*f),
        DataValue::Boolean(b) => DataValueInner::Boolean(*b),
        DataValue::Bytes(b) => DataValueInner::Bytes(b.clone()),
        DataValue::Error(e) => DataValueInner::Error(e.into()),
        DataValue::List(_) | DataValue::Map(_) => pack_children(value, nodes, depth + 1)?,
    };
    nodes[index as usize] = inner;
    Ok(index)
}

/// Unpacks `inner`. `this` is the index of the node it came from, or none for the root. `depth` is
/// how many lists and maps contain it.
fn unpack_inner(
    inner: &DataValueInner,
    nodes: &[DataValueInner],
    this: Option<u16>,
    depth: usize,
) -> Result<DataValue, UnpackError> {
    // What a list or map is inside, which is one more than what contains it.
    let inside = depth + 1;
    if inside > MAX_DEPTH && matches!(inner, DataValueInner::List(_) | DataValueInner::Map(_)) {
        return Err(UnpackError::TooDeep);
    }
    let child = |index: u16| -> Result<DataValue, UnpackError> {
        if let Some(parent) = this {
            if index <= parent {
                return Err(UnpackError::IndexNotForward { index, parent });
            }
        }
        let node = nodes
            .get(index as usize)
            .ok_or(UnpackError::IndexOutOfRange {
                index,
                nodes: nodes.len(),
            })?;
        unpack_inner(node, nodes, Some(index), inside)
    };

    Ok(match inner {
        DataValueInner::Null => DataValue::Null,
        DataValueInner::Str(s) => DataValue::String(s.into()),
        DataValueInner::Int(i) => DataValue::Integer(*i),
        DataValueInner::Uint(i) => DataValue::UInteger(*i),
        DataValueInner::Double(f) => DataValue::Double(*f),
        DataValueInner::Boolean(b) => DataValue::Boolean(*b),
        DataValueInner::Bytes(b) => DataValue::Bytes(b.clone()),
        DataValueInner::Error(e) => DataValue::Error(e.into()),
        DataValueInner::List(indices) => DataValue::List(
            indices
                .iter()
                .map(|i| child(*i))
                .collect::<Result<_, _>>()?,
        ),
        DataValueInner::Map(entries) => {
            let mut map = HashMap::with_capacity(entries.len());
            for (key, index) in entries {
                map.insert(key.into(), child(*index)?);
            }
            DataValue::Map(map)
        }
    })
}

impl TryFrom<&DataValue> for WitDataValue {
    type Error = PackError;

    fn try_from(value: &DataValue) -> Result<Self, PackError> {
        value.pack()
    }
}

impl TryFrom<DataValue> for WitDataValue {
    type Error = PackError;

    fn try_from(value: DataValue) -> Result<Self, PackError> {
        value.pack()
    }
}

impl TryFrom<&WitDataValue> for DataValue {
    type Error = UnpackError;

    fn try_from(value: &WitDataValue) -> Result<Self, UnpackError> {
        DataValue::unpack(value)
    }
}

impl TryFrom<WitDataValue> for DataValue {
    type Error = UnpackError;

    fn try_from(value: WitDataValue) -> Result<Self, UnpackError> {
        DataValue::unpack(&value)
    }
}

impl From<&str> for WitDataValue {
    fn from(value: &str) -> Self {
        WitDataValue {
            value: DataValueInner::Str(value.into()),
            nodes: None,
        }
    }
}

impl From<String> for WitDataValue {
    fn from(value: String) -> Self {
        WitDataValue {
            value: DataValueInner::Str(value.into()),
            nodes: None,
        }
    }
}

impl From<&String> for WitDataValue {
    fn from(value: &String) -> Self {
        WitDataValue {
            value: DataValueInner::Str(value.into()),
            nodes: None,
        }
    }
}

impl From<&str> for StringValue {
    fn from(value: &str) -> Self {
        StringValue {
            encoding: None,
            data: value.as_bytes().to_vec(),
        }
    }
}

impl From<String> for StringValue {
    fn from(value: String) -> Self {
        StringValue {
            encoding: None,
            data: value.into_bytes(),
        }
    }
}

impl From<&String> for StringValue {
    fn from(value: &String) -> Self {
        value.as_str().into()
    }
}

/// Decodes the text. The encoding is UTF-8 when none is given, and UTF-16LE and UTF-16BE are
/// understood. Invalid input, and text in any other encoding, is decoded as UTF-8 with the invalid
/// parts replaced.
impl From<&StringValue> for String {
    fn from(value: &StringValue) -> Self {
        let encoding = value.encoding.as_deref().unwrap_or("UTF-8");
        if encoding.eq_ignore_ascii_case("UTF-16LE") {
            decode_utf16(&value.data, u16::from_le_bytes)
        } else if encoding.eq_ignore_ascii_case("UTF-16BE") {
            decode_utf16(&value.data, u16::from_be_bytes)
        } else {
            String::from_utf8_lossy(&value.data).into_owned()
        }
    }
}

impl From<StringValue> for String {
    fn from(value: StringValue) -> Self {
        (&value).into()
    }
}

fn decode_utf16(data: &[u8], to_unit: fn([u8; 2]) -> u16) -> String {
    let (pairs, remainder) = data.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().map(|pair| to_unit(*pair)).collect();
    let mut text = String::from_utf16_lossy(&units);
    if !remainder.is_empty() {
        text.push(char::REPLACEMENT_CHARACTER);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(i: i64) -> DataValue {
        DataValue::Integer(i)
    }

    fn map<const N: usize>(entries: [(&str, DataValue); N]) -> DataValue {
        DataValue::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn wit(value: DataValueInner, nodes: Option<Vec<DataValueInner>>) -> WitDataValue {
        WitDataValue { value, nodes }
    }

    #[test]
    fn scalars_have_no_nodes() {
        assert_eq!(
            DataValue::Null.pack().unwrap(),
            wit(DataValueInner::Null, None)
        );
        assert_eq!(int(-1).pack().unwrap(), wit(DataValueInner::Int(-1), None));
        assert_eq!(
            DataValue::UInteger(1).pack().unwrap(),
            wit(DataValueInner::Uint(1), None)
        );
        assert_eq!(
            DataValue::Double(1.5).pack().unwrap(),
            wit(DataValueInner::Double(1.5), None)
        );
        assert_eq!(
            DataValue::Boolean(true).pack().unwrap(),
            wit(DataValueInner::Boolean(true), None)
        );
        assert_eq!(
            DataValue::Bytes(vec![1, 2]).pack().unwrap(),
            wit(DataValueInner::Bytes(vec![1, 2]), None)
        );
        assert_eq!(
            DataValue::Error("boom".into()).pack().unwrap(),
            wit(DataValueInner::Error("boom".into()), None)
        );
        assert_eq!(
            DataValue::from("hi").pack().unwrap(),
            wit(DataValueInner::Str("hi".into()), None)
        );
    }

    #[test]
    fn nodes_are_laid_out_in_pre_order() {
        let value = DataValue::List(vec![
            int(100),
            DataValue::UInteger(100),
            DataValue::List(vec![
                int(200),
                DataValue::List(vec![int(300), DataValue::UInteger(300)]),
                int(200),
            ]),
        ]);
        assert_eq!(
            value.pack().unwrap(),
            wit(
                DataValueInner::List(vec![0, 1, 2]),
                Some(vec![
                    DataValueInner::Int(100),
                    DataValueInner::Uint(100),
                    DataValueInner::List(vec![3, 4, 7]),
                    DataValueInner::Int(200),
                    DataValueInner::List(vec![5, 6]),
                    DataValueInner::Int(300),
                    DataValueInner::Uint(300),
                    DataValueInner::Int(200),
                ])
            )
        );
    }

    #[test]
    fn maps_are_packed_in_key_order() {
        let value = map([
            ("c", DataValue::Boolean(true)),
            ("a", int(100)),
            ("b", map([("e", DataValue::UInteger(200)), ("d", int(200))])),
        ]);
        assert_eq!(
            value.pack().unwrap(),
            wit(
                DataValueInner::Map(vec![("a".into(), 0), ("b".into(), 1), ("c".into(), 4)]),
                Some(vec![
                    DataValueInner::Int(100),
                    DataValueInner::Map(vec![("d".into(), 2), ("e".into(), 3)]),
                    DataValueInner::Int(200),
                    DataValueInner::Uint(200),
                    DataValueInner::Boolean(true),
                ])
            )
        );
    }

    #[test]
    fn empty_collections_have_an_empty_node_list() {
        assert_eq!(
            DataValue::List(vec![]).pack().unwrap(),
            wit(DataValueInner::List(vec![]), Some(vec![]))
        );
        assert_eq!(
            map([]).pack().unwrap(),
            wit(DataValueInner::Map(vec![]), Some(vec![]))
        );
    }

    #[test]
    fn unpack_inverts_pack() {
        let value = map([
            (
                "list",
                DataValue::List(vec![int(1), DataValue::Null, DataValue::List(vec![])]),
            ),
            (
                "map",
                map([("inner", DataValue::from("x")), ("empty", map([]))]),
            ),
            ("bytes", DataValue::Bytes(vec![0, 255])),
            ("error", DataValue::Error("boom".into())),
        ]);
        assert_eq!(DataValue::unpack(&value.pack().unwrap()).unwrap(), value);
    }

    #[test]
    fn node_limit_is_what_a_u16_can_address() {
        let full = DataValue::List(vec![DataValue::Null; 65_536]);
        let packed = full.pack().unwrap();
        assert_eq!(packed.nodes.as_ref().map(Vec::len), Some(65_536));
        assert_eq!(DataValue::unpack(&packed).unwrap(), full);

        let too_many = DataValue::List(vec![DataValue::Null; 65_537]);
        assert_eq!(too_many.pack(), Err(PackError::TooManyNodes));

        // Nested nodes count too.
        let nested = DataValue::List(vec![DataValue::List(vec![DataValue::Null; 65_536])]);
        assert_eq!(nested.pack(), Err(PackError::TooManyNodes));
    }

    #[test]
    fn unpack_rejects_indices_it_cannot_follow() {
        let missing = wit(
            DataValueInner::List(vec![1]),
            Some(vec![DataValueInner::Null]),
        );
        assert_eq!(
            DataValue::unpack(&missing),
            Err(UnpackError::IndexOutOfRange { index: 1, nodes: 1 })
        );

        let no_nodes = wit(DataValueInner::Map(vec![("a".into(), 0)]), None);
        assert_eq!(
            DataValue::unpack(&no_nodes),
            Err(UnpackError::IndexOutOfRange { index: 0, nodes: 0 })
        );

        let itself = wit(
            DataValueInner::List(vec![0]),
            Some(vec![DataValueInner::List(vec![0])]),
        );
        assert_eq!(
            DataValue::unpack(&itself),
            Err(UnpackError::IndexNotForward {
                index: 0,
                parent: 0
            })
        );

        let backwards = wit(
            DataValueInner::List(vec![1]),
            Some(vec![DataValueInner::Null, DataValueInner::List(vec![0])]),
        );
        assert_eq!(
            DataValue::unpack(&backwards),
            Err(UnpackError::IndexNotForward {
                index: 0,
                parent: 1
            })
        );
    }

    #[test]
    fn unpack_allows_an_empty_collection_with_no_nodes() {
        assert_eq!(
            DataValue::unpack(&wit(DataValueInner::List(vec![]), None)),
            Ok(DataValue::List(vec![]))
        );
    }

    #[test]
    fn unpack_allows_a_node_to_be_shared() {
        let shared = wit(
            DataValueInner::List(vec![0, 0]),
            Some(vec![DataValueInner::Int(1)]),
        );
        assert_eq!(
            DataValue::unpack(&shared),
            Ok(DataValue::List(vec![int(1), int(1)]))
        );
    }

    #[test]
    fn text_is_decoded_by_its_encoding() {
        let string = |encoding: Option<&str>, data: &[u8]| -> String {
            (&StringValue {
                encoding: encoding.map(str::to_string),
                data: data.to_vec(),
            })
                .into()
        };
        assert_eq!(string(None, "héllo".as_bytes()), "héllo");
        assert_eq!(string(Some("utf-8"), "héllo".as_bytes()), "héllo");
        assert_eq!(string(Some("UTF-16LE"), &[0x68, 0x00, 0xe9, 0x00]), "hé");
        assert_eq!(string(Some("utf-16be"), &[0x00, 0x68, 0x00, 0xe9]), "hé");
        // A surrogate pair.
        assert_eq!(string(Some("UTF-16LE"), &[0x3d, 0xd8, 0x00, 0xde]), "😀");
        // Bad input does not panic.
        assert_eq!(string(Some("UTF-16LE"), &[0x68, 0x00, 0x69]), "h\u{fffd}");
        assert_eq!(string(Some("UTF-16LE"), &[0x00, 0xd8]), "\u{fffd}");
        assert_eq!(string(None, &[0x68, 0xff]), "h\u{fffd}");
        // An encoding we do not know is read as UTF-8.
        assert_eq!(string(Some("latin-1"), b"abc"), "abc");
    }

    /// `depth` lists, each inside the last, with an empty list at the bottom.
    fn nested_lists(depth: usize) -> DataValue {
        (1..depth).fold(DataValue::List(vec![]), |inner, _| {
            DataValue::List(vec![inner])
        })
    }

    #[test]
    fn nesting_is_limited() {
        let at_limit = nested_lists(MAX_DEPTH);
        let packed = at_limit.pack().unwrap();
        assert_eq!(DataValue::unpack(&packed).unwrap(), at_limit);

        assert_eq!(nested_lists(MAX_DEPTH + 1).pack(), Err(PackError::TooDeep));

        // Maps count as nesting too, and so do maps inside lists.
        let mut mixed = DataValue::List(vec![]);
        for level in 1..MAX_DEPTH {
            mixed = if level % 2 == 0 {
                DataValue::List(vec![mixed])
            } else {
                map([("k", mixed)])
            };
        }
        assert!(mixed.pack().is_ok());
        assert_eq!(map([("k", mixed)]).pack(), Err(PackError::TooDeep));
    }

    #[test]
    fn unpack_stops_at_the_nesting_limit() {
        // A chain of nodes, each a list holding the next, is a valid encoding however long it is.
        let chain = |links: usize| {
            let mut nodes: Vec<DataValueInner> = (1..links)
                .map(|next| DataValueInner::List(vec![next as u16]))
                .collect();
            nodes.push(DataValueInner::List(vec![]));
            wit(DataValueInner::List(vec![0]), Some(nodes))
        };

        // The root and `links` nodes are all lists.
        assert!(DataValue::unpack(&chain(MAX_DEPTH - 1)).is_ok());
        assert_eq!(
            DataValue::unpack(&chain(MAX_DEPTH)),
            Err(UnpackError::TooDeep)
        );

        // The longest chain a data-value can hold would nest 65,536 deep.
        assert_eq!(DataValue::unpack(&chain(65_536)), Err(UnpackError::TooDeep));
    }

    #[test]
    fn try_from_goes_both_ways() {
        let value = DataValue::from(vec![1i64, 2]);
        let packed = WitDataValue::try_from(&value).unwrap();
        assert_eq!(DataValue::try_from(&packed).unwrap(), value);
        assert_eq!(
            DataValue::try_from(WitDataValue::from("x")).unwrap(),
            DataValue::from("x")
        );
    }
}
