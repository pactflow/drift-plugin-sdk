//! An example Drift plugin: a data format for plain text.
//!
//! It shows the parts every plugin has. A world says what it implements, `generate!` makes the types
//! for it, and the SDK supplies the value type and the manifest. What is left is the plugin's own
//! behaviour, which is the three functions of the `data-format` interface.

use drift_plugin_sdk::bindings::drift::plugins::data_values::DataValue as WitDataValue;
use drift_plugin_sdk::{Capability, CapabilityType, DataValue, Manifest};

wit_bindgen::generate!({
    world: "component:text-plugin/text-plugin",
    path: ["../../wit/plugin.wit", "wit/world.wit"],
    generate_all,
    with: {
        // Use the SDK's `data-value` types, so that `DataValue::pack` and `unpack` work on them.
        "drift:plugins/data-values@1.0.0": drift_plugin_sdk::bindings::drift::plugins::data_values,
    }
});

use exports::drift::plugins::data_format::{FutureComputation, Guest as DataFormat};
use exports::drift::plugins::plugin_manifest::Guest as PluginManifest;

// `wit-bindgen` generated a `Manifest` of its own for this plugin. This lets us return the SDK's.
drift_plugin_sdk::manifest_into_wit!(exports::drift::plugins::plugin_manifest);

struct TextPlugin;

impl PluginManifest for TextPlugin {
    fn get_manifest() -> exports::drift::plugins::plugin_manifest::Manifest {
        Manifest::builder("text", env!("CARGO_PKG_VERSION"))
            .description("Plain text data format")
            .capability(
                // The key is what Drift stores the capability against. For a data format, the
                // content types it handles go in the metadata.
                Capability::new("text", CapabilityType::DataFormat)
                    .metadata("content-type", "text/plain"),
            )
            .build()
            .expect("the manifest is valid")
            .into()
    }
}

impl DataFormat for TextPlugin {
    /// Whether the plugin handles a content type. The manifest's metadata lists `text/plain`, and
    /// this answers for it too. In the Drift this was tested with, a content type that was only in
    /// the metadata was not matched, so every data format plugin answers here as well. It is also
    /// where a plugin can accept types it is configured to.
    fn supports_content_type(content_type: String) -> bool {
        is_text_plain(&content_type)
    }

    fn parse(data: WitDataValue, _content_type: Option<String>) -> FutureComputation {
        completed(unpack(&data).and_then(|data| parse(&data)))
    }

    fn construct(data: WitDataValue) -> FutureComputation {
        completed(unpack(&data).and_then(|data| construct(&data)))
    }

    fn validate(
        expected: Option<WitDataValue>,
        actual: WitDataValue,
        // Plain text has no schema language, so a schema is ignored.
        _schema: Option<WitDataValue>,
        _content_type: Option<String>,
    ) -> FutureComputation {
        let expected = match expected.as_ref().map(unpack).transpose() {
            Ok(expected) => expected,
            Err(e) => return completed(Err(e)),
        };
        completed(unpack(&actual).and_then(|actual| validate(expected.as_ref(), &actual)))
    }
}

export!(TextPlugin);

/// If a content type is `text/plain`, whatever parameters it has, such as `; charset=utf-8`.
fn is_text_plain(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .is_some_and(|essence| essence.trim().eq_ignore_ascii_case("text/plain"))
}

/// The text in a value that is text, or bytes of it.
fn text(value: &DataValue) -> Result<String, String> {
    match value {
        DataValue::String(text) => Ok(text.clone()),
        DataValue::Bytes(bytes) => {
            String::from_utf8(bytes.clone()).map_err(|e| format!("the bytes are not UTF-8: {e}"))
        }
        other => Err(format!(
            "expected text or bytes but got {}",
            other.type_name()
        )),
    }
}

/// Text, or bytes of it, parse to the text.
fn parse(data: &DataValue) -> Result<DataValue, String> {
    text(data).map(DataValue::String)
}

/// Text is already in its constructed form, and nothing is the empty text.
fn construct(data: &DataValue) -> Result<DataValue, String> {
    match data {
        DataValue::Null => Ok(DataValue::String(String::new())),
        other => text(other).map(DataValue::String),
    }
}

/// The result of validating is a map keyed by the kind of error, and an empty map is success. Each
/// error has a `location` and an `error`. Text has no structure, so the location is the whole of it.
fn validate(expected: Option<&DataValue>, actual: &DataValue) -> Result<DataValue, String> {
    let actual = text(actual)?;
    let mut errors = std::collections::HashMap::new();
    if let Some(expected) = expected {
        let expected = text(expected)?;
        if expected != actual {
            let mismatch = DataValue::from(std::collections::HashMap::from([
                ("location".to_string(), DataValue::from("")),
                (
                    "error".to_string(),
                    DataValue::from(format!("Expected {expected:?} but got {actual:?}")),
                ),
            ]));
            errors.insert("mismatches".to_string(), DataValue::List(vec![mismatch]));
        }
    }
    Ok(DataValue::Map(errors))
}

fn unpack(value: &WitDataValue) -> Result<DataValue, String> {
    DataValue::unpack(value).map_err(|e| e.to_string())
}

/// What a `data-format` function returns when it has the answer now. A failure is a value too, and
/// an error value is how Drift expects one.
fn completed(result: Result<DataValue, String>) -> FutureComputation {
    FutureComputation::Completed(match result {
        Ok(value) => value
            .pack()
            .map(|packed| (packed,))
            .map_err(|e| error_value(e.to_string())),
        Err(message) => Err(error_value(message)),
    })
}

fn error_value(message: String) -> WitDataValue {
    DataValue::Error(message)
        .pack()
        .expect("an error value is a scalar, so it packs")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_value(text: &str) -> DataValue {
        DataValue::from(text)
    }

    /// The value in a completed computation, or the error value it failed with.
    fn outcome(computation: FutureComputation) -> Result<DataValue, DataValue> {
        match computation {
            FutureComputation::Completed(result) => result
                .map(|(value,)| DataValue::unpack(&value).unwrap())
                .map_err(|error| DataValue::unpack(&error).unwrap()),
            _ => panic!("expected a completed computation"),
        }
    }

    fn wit(value: DataValue) -> WitDataValue {
        value.pack().unwrap()
    }

    #[test]
    fn manifest_provides_the_text_data_format() {
        let manifest = TextPlugin::get_manifest();
        assert_eq!(String::from(&manifest.name), "text");
        assert_eq!(
            manifest.interface_version,
            drift_plugin_sdk::INTERFACE_VERSION
        );
        assert_eq!(manifest.capabilities.len(), 1);
        let capability = &manifest.capabilities[0];
        assert_eq!(String::from(&capability.key), "text");
        assert_eq!(capability.capability, CapabilityType::DataFormat.value());
        let (key, value) = &capability.metadata[0];
        assert_eq!(String::from(key), "content-type");
        assert_eq!(DataValue::unpack(value).unwrap(), text_value("text/plain"));
    }

    #[test]
    fn supports_text_plain_with_or_without_parameters() {
        assert!(TextPlugin::supports_content_type("text/plain".into()));
        assert!(TextPlugin::supports_content_type(
            "text/plain; charset=utf-8".into()
        ));
        assert!(TextPlugin::supports_content_type("Text/Plain".into()));
        assert!(!TextPlugin::supports_content_type("text/html".into()));
        assert!(!TextPlugin::supports_content_type(
            "application/json".into()
        ));
        assert!(!TextPlugin::supports_content_type("text/plainer".into()));
    }

    #[test]
    fn parses_text_and_bytes_to_text() {
        let parsed = outcome(TextPlugin::parse(wit(text_value("hello")), None));
        assert_eq!(parsed, Ok(text_value("hello")));
        let parsed = outcome(TextPlugin::parse(
            wit(DataValue::Bytes(b"hello".to_vec())),
            None,
        ));
        assert_eq!(parsed, Ok(text_value("hello")));
    }

    #[test]
    fn parse_fails_with_an_error_value() {
        let failed = outcome(TextPlugin::parse(wit(DataValue::Integer(1)), None));
        assert_eq!(
            failed,
            Err(DataValue::Error(
                "expected text or bytes but got Integer".into()
            ))
        );
        let failed = outcome(TextPlugin::parse(wit(DataValue::Bytes(vec![0xff])), None));
        assert!(
            matches!(failed, Err(DataValue::Error(m)) if m.starts_with("the bytes are not UTF-8"))
        );
    }

    #[test]
    fn constructs_text() {
        assert_eq!(
            outcome(TextPlugin::construct(wit(text_value("hi")))),
            Ok(text_value("hi"))
        );
        assert_eq!(
            outcome(TextPlugin::construct(wit(DataValue::Null))),
            Ok(text_value(""))
        );
        assert!(outcome(TextPlugin::construct(wit(DataValue::Boolean(true)))).is_err());
    }

    #[test]
    fn validate_is_an_empty_map_when_the_text_matches() {
        let result = outcome(TextPlugin::validate(
            Some(wit(text_value("hello"))),
            wit(text_value("hello")),
            None,
            None,
        ));
        assert_eq!(result, Ok(DataValue::Map(Default::default())));

        let result = outcome(TextPlugin::validate(
            None,
            wit(text_value("anything")),
            None,
            None,
        ));
        assert_eq!(result, Ok(DataValue::Map(Default::default())));
    }

    #[test]
    fn validate_reports_a_mismatch() {
        let result = outcome(TextPlugin::validate(
            Some(wit(text_value("hello"))),
            wit(text_value("goodbye")),
            None,
            None,
        ))
        .unwrap();
        let mismatch = &result.get("mismatches").unwrap().as_list().unwrap()[0];
        assert_eq!(mismatch.get("location"), Some(&text_value("")));
        assert_eq!(
            mismatch.get("error"),
            Some(&text_value("Expected \"hello\" but got \"goodbye\""))
        );
    }

    #[test]
    fn a_value_that_cannot_be_unpacked_is_an_error() {
        use drift_plugin_sdk::bindings::drift::plugins::data_values::DataValueInner;
        let broken = WitDataValue {
            value: DataValueInner::List(vec![0]),
            nodes: None,
        };
        assert!(outcome(TextPlugin::parse(broken, None)).is_err());
    }
}
