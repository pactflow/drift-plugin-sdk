# 0001. The Rust SDK has its own value type

Status: accepted, 2026-11-24.

It resolves question 3 of [the proposal](../proposals/repository-and-releases.md#8-open-questions):
"the Rust value type. Its own, as §6 proposes, or a slimmed public `drift-models`?"

## Context

Every call between Drift and a plugin passes a WIT `data-value`: a root value and a flat list of nodes
addressed by index, because WIT has no recursive types. Drift's own plugins turn that into a
`drift_models::DataValue` and back with a conversion file, `pack_unpack.rs`, copied into 17 plugins.

`drift_models` is a private crate. It is also much more than a value type. `DataValue` sits in the
module for Drift's test-case model, and the crate pulls in a YAML parser, an expression parser, JSON
Schema generation and HTTP types. Publishing it would make that model a public API, used at 101 import
sites in Drift's plugins, which then could not change without a breaking release.

## Decision

The SDK has its own `DataValue`, in `drift_plugin_sdk`. It has the same variants as Drift's, and the
accessors and `From` conversions plugins use most, and nothing else. It does not depend on
`drift_models` and does not re-export it.

It differs from Drift's in ways that matter to someone moving a plugin over:

- Bytes are a `Vec<u8>`, not `bytes::Bytes`.
- Accessors borrow instead of cloning. `get`, `as_map`, `as_list` and `as_bytes` return references.
- There are no merge, compare, lookup, JSON, YAML or HTTP helpers.
- `From<()>` is not provided. Drift's makes it `Boolean(true)`.
- The conversion to and from the WIT record is fallible. See below.

The conversion is part of the decision, because the copies disagree with the WIT in ways that need
settling once, for every SDK:

- A value with more than 65,536 nodes cannot be packed. Nodes are addressed by a `u16`, and the
  copies wrapped silently.
- Lists and maps nest at most 128 deep, packing and unpacking, so that a crafted `data-value` cannot
  exhaust the stack. 128 is `serde_json`'s limit, so a value parsed from JSON is never over it.
- An index in a packed value must point forward of its own node, so a cyclic encoding is an error and
  not a loop. The copies indexed out of range and panicked.
- Map keys are packed in UTF-8 byte order.

These are written down in `conformance/data-values/README.md` and pinned by its vectors, which every
SDK runs.

## Consequences

- Drift's plugins cannot swap the SDK's type in without a conversion at the edge where they meet
  `drift_models`. The 17 copies of `pack_unpack.rs` go only when a plugin stops using `drift_models`
  for what it holds, or the SDK's type grows what it needs.
- Moving a plugin over means handling the differences above. The compiler finds most of them.
- A value that Drift's plugins can pack today may not pack through the SDK: one nested deeper than 128,
  or with more than 65,536 nodes. The first is not an encoding anyone should produce. The second is a
  limit of the WIT, and the copies were already wrong past it.
- Other languages implement the same rules against the same vectors. A value type that is not Rust's is
  theirs to design.

## Alternatives

- **A slimmed, public `drift-models`.** One type everywhere, and no conversion at the edge. It makes
  Drift commit to a public crate, and its release process and registry with it, before anything outside
  Drift has used it. It can still happen later: it would supersede this record.
- **Re-export `drift_models`.** The same commitment, with the whole test-case model attached.
