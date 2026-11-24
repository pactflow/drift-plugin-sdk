//! Runs `conformance/data-values/vectors.json`, which every SDK runs. See that directory's README
//! for the format.

use std::collections::HashMap;
use std::path::PathBuf;

use drift_plugin_sdk::bindings::drift::plugins::data_values::{
    DataValue as WitDataValue, DataValueInner, StringValue,
};
use drift_plugin_sdk::{DataValue, PackError, UnpackError};
use serde_json::Value as Json;

fn field<'a>(json: &'a Json, name: &str) -> &'a Json {
    json.get(name)
        .unwrap_or_else(|| panic!("{json} has no `{name}`"))
}

fn tag(json: &Json) -> &str {
    field(json, "type")
        .as_str()
        .unwrap_or_else(|| panic!("`type` of {json} is not a string"))
}

fn text(json: &Json) -> &str {
    json.as_str()
        .unwrap_or_else(|| panic!("{json} is not a string"))
}

fn hex(json: &Json) -> Vec<u8> {
    let digits = text(json);
    let (pairs, remainder) = digits.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty(), "odd length hex: {digits}");
    pairs
        .iter()
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .unwrap_or_else(|| panic!("bad hex: {digits}"))
        })
        .collect()
}

fn i64_of(json: &Json) -> i64 {
    text(json)
        .parse()
        .unwrap_or_else(|_| panic!("{json} is not an i64"))
}

fn u64_of(json: &Json) -> u64 {
    text(json)
        .parse()
        .unwrap_or_else(|_| panic!("{json} is not a u64"))
}

fn f64_of(json: &Json) -> f64 {
    json.as_f64()
        .unwrap_or_else(|| panic!("{json} is not a number"))
}

/// The `value` side of a vector.
fn value(json: &Json) -> DataValue {
    let inner = || field(json, "value");
    match tag(json) {
        "null" => DataValue::Null,
        "string" => DataValue::String(text(inner()).to_string()),
        "int" => DataValue::Integer(i64_of(inner())),
        "uint" => DataValue::UInteger(u64_of(inner())),
        "double" => DataValue::Double(f64_of(inner())),
        "boolean" => DataValue::Boolean(inner().as_bool().expect("a boolean")),
        "bytes" => DataValue::Bytes(hex(inner())),
        "error" => DataValue::Error(text(inner()).to_string()),
        "list" => DataValue::List(
            inner()
                .as_array()
                .expect("an array")
                .iter()
                .map(value)
                .collect(),
        ),
        "map" => DataValue::Map(
            inner()
                .as_object()
                .expect("an object")
                .iter()
                .map(|(k, v)| (k.clone(), value(v)))
                .collect::<HashMap<_, _>>(),
        ),
        other => panic!("unknown value type `{other}`"),
    }
}

/// A WIT `string-value`: a JSON string is UTF-8 with no encoding named.
fn string_value(json: &Json) -> StringValue {
    match json {
        Json::String(s) => StringValue {
            encoding: None,
            data: s.as_bytes().to_vec(),
        },
        Json::Object(_) => StringValue {
            encoding: Some(text(field(json, "encoding")).to_string()),
            data: hex(field(json, "data")),
        },
        other => panic!("{other} is not a string-value"),
    }
}

fn index(json: &Json) -> u16 {
    json.as_u64()
        .and_then(|i| u16::try_from(i).ok())
        .unwrap_or_else(|| panic!("{json} is not a u16"))
}

/// A WIT `data-value-inner`.
fn inner(json: &Json) -> DataValueInner {
    let content = || field(json, "value");
    match tag(json) {
        "null" => DataValueInner::Null,
        "str" => DataValueInner::Str(string_value(content())),
        "int" => DataValueInner::Int(i64_of(content())),
        "uint" => DataValueInner::Uint(u64_of(content())),
        "double" => DataValueInner::Double(f64_of(content())),
        "boolean" => DataValueInner::Boolean(content().as_bool().expect("a boolean")),
        "bytes" => DataValueInner::Bytes(hex(content())),
        "error" => DataValueInner::Error(string_value(content())),
        "list" => DataValueInner::List(
            content()
                .as_array()
                .expect("an array")
                .iter()
                .map(index)
                .collect(),
        ),
        "map" => DataValueInner::Map(
            content()
                .as_array()
                .expect("an array")
                .iter()
                .map(|entry| {
                    let pair = entry.as_array().expect("a [key, index] pair");
                    assert_eq!(pair.len(), 2, "{entry} is not a [key, index] pair");
                    (string_value(&pair[0]), index(&pair[1]))
                })
                .collect(),
        ),
        other => panic!("unknown data-value-inner type `{other}`"),
    }
}

/// The `wit` side of a vector.
fn wit(json: &Json) -> WitDataValue {
    WitDataValue {
        value: inner(field(json, "value")),
        nodes: match field(json, "nodes") {
            Json::Null => None,
            nodes => Some(
                nodes
                    .as_array()
                    .expect("an array")
                    .iter()
                    .map(inner)
                    .collect(),
            ),
        },
    }
}

/// The value and encoding for a `nested_lists` vector: `depth` lists, each holding the next, with an
/// empty list at the bottom. The encoding is the root list `[0]` and `depth - 1` nodes, each a list of
/// the next node, the last an empty list. See the README.
fn nested_lists(depth: usize) -> (DataValue, WitDataValue) {
    assert!(
        depth >= 2,
        "a nested_lists vector needs a depth of at least 2"
    );
    let value = (1..depth).fold(DataValue::List(vec![]), |inner, _| {
        DataValue::List(vec![inner])
    });
    let mut nodes: Vec<DataValueInner> = (1..depth - 1)
        .map(|next| DataValueInner::List(vec![u16::try_from(next).expect("a small depth")]))
        .collect();
    nodes.push(DataValueInner::List(vec![]));
    let packed = WitDataValue {
        value: DataValueInner::List(vec![0]),
        nodes: Some(nodes),
    };
    (value, packed)
}

fn unpack_error_kind(error: UnpackError) -> &'static str {
    match error {
        UnpackError::IndexOutOfRange { .. } => "index-out-of-range",
        UnpackError::IndexNotForward { .. } => "index-not-forward",
        UnpackError::TooDeep => "too-deep",
    }
}

fn pack_error_kind(error: PackError) -> &'static str {
    match error {
        PackError::TooManyNodes => "too-many-nodes",
        PackError::TooDeep => "too-deep",
    }
}

#[test]
fn data_value_vectors() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../conformance/data-values/vectors.json");
    let file: Json = serde_json::from_str(
        &std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display())),
    )
    .expect("vectors.json is valid JSON");
    assert_eq!(
        field(&file, "format"),
        1,
        "this runner understands format 1"
    );

    let vectors = field(&file, "vectors").as_array().expect("an array");
    assert!(!vectors.is_empty());

    let mut names = std::collections::HashSet::new();
    let mut failures = Vec::new();
    for vector in vectors {
        let name = text(field(vector, "name"));
        assert!(names.insert(name), "two vectors are called `{name}`");

        if let Some(depth) = vector.get("nested_lists") {
            let depth = depth.as_u64().expect("a number") as usize;
            let (value, packed) = nested_lists(depth);
            match vector.get("error") {
                None => {
                    if DataValue::unpack(&packed) != Ok(value.clone()) {
                        failures.push(format!("{name}: unpack did not give the value"));
                    }
                    if value.pack().as_ref() != Ok(&packed) {
                        failures.push(format!("{name}: pack did not give the encoding"));
                    }
                }
                Some(kind) => {
                    let kind = text(kind);
                    match value.pack() {
                        Err(e) if pack_error_kind(e) == kind => {}
                        other => {
                            failures.push(format!("{name}: pack gave {other:?}, wanted {kind}"))
                        }
                    }
                    match DataValue::unpack(&packed) {
                        Err(e) if unpack_error_kind(e) == kind => {}
                        other => {
                            failures.push(format!("{name}: unpack gave {other:?}, wanted {kind}"))
                        }
                    }
                }
            }
            continue;
        }

        let packed = wit(field(vector, "wit"));

        if let Some(expected) = vector.get("unpack_error") {
            match DataValue::unpack(&packed) {
                Ok(unpacked) => {
                    failures.push(format!("{name}: unpacked to {unpacked:?}, wanted an error"))
                }
                Err(e) if unpack_error_kind(e) == text(expected) => {}
                Err(e) => failures.push(format!("{name}: got {e:?}, wanted {expected}")),
            }
            continue;
        }

        let expected = value(field(vector, "value"));
        match DataValue::unpack(&packed) {
            Ok(unpacked) if unpacked == expected => {}
            other => failures.push(format!(
                "{name}: unpack gave {other:?}, wanted {expected:?}"
            )),
        }
        if vector.get("pack").and_then(Json::as_bool) != Some(false) {
            match expected.pack() {
                Ok(repacked) if repacked == packed => {}
                other => failures.push(format!("{name}: pack gave {other:?}, wanted {packed:?}")),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} vectors failed:\n{}",
        failures.len(),
        vectors.len(),
        failures.join("\n")
    );
}
