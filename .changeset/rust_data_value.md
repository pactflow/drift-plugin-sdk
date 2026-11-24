---
rust: minor
---

# Add DataValue and its conversion to the WIT data-value

`DataValue` is a value type with `pack` and `unpack`, which convert it to and from the flattened WIT
`data-value` record that every call between Drift and a plugin passes. The crate also generates the
Rust types for the `data-values` interface of `drift:plugins@1.0.0`, in `bindings`.

`pack` fails with `PackError` when a value has more nodes than a `u16` index can address, and `unpack`
fails with `UnpackError` on an index that is out of range or does not point forwards. The conversion
is tested against the shared `conformance/data-values` vectors.
