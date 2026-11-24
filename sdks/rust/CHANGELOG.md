# Changelog

All notable changes to the Rust SDK are recorded here. It is maintained by knope from the change files
in `.changeset/`; do not edit it by hand.
## 0.1.0 (2026-11-24)

### Features

- Add DataValue and its conversion to the WIT data-value (#3)
- Add a manifest builder and an example plugin (#4)

#### Add DataValue and its conversion to the WIT data-value

`DataValue` is a value type with `pack` and `unpack`, which convert it to and from the flattened WIT
`data-value` record that every call between Drift and a plugin passes. The crate also generates the
Rust types for the `data-values` interface of `drift:plugins@1.0.0`, in `bindings`.

`pack` fails with `PackError` when a value has more nodes than a `u16` index can address, or nests
lists and maps more than `MAX_DEPTH` (128) deep. `unpack` fails with `UnpackError` on an index that is
out of range or does not point forwards, or on nesting past `MAX_DEPTH`. The conversion is tested
against the shared `conformance/data-values` vectors.

#### Add a manifest builder and an example plugin

`Manifest::builder` builds the manifest a plugin returns from `get-manifest`, with `Capability` and
`CapabilityType` for what it provides. It sets `interface-version` from the WIT the SDK vendors, which
is also available as `INTERFACE_VERSION` and `WIT_VERSION`. `build` reports a missing name, version or
capability key, a capability type that does not follow the WIT's numbering, and metadata that cannot be
packed.

`manifest_into_wit!` implements `From<Manifest>` for the manifest type a plugin's own bindings
generate, so that `get-manifest` can return `manifest.into()`.

The `text-plugin` example is a complete data format plugin, and `just build-examples` builds it to
`target/plugins/text-plugin.plugin`.

### Fixes

#### Vendor drift:plugins@1.0.0

The crate now carries `wit/plugin.wit` for `drift:plugins@1.0.0`, vendored from Drift's published file.
