//! SDK for writing Drift plugins in Rust.
//!
//! A Drift plugin is a WebAssembly component that implements parts of the `drift:plugins` WIT
//! package. This crate is at an early stage. It has:
//!
//! * [`DataValue`]: a value type, and its conversion to and from the WIT `data-value` record,
//!   which is what every call between Drift and a plugin passes.
//! * [`Manifest`]: a builder for the manifest a plugin returns from `get-manifest`, which sets
//!   `interface-version` from the WIT, and [`manifest_into_wit!`] to convert it to the plugin's own
//!   generated type.
//! * [`bindings`]: the generated Rust types for the `data-values` interface of
//!   `drift:plugins@1.0.0`, the WIT this version of the SDK targets.
//!
//! It does not yet have bindings for the rest of the WIT, helpers for events or plugin config, or a
//! command to build a `.plugin` file. A plugin still runs `wit-bindgen` for its own world. The
//! repository's `text-plugin` example shows all of this working together.
//!
//! # Using it in a plugin
//!
//! The conversions are implemented on the types in [`bindings`], so a plugin must tell `wit-bindgen`
//! to use them for `data-values`. Otherwise it generates types of its own, which [`DataValue`]
//! cannot convert:
//!
//! ```ignore
//! wit_bindgen::generate!({
//!     world: "component:my-plugin/my-plugin",
//!     path: ["wit/plugin.wit", "wit/world.wit"],
//!     generate_all,
//!     with: {
//!         "drift:plugins/data-values@1.0.0": drift_plugin_sdk::bindings::drift::plugins::data_values,
//!     }
//! });
//!
//! drift_plugin_sdk::manifest_into_wit!(exports::drift::plugins::plugin_manifest);
//!
//! // In a handler that is given `data: drift_plugin_sdk::bindings::drift::plugins::data_values::DataValue`:
//! let value = drift_plugin_sdk::DataValue::unpack(&data)?;
//! ```
//!
//! Use the same `wit-bindgen` version as this crate, which is 0.62.

pub mod bindings;
mod convert;
mod manifest;
mod value;
mod wit_version {
    include!(concat!(env!("OUT_DIR"), "/wit_version.rs"));
}

pub use convert::{PackError, UnpackError, MAX_DEPTH};
pub use manifest::{
    Capability, CapabilityType, Manifest, ManifestBuilder, ManifestError, WireCapability,
};
pub use value::DataValue;
pub use wit_bindgen;
pub use wit_version::{INTERFACE_VERSION, WIT_VERSION};
