---
rust: minor
---

# Add a manifest builder and an example plugin

`Manifest::builder` builds the manifest a plugin returns from `get-manifest`, with `Capability` and
`CapabilityType` for what it provides. It sets `interface-version` from the WIT the SDK vendors, which
is also available as `INTERFACE_VERSION` and `WIT_VERSION`. `build` reports a missing name, version or
capability key, a capability type that does not follow the WIT's numbering, and metadata that cannot be
packed.

`manifest_into_wit!` implements `From<Manifest>` for the manifest type a plugin's own bindings
generate, so that `get-manifest` can return `manifest.into()`.

The `text-plugin` example is a complete data format plugin, and `just build-examples` builds it to
`target/plugins/text-plugin.plugin`.
