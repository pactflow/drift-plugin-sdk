# 0002. The Rust SDK generates only the `data-values` bindings, and converts its manifest into a plugin's own

Status: accepted, 2026-11-24.

It departs from §6 of [the proposal](../proposals/repository-and-releases.md#6-what-goes-in-an-sdk),
which has the SDK ship "bindings generated from the vendored WIT, re-exported so a plugin does not run
`wit-bindgen` itself".

## Context

A plugin is a component built from a world it writes itself. It imports and exports only the interfaces
it uses. Drift's plugins each have a `world.wit` for this. An SDK that wants to give a plugin generated
types has to coexist with that world, and these are what was tried against `wit-bindgen` 0.62 and
`drift:plugins@1.0.0`.

1. **Generate the whole `plugin` world in the SDK.** The SDK's crate then carries that world's metadata
   into every component that links it. A plugin whose world exports a different set of interfaces fails
   to build: `export data-format depends on interop previously as an import`.
2. **Generate a world that only imports `data-values`.** This builds, and the component exports only
   what the plugin's world says. `data-values` is only types, which nothing exports, so the two
   worlds agree. The plugin tells `wit-bindgen` to use the SDK's types with
   `with: { "drift:plugins/data-values@1.0.0": ... }`.
3. **The same for `plugin-manifest`, so a builder can return the generated manifest.** The interface is
   exported by the plugin. A `with` entry for an exported interface is not honoured: the plugin
   generates its own `Manifest` type, and the SDK's is a different type. Making the SDK export
   `plugin-manifest` as well and remapping onto its `exports` module has the same result.

## Decision

- The SDK generates the `data-values` interface only, from a world that imports nothing else. A plugin
  maps its own `data-values` onto the SDK's, which makes the SDK's `DataValue` conversion work on the
  values the plugin is given.
- The manifest builder produces an SDK-owned `Manifest`, made of plain Rust types and packed metadata.
  A macro, `manifest_into_wit!`, implements `From<Manifest>` for the manifest type the plugin's own
  bindings generated. The plugin names the module it was generated into, once.
- `interface-version` is set by the SDK from the package line of the vendored WIT, by a build script, so
  that moving the SDK to a new WIT major changes it without anyone remembering to.

## Consequences

- A plugin still runs `wit-bindgen` for its own world, and so uses the same `wit-bindgen` minor as the
  SDK. The SDK re-exports it as `drift_plugin_sdk::wit_bindgen`.
- The manifest has one line of glue in each plugin: the macro call. It cannot be removed without a
  change in `wit-bindgen`, or a world that needs no exports from the plugin.
- Interfaces that a plugin exports will need the same treatment, an SDK-owned type and a conversion,
  where the SDK wants to offer helpers for them. Interfaces a plugin imports and that are only types or
  resources (`logger`, `config`, `events`) can be generated in the SDK as `data-values` is, if their
  imports are not a problem. That has not been tried.
- If `wit-bindgen` starts honouring `with` for exported interfaces, the macro and the owned manifest can
  go and the builder can return the generated type. That would supersede this record.

## Alternatives

- **Ship a `wit-bindgen` macro wrapper**, so a plugin writes `drift_plugin_sdk::generate!`. It would
  hide the `with` entry and the version match. It needs the plugin's world to be written as a
  parameter, and is worth doing once there are enough helpers to justify it.
- **Make the SDK define the world**, with one world per kind of plugin (question 2 of the proposal's open
  questions). That would let the SDK own the export glue. It needs worlds published in the WIT first.
