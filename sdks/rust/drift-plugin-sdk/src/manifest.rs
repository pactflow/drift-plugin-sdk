//! A plugin's manifest: what it is called, what it provides and which events it wants.
//!
//! Build one with [`Manifest::builder`], and turn it into the type a plugin's own bindings generated
//! for `get-manifest` with [`manifest_into_wit!`](crate::manifest_into_wit).
//!
//! ```
//! use drift_plugin_sdk::{Capability, CapabilityType, Manifest};
//!
//! let manifest = Manifest::builder("json", "1.2.0")
//!     .description("Parses and validates JSON")
//!     .capability(
//!         Capability::new("json", CapabilityType::DataFormat)
//!             .metadata("content-type", "application/json")
//!             .metadata("content-type", "application/*+json"),
//!     )
//!     .build()?;
//!
//! assert_eq!(manifest.interface_version, drift_plugin_sdk::INTERFACE_VERSION);
//! # Ok::<(), drift_plugin_sdk::ManifestError>(())
//! ```

use std::fmt::{Display, Formatter};

use crate::bindings::drift::plugins::data_values::DataValue as WitDataValue;
use crate::convert::PackError;
use crate::value::DataValue;
use crate::INTERFACE_VERSION;

/// The lowest number the WIT gives to a capability that is not one of the named types.
const FIRST_OTHER_CAPABILITY: u16 = 9;

/// The kind of thing a capability says a plugin provides.
///
/// These are the values of the WIT's `capability-type`. A capability crosses the boundary as the
/// number, and an unknown one is not an error to the host, so [`CapabilityType::Other`] is how a plugin
/// provides a kind this SDK does not name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityType {
    /// A source file format that provides test cases. Not yet supported by Drift.
    TestcaseFile,
    /// A source file format that provides details about operations, such as OpenAPI.
    OperationFile,
    /// Dataset source files.
    DatasetFile,
    /// Data values that can be bound to a test value.
    DataValues,
    /// A validator. Not yet supported by Drift.
    DataValidator,
    /// Test setup functionality. Not yet supported by Drift.
    TestLifecycle,
    /// A network transport, such as HTTP or gRPC.
    NetworkTransport,
    /// A data format, such as JSON.
    DataFormat,
    /// Any other kind. The number must be 9 or more, since lower ones are the kinds above.
    Other(u16),
}

impl CapabilityType {
    /// The number the WIT uses for this kind.
    pub fn value(self) -> u16 {
        match self {
            CapabilityType::TestcaseFile => 1,
            CapabilityType::OperationFile => 2,
            CapabilityType::DatasetFile => 3,
            CapabilityType::DataValues => 4,
            CapabilityType::DataValidator => 5,
            CapabilityType::TestLifecycle => 6,
            CapabilityType::NetworkTransport => 7,
            CapabilityType::DataFormat => 8,
            CapabilityType::Other(value) => value,
        }
    }
}

/// Something a plugin provides, and the details Drift needs to find and use it.
#[derive(Clone, Debug, PartialEq)]
pub struct Capability {
    /// What the capability is stored against.
    pub key: String,
    /// The kind of capability.
    pub capability_type: CapabilityType,
    /// Details of the capability. A key may be repeated, as `content-type` is for a data format that
    /// handles more than one.
    pub metadata: Vec<(String, DataValue)>,
}

impl Capability {
    /// A capability with no metadata.
    pub fn new(key: impl Into<String>, capability_type: CapabilityType) -> Self {
        Capability {
            key: key.into(),
            capability_type,
            metadata: Vec::new(),
        }
    }

    /// Adds a metadata entry. Entries are kept in the order they are added, and a key can be added
    /// more than once.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<DataValue>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

/// A capability as it crosses the boundary: its metadata is already packed.
#[derive(Clone, Debug, PartialEq)]
pub struct WireCapability {
    /// What the capability is stored against.
    pub key: String,
    /// The number for the kind of capability. See [`CapabilityType::value`].
    pub capability: u16,
    /// The metadata, in the order it was added.
    pub metadata: Vec<(String, WitDataValue)>,
}

/// A plugin's manifest, ready to return from `get-manifest`.
///
/// [`manifest_into_wit!`](crate::manifest_into_wit) converts it to the plugin's generated type.
#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    /// The name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
    /// What the plugin does.
    pub description: String,
    /// The major version of the WIT the plugin was built against. See [`INTERFACE_VERSION`].
    pub interface_version: u16,
    /// What the plugin provides.
    pub capabilities: Vec<WireCapability>,
    /// The events the plugin wants. `*` matches all events or all of a class, so `file:*` is every
    /// file event and `*:failed` is every failure. A plugin that lists any must also implement the
    /// `event-callback` interface.
    pub events: Vec<String>,
}

impl Manifest {
    /// Starts a manifest. The name and version are the only things a manifest cannot do without.
    pub fn builder(name: impl Into<String>, version: impl Into<String>) -> ManifestBuilder {
        ManifestBuilder {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            capabilities: Vec::new(),
            events: Vec::new(),
        }
    }
}

/// Builds a [`Manifest`]. Get one from [`Manifest::builder`].
#[derive(Clone, Debug)]
pub struct ManifestBuilder {
    name: String,
    version: String,
    description: String,
    capabilities: Vec<Capability>,
    events: Vec<String>,
}

impl ManifestBuilder {
    /// Says what the plugin does. It is empty if this is not called.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds something the plugin provides.
    pub fn capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Adds an event the plugin wants to be told about.
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.events.push(event.into());
        self
    }

    /// Checks what was given and builds the manifest, with `interface_version` set to the SDK's
    /// [`INTERFACE_VERSION`].
    pub fn build(self) -> Result<Manifest, ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::EmptyName);
        }
        if self.version.is_empty() {
            return Err(ManifestError::EmptyVersion);
        }

        let mut capabilities = Vec::with_capacity(self.capabilities.len());
        for capability in self.capabilities {
            if capability.key.is_empty() {
                return Err(ManifestError::EmptyCapabilityKey);
            }
            let value = capability.capability_type.value();
            let other_uses_a_named_number =
                matches!(capability.capability_type, CapabilityType::Other(_))
                    && value < FIRST_OTHER_CAPABILITY;
            if other_uses_a_named_number {
                return Err(ManifestError::InvalidCapabilityType {
                    key: capability.key,
                    value,
                });
            }
            let mut metadata = Vec::with_capacity(capability.metadata.len());
            for (key, data) in &capability.metadata {
                let packed = data.pack().map_err(|error| ManifestError::Metadata {
                    capability: capability.key.clone(),
                    key: key.clone(),
                    error,
                })?;
                metadata.push((key.clone(), packed));
            }
            capabilities.push(WireCapability {
                key: capability.key,
                capability: value,
                metadata,
            });
        }

        Ok(Manifest {
            name: self.name,
            version: self.version,
            description: self.description,
            interface_version: INTERFACE_VERSION,
            capabilities,
            events: self.events,
        })
    }
}

/// Why a manifest could not be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    /// The plugin has no name.
    EmptyName,
    /// The plugin has no version.
    EmptyVersion,
    /// A capability has no key.
    EmptyCapabilityKey,
    /// A capability is [`CapabilityType::Other`] with a number that one of the named kinds has.
    InvalidCapabilityType {
        /// The capability's key.
        key: String,
        /// The number it was given.
        value: u16,
    },
    /// A metadata value could not be packed.
    Metadata {
        /// The key of the capability the metadata is on.
        capability: String,
        /// The metadata's key.
        key: String,
        /// Why it could not be packed.
        error: PackError,
    },
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::EmptyName => write!(f, "the plugin has no name"),
            ManifestError::EmptyVersion => write!(f, "the plugin has no version"),
            ManifestError::EmptyCapabilityKey => write!(f, "a capability has no key"),
            ManifestError::InvalidCapabilityType { key, value } => write!(
                f,
                "capability `{key}` is Other({value}), but {value} is the number of a named kind; \
                 other kinds start at {FIRST_OTHER_CAPABILITY}"
            ),
            ManifestError::Metadata {
                capability,
                key,
                error,
            } => {
                write!(f, "metadata `{key}` on capability `{capability}`: {error}")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// Implements `From<drift_plugin_sdk::Manifest>` for the manifest type a plugin's own bindings
/// generated, so that `get-manifest` can return `manifest.into()`.
///
/// `wit-bindgen` generates a plugin's manifest type in the module for the `plugin-manifest` interface
/// that the plugin exports, and gives the SDK no way to name it. Call this once, with the path of that
/// module:
///
/// ```ignore
/// wit_bindgen::generate!({ /* ... */ });
///
/// drift_plugin_sdk::manifest_into_wit!(exports::drift::plugins::plugin_manifest);
///
/// impl exports::drift::plugins::plugin_manifest::Guest for MyPlugin {
///     fn get_manifest() -> exports::drift::plugins::plugin_manifest::Manifest {
///         drift_plugin_sdk::Manifest::builder("my-plugin", env!("CARGO_PKG_VERSION"))
///             .build()
///             .expect("the manifest is valid")
///             .into()
///     }
/// }
/// ```
///
/// The plugin must have mapped its `data-values` onto the SDK's, as the crate documentation says,
/// since the metadata in the manifest is the SDK's `data-value` type.
#[macro_export]
macro_rules! manifest_into_wit {
    ($($module:ident)::+) => {
        impl ::core::convert::From<$crate::Manifest> for $($module)::+::Manifest {
            fn from(manifest: $crate::Manifest) -> Self {
                Self {
                    name: manifest.name.into(),
                    version: manifest.version.into(),
                    description: manifest.description.into(),
                    interface_version: manifest.interface_version,
                    capabilities: manifest
                        .capabilities
                        .into_iter()
                        .map(|capability| $($module)::+::Capability {
                            key: capability.key.into(),
                            capability: capability.capability,
                            metadata: capability
                                .metadata
                                .into_iter()
                                .map(|(key, value)| (key.into(), value))
                                .collect(),
                        })
                        .collect(),
                    events: manifest.events.into_iter().map(::core::convert::Into::into).collect(),
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::drift::plugins::data_values::StringValue;
    use crate::WIT_VERSION;

    fn data_format() -> Capability {
        Capability::new("json", CapabilityType::DataFormat)
    }

    #[test]
    fn builds_with_the_sdks_interface_version() {
        let manifest = Manifest::builder("json", "1.2.0")
            .description("Parses JSON")
            .capability(data_format().metadata("content-type", "application/json"))
            .event("file:*")
            .build()
            .unwrap();

        assert_eq!(manifest.name, "json");
        assert_eq!(manifest.version, "1.2.0");
        assert_eq!(manifest.description, "Parses JSON");
        assert_eq!(manifest.interface_version, INTERFACE_VERSION);
        assert_eq!(manifest.events, vec!["file:*".to_string()]);
        assert_eq!(manifest.capabilities.len(), 1);
        assert_eq!(manifest.capabilities[0].key, "json");
        assert_eq!(manifest.capabilities[0].capability, 8);
    }

    #[test]
    fn description_defaults_to_empty() {
        assert_eq!(Manifest::builder("a", "1").build().unwrap().description, "");
    }

    #[test]
    fn metadata_keeps_its_order_and_repeated_keys() {
        let manifest = Manifest::builder("json", "1")
            .capability(
                data_format()
                    .metadata("content-type", "application/json")
                    .metadata("other", 1i64)
                    .metadata("content-type", "application/*+json"),
            )
            .build()
            .unwrap();

        let metadata = &manifest.capabilities[0].metadata;
        let keys: Vec<_> = metadata.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["content-type", "other", "content-type"]);
        assert_eq!(
            DataValue::unpack(&metadata[0].1).unwrap(),
            DataValue::from("application/json")
        );
        assert_eq!(
            DataValue::unpack(&metadata[1].1).unwrap(),
            DataValue::Integer(1)
        );
        assert_eq!(
            DataValue::unpack(&metadata[2].1).unwrap(),
            DataValue::from("application/*+json")
        );
    }

    #[test]
    fn capability_types_have_the_wits_numbers() {
        let numbers: Vec<u16> = [
            CapabilityType::TestcaseFile,
            CapabilityType::OperationFile,
            CapabilityType::DatasetFile,
            CapabilityType::DataValues,
            CapabilityType::DataValidator,
            CapabilityType::TestLifecycle,
            CapabilityType::NetworkTransport,
            CapabilityType::DataFormat,
            CapabilityType::Other(9),
            CapabilityType::Other(500),
        ]
        .into_iter()
        .map(CapabilityType::value)
        .collect();
        assert_eq!(numbers, [1, 2, 3, 4, 5, 6, 7, 8, 9, 500]);
    }

    #[test]
    fn rejects_what_is_not_a_manifest() {
        assert_eq!(
            Manifest::builder("", "1").build(),
            Err(ManifestError::EmptyName)
        );
        assert_eq!(
            Manifest::builder("a", "").build(),
            Err(ManifestError::EmptyVersion)
        );
        assert_eq!(
            Manifest::builder("a", "1")
                .capability(Capability::new("", CapabilityType::DataFormat))
                .build(),
            Err(ManifestError::EmptyCapabilityKey)
        );
        assert_eq!(
            Manifest::builder("a", "1")
                .capability(Capability::new("k", CapabilityType::Other(8)))
                .build(),
            Err(ManifestError::InvalidCapabilityType {
                key: "k".into(),
                value: 8
            })
        );
    }

    #[test]
    fn reports_metadata_that_cannot_be_packed() {
        let too_deep = (0..crate::MAX_DEPTH + 1)
            .fold(DataValue::Null, |inner, _| DataValue::List(vec![inner]));
        let result = Manifest::builder("a", "1")
            .capability(data_format().metadata("deep", too_deep))
            .build();
        assert_eq!(
            result,
            Err(ManifestError::Metadata {
                capability: "json".into(),
                key: "deep".into(),
                error: PackError::TooDeep
            })
        );
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("deep") && message.contains("json"),
            "{message}"
        );
    }

    #[test]
    fn interface_version_is_the_major_of_the_vendored_wit() {
        let wit = include_str!("../wit/plugin.wit");
        assert!(
            wit.contains(&format!("package drift:plugins@{WIT_VERSION};")),
            "the WIT is not version {WIT_VERSION}"
        );
        assert_eq!(
            WIT_VERSION.split('.').next(),
            Some(INTERFACE_VERSION.to_string().as_str())
        );
    }

    /// Stands in for the module `wit-bindgen` generates in a plugin.
    mod generated {
        use crate::bindings::drift::plugins::data_values::{DataValue, StringValue};

        #[derive(Debug, PartialEq)]
        pub struct Manifest {
            pub name: StringValue,
            pub version: StringValue,
            pub description: StringValue,
            pub interface_version: u16,
            pub capabilities: Vec<Capability>,
            pub events: Vec<StringValue>,
        }

        #[derive(Debug, PartialEq)]
        pub struct Capability {
            pub key: StringValue,
            pub capability: u16,
            pub metadata: Vec<(StringValue, DataValue)>,
        }
    }

    manifest_into_wit!(tests::generated);

    #[test]
    fn converts_to_a_generated_manifest() {
        let manifest = Manifest::builder("json", "1.2.0")
            .description("Parses JSON")
            .capability(data_format().metadata("content-type", "application/json"))
            .event("file:*")
            .build()
            .unwrap();
        let packed = manifest.capabilities[0].metadata[0].1.clone();

        let wit: generated::Manifest = manifest.into();
        assert_eq!(
            wit,
            generated::Manifest {
                name: StringValue::from("json"),
                version: StringValue::from("1.2.0"),
                description: StringValue::from("Parses JSON"),
                interface_version: INTERFACE_VERSION,
                capabilities: vec![generated::Capability {
                    key: StringValue::from("json"),
                    capability: 8,
                    metadata: vec![(StringValue::from("content-type"), packed)],
                }],
                events: vec![StringValue::from("file:*")],
            }
        );
    }
}
