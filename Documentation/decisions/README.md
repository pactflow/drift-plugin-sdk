# Decisions

Architecture decision records. Each says what was decided, why, and what it costs, so that the next
person does not have to work it out again. They take precedence over the proposals they come from.

A record is numbered, dated and has a status: `accepted`, or `superseded by NNNN`. Do not edit an
accepted record to change the decision. Write a new one that supersedes it.

| Record | Decision |
|---|---|
| [0001](0001-rust-value-type.md) | The Rust SDK has its own value type |
| [0002](0002-rust-bindings-and-manifest.md) | The Rust SDK generates only the `data-values` bindings, and converts its manifest into a plugin's own |
| [0003](0003-release-process.md) | The release process publishes before it tags, bumps `Cargo.lock`, and builds knope's version override from `knope.toml` |
