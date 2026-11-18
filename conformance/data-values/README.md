# data-values

Vectors for the `data-value` conversion every SDK implements: a value and its WIT `data-value`
encoding (a flattened node list addressed by index), checked in both directions. Each SDK's `test`
recipe runs them.

There are none yet. They are written from the conversion Drift's plugins copy (`pack_unpack.rs`)
before the first SDK implements it, so that the SDKs agree with Drift and with each other.
