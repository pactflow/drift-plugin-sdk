# drift-plugin-sdk

SDK for writing [Drift](https://pactflow.io) plugins in Rust.

This crate is at an early stage and its API will change. It targets the WIT package
`drift:plugins@1.0.0`. It has:

- `DataValue`, a value type, with `pack` and `unpack` to convert it to and from the WIT `data-value`
  record that every call between Drift and a plugin passes.
- `bindings`, the generated Rust types for the WIT `data-values` interface.

It does not yet have bindings for the rest of the WIT, a manifest builder, helpers for events or plugin
config, or a way to build a `.plugin` file. See the repository's
`Documentation/proposals/repository-and-releases.md`, §6, for where it is going.

## Using it

Add the crate and `wit-bindgen`, at the same version this crate uses:

```toml
[dependencies]
drift-plugin-sdk = "0"
wit-bindgen = "0.62"
```

The conversions are implemented on the types in `drift_plugin_sdk::bindings`, so your plugin's
`wit_bindgen::generate!` must use them for `data-values`:

```rust
wit_bindgen::generate!({
    world: "component:my-plugin/my-plugin",
    path: ["wit/plugin.wit", "wit/world.wit"],
    generate_all,
    with: {
        "drift:plugins/data-values@1.0.0": drift_plugin_sdk::bindings::drift::plugins::data_values,
    }
});
```

Then:

```rust
use drift_plugin_sdk::DataValue;

let value = DataValue::unpack(&wit_value)?;          // from what Drift sent
let reply = DataValue::from("done").pack()?;          // to send back
```

`pack` fails if the value has more than 65,536 nodes below its root, which is the most a `data-value`
can address. `unpack` fails on an index that points outside the node list or not forwards. Both return
errors you can show.
