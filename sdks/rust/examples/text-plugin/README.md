# text-plugin

An example Drift plugin written with `drift-plugin-sdk`. It provides one capability, a data format for
`text/plain`:

- `parse` takes text or bytes and returns text.
- `construct` takes text, bytes or nothing and returns text.
- `validate` compares the expected text with the actual text, if there is an expectation.

Read `src/lib.rs`. It is meant to be copied.

## Building it

From `sdks/rust`:

```sh
just build-examples       # target/plugins/text-plugin.plugin
```

A `.plugin` file is the built WebAssembly component with that extension. To use it, copy it into
Drift's plugin directory.

## What to copy

- `wit/world.wit` names the interfaces the plugin implements. Add `export drift:plugins/...` lines for
  other capabilities.
- The `generate!` call maps `data-values` onto the SDK's types. Without that, the SDK's `DataValue`
  cannot convert the values the plugin is given.
- `manifest_into_wit!` is the one line that lets `get-manifest` return an SDK `Manifest`.
