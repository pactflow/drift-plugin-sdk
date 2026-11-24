# drift-plugin-sdk

SDK for writing [Drift](https://support.smartbear.com/swagger/docs/contract-testing/drift/) plugins in Rust.

This crate is at an early stage and its API will change. It targets the WIT package
`drift:plugins@1.0.0`. It has:

- `DataValue`, a value type, with `pack` and `unpack` to convert it to and from the WIT `data-value`
  record that every call between Drift and a plugin passes.
- `Manifest`, with a builder, for the manifest a plugin returns from `get-manifest`. It sets
  `interface-version` for you.
- `bindings`, the generated Rust types for the WIT `data-values` interface.

It does not yet have bindings for the rest of the WIT, helpers for events or plugin config, or a command
to build a `.plugin` file. A plugin still runs `wit-bindgen` for its own world. See the repository's
`Documentation/proposals/repository-and-releases.md`, §6, for where it is going, and
`Documentation/decisions/` for why it is as it is.

## An example

[`examples/text-plugin`](https://github.com/pactflow/drift-plugin-sdk/tree/main/sdks/rust/examples/text-plugin) is a complete plugin: a data format for `text/plain`.
It is the place to start, and to copy from.

## Using it

The crate is not on crates.io yet. Until it is, depend on it from git, and add `wit-bindgen` at the same
minor version this crate uses:

```toml
[dependencies]
drift-plugin-sdk = { git = "https://github.com/pactflow/drift-plugin-sdk" }
wit-bindgen = "0.62"

[lib]
crate-type = ["cdylib"]
```

A plugin is a WebAssembly component built from a world that you write, naming the interfaces the
plugin implements:

```wit
package component:my-plugin;

world my-plugin {
  export drift:plugins/plugin-manifest@1.0.0;
  export drift:plugins/data-format@1.0.0;
}
```

The conversions are implemented on the types in `drift_plugin_sdk::bindings`, so your plugin's
`wit_bindgen::generate!` must use them for `data-values`. Also give the SDK's `manifest_into_wit!` the
module where the bindings put the manifest, so that you can return the SDK's `Manifest`:

```rust
wit_bindgen::generate!({
    world: "component:my-plugin/my-plugin",
    path: ["wit/plugin.wit", "wit/world.wit"],
    generate_all,
    with: {
        "drift:plugins/data-values@1.0.0": drift_plugin_sdk::bindings::drift::plugins::data_values,
    }
});

drift_plugin_sdk::manifest_into_wit!(exports::drift::plugins::plugin_manifest);
```

`wit/plugin.wit` is the `drift:plugins` WIT. It is vendored in this repository at `sdks/rust/wit/plugin.wit`,
and Drift publishes it at `https://download.pactflow.io/drift/wit/`.

Build it for `wasm32-wasip2`, and the result is the plugin:

```sh
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/my_plugin.wasm my-plugin.plugin
```

### Values

```rust
use drift_plugin_sdk::DataValue;

let value = DataValue::unpack(&wit_value)?;          // from what Drift sent
let reply = DataValue::from("done").pack()?;          // to send back
```

`pack` fails if the value has more than 65,536 nodes below its root, which is the most a `data-value`
can address, or if lists and maps nest more than 128 deep. `unpack` fails on an index that points
outside the node list or not forwards, and on the same nesting. Both return errors you can show.

### The manifest

```rust
use drift_plugin_sdk::{Capability, CapabilityType, Manifest};

let manifest = Manifest::builder("json", "1.2.0")
    .description("Parses and validates JSON")
    .capability(
        Capability::new("json", CapabilityType::DataFormat)
            .metadata("content-type", "application/json")
            .metadata("content-type", "application/*+json"),
    )
    .build()?;
```

`build` fails on a missing name, version or capability key, on a capability type that does not follow
the WIT's numbering, and on metadata that cannot be packed.

A plugin that provides a data format should also implement `supports-content-type` for the same content
types. In the Drift this SDK was tested with, a content type listed only in the manifest was not matched.
