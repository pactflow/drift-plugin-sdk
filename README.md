# Drift Plugin SDKs

SDKs for writing [Drift](https://pactflow.io) plugins: user-contributed WebAssembly components that
implement the `drift:plugins` WIT contract.

> This repository is being set up. No SDK has been released yet.
> See [the proposal](Documentation/proposals/repository-and-releases.md) for how it is organised.

## SDKs

| SDK | Directory | Status | WIT | Package |
|---|---|---|---|---|
| Rust | [`sdks/rust`](sdks/rust) | experimental | `1.0.0` | `drift-plugin-sdk` on crates.io (not yet published) |

Status is `experimental`, `beta` or `stable`, and is set in each SDK's `sdk.toml`.

To see what a plugin looks like, read the Rust example, [`text-plugin`](sdks/rust/examples/text-plugin).

## Compatibility

Three versions are kept apart: an SDK's version, the WIT's, and Drift's bundle version. Each SDK's README
and release notes name the WIT it targets; this table maps WIT versions to the first Drift bundle that
implements them.

| WIT (`drift:plugins`) | First Drift bundle |
|---|---|
| `1.0.0` | `2610.2.0` |

The table is written by hand, from the `index.json` that Drift publishes beside the WIT
(`https://download.pactflow.io/drift/wit/index.json`). The proposal has it generated from Drift's release
records. That is not done yet.

## Repository

| Path | What |
|---|---|
| `Documentation/` | The language-neutral plugin guide, proposals and decisions |
| `wit/` | `wit.lock.json`: the published WIT versions this repository uses, with their SHA-256 |
| `conformance/` | Test vectors and Drift test cases every SDK is held to |
| `sdks/<name>/` | One SDK per language, each the root of its own toolchain |
| `scripts/` | Language-neutral tooling used by CI and the justfile |

Nothing at the root belongs to a language. Run an SDK's recipes with `just sdk <name> <recipe>`, for
example `just sdk rust test`. Contributing is in [CONTRIBUTING.md](CONTRIBUTING.md); AI agents should read
[AGENTS.md](AGENTS.md).

## Licence

Apache-2.0. See [LICENSE](LICENSE).
