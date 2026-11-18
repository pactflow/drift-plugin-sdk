# suites

Drift test cases that each SDK's example plugins are run under, by the end-to-end CI job: it installs
the examples' `.plugin` files into a Drift bundle's plugin directory, pinned by version and checked
against its `SHA256SUMS`, and runs these. It is the only test that proves an SDK's output loads in Drift.

There are none yet, and the job is not written; both wait for the Rust SDK's first example.
