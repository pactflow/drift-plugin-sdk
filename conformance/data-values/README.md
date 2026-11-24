# data-values

Vectors for the `data-value` conversion every SDK implements: a value and its WIT `data-value`
encoding (a flattened node list addressed by index), checked in both directions. Each SDK's `test`
recipe runs them.

`vectors.json` holds them. The Rust SDK's runner is `sdks/rust/drift-plugin-sdk/tests/conformance.rs`.
An SDK in another language writes its own runner for the same file.

## The encoding

WIT has no recursive types, so a value crosses the boundary as a record of a root value and a list of
nodes (see `data-value` in `wit/wit.lock.json`'s package). A list or map refers to its children by
their index in the node list, as a `u16`.

**Packing** a value:

- A scalar is the root, and `nodes` is none.
- A list or map is the root, with its children in `nodes`. An empty list or map has an empty `nodes`,
  not none.
- Nodes are in pre-order: a node comes first, then everything beneath it, then its next sibling. A
  node's index is its position in that walk.
- A map's entries are in key order, comparing keys as UTF-8 bytes. That is not the same as comparing
  UTF-16 code units, which JavaScript and Java do by default, so there are vectors for it.
- Text is UTF-8, and no encoding is named.
- A value with more nodes than a `u16` can address cannot be packed, and packing reports an error.

**Unpacking** accepts more than packing produces:

- A string may name an encoding. `UTF-8`, `UTF-16LE` and `UTF-16BE` are understood, in any case. Any
  other encoding is read as UTF-8. Text that is invalid in its encoding is decoded with replacement
  characters; the vectors do not pin down which.
- Map entries may be in any order, a node may be referred to more than once, and nodes may go
  unused.
- Duplicate keys in a map are not specified.
- An index must be inside the node list, and inside a node it must be greater than that node's own
  index. A node that refers to itself or to an earlier node is an error, since following it could loop
  for ever. Unpacking reports `index-out-of-range` or `index-not-forward`.

## The file

```json
{ "format": 1, "vectors": [ { "name": "...", ... } ] }
```

A vector has a unique `name` and a `wit` side. It also has either a `value` or an `unpack_error`.

- `value` is the value, tagged by `type`: `null`, `string`, `int`, `uint`, `double`, `boolean`,
  `bytes`, `error`, `list` and `map`. `int` and `uint` hold their number as a string, since 64-bit
  integers do not survive JSON in every language. `bytes` is lowercase hex. A `list` holds values and
  a `map` is an object of values.
- `wit` is `{ "value": <inner>, "nodes": null | [<inner>...] }`, mirroring the record. An `<inner>`
  is tagged by `type` as the WIT variant is: `null`, `str`, `int`, `uint`, `double`, `boolean`,
  `bytes`, `error`, `list` (an array of indices) and `map` (an array of `[key, index]` pairs). A
  `str`, `error` or map key is a string-value: a JSON string for UTF-8 with no encoding, or
  `{ "encoding": "...", "data": "<hex>" }`.

A runner does this for each vector:

- With `unpack_error`: unpacking `wit` must fail with that kind of error.
- Otherwise: unpacking `wit` must give `value`, and, unless the vector has `"pack": false`, packing
  `value` must give `wit`. `"pack": false` marks a vector whose `wit` is a form that unpacking accepts
  but packing does not produce.

New vectors go at the end of their group. Do not change a vector to suit an SDK: an SDK that
disagrees has a bug, or the vector is wrong and every SDK is told.
