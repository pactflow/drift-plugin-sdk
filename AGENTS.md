# AGENTS.md

Instructions for AI coding agents working in this repository. Humans: see [CONTRIBUTING.md](CONTRIBUTING.md).

## What this repository is

SDKs, one per language, for writing Drift plugins: WebAssembly components that implement the
`drift:plugins` WIT contract. The design is in
[`Documentation/proposals/repository-and-releases.md`](Documentation/proposals/repository-and-releases.md);
**read it before changing the structure, the release process or CI**, and say so in your change if you
depart from it. Once its decisions become ADRs, those take precedence.

This repository is public. The Drift source, `pactflow/drift`, is private. Do not copy code, internal
URLs, ticket links or other private detail from it into files here, commit messages or pull requests.

## Layout and the rules that keep it generic

```
Documentation/    language-neutral guide, proposals, decisions
wit/wit.lock.json the published WIT versions in use, with SHA-256
conformance/      data-values/ vectors and suites/ Drift test cases, shared by every SDK
sdks/<name>/      one SDK: sdk.toml, justfile, CHANGELOG.md, wit/ (vendored), its package(s), examples/
scripts/          language-neutral Python tooling (3.11+, standard library only)
.changeset/       change files, consumed by knope
.github/          actions/setup-<sdk>/, workflows/
knope.toml        one [packages.<name>] per SDK
justfile          delegates only: `just sdk <name> <recipe>`
```

1. **Nothing at the root belongs to a language.** No root `Cargo.toml`, `package.json`, `go.mod`,
   `pyproject.toml`, lockfile or toolchain config. Each SDK directory is the root of its own toolchain.
2. **A change to one SDK touches only that SDK** (plus its entry in `knope.toml`, its setup action and
   its workflow steps when it is new). Do not share code between SDKs; share vectors and suites through
   `conformance/`.
3. **Every SDK provides the same `just` recipes**, and CI and `release.yml` use only these: `check`,
   `test`, `build-examples`, `package`, `publish`, `sync-wit`, plus `version`, which prints the version
   `publish` would release and is how `release.yml` decides what to tag. Do not add steps to a workflow
   that bypass a recipe.
4. **Do not hardcode an SDK's name or toolchain into shared files** (`scripts/`, the root `justfile`,
   `plan` jobs). Discover SDKs from `sdks/*/sdk.toml`. The one unavoidable exception is the
   `if: matrix.sdk == '<name>'` step that selects `.github/actions/setup-<name>`, because `uses:` cannot
   be computed.

## The WIT

- **Never edit a WIT file here.** The source of truth is in Drift. This repository consumes released
  versions.
- `sdks/<name>/wit/` is vendored output. Change it only with `just sdk <name> sync-wit`, which refuses a
  version that is not in `wit/wit.lock.json` or whose SHA-256 does not match. Never hand-edit it or the
  hashes to make a check pass.
- Adding a version to `wit/wit.lock.json` is its own change, with the SHA-256 Drift published
  (`SHA256SUMS` beside the file), never one you computed from a file you did not get from Drift's
  publication.
- Three versions are kept apart: an SDK's, the WIT's and Drift's bundle. Do not conflate them in docs or
  release notes.

## Versions and releases

- SDK versions are set by knope when a release is prepared. **Never edit a version, `CHANGELOG.md` or
  a `.changeset/*.md` by hand to release something**, and never create a tag. A package's version is
  `0.0.0` until its first release.
- A change under `sdks/<name>/` needs a change file in `.changeset/` naming `<name>` (`knope
  document-change`, or write it by hand as in [CONTRIBUTING.md](CONTRIBUTING.md)), unless it is a
  refactor or test, which the pull request marks `no-release`. Choose the bump from the SDK's public
  API: new API is `minor`, a break is `major`, a fix is `patch` (before 1.0, as the SDK's own docs say).
  Documentation, CI and the root `wit/wit.lock.json` need none. The vendored `sdks/<name>/wit/` is under
  `sdks/<name>/`, so changing it **does** need one.
- Before you open a pull request, commit everything, then run `just check-changeset origin/main`; it
  compares committed changes only, and it is what the `changeset` check on the pull request runs. If it
  fails, add the change file (or ask for `no-release`) in the same pull request. Do not open a pull
  request that you know will fail it.
- Do not run `publish`, `release.yml` or `prepare-release.yml`, push tags, or create GitHub releases.
  Publishing is for the `release` workflow in the `release` environment.
- Tags are `<package>/v<version>`; an SDK may set `tag-prefix` in `sdk.toml` (Go will).

## Adding a language

1. `sdks/<name>/` with `sdk.toml` (`name`, `wit`, `status`), a `justfile` with every recipe above,
   `CHANGELOG.md`, `wit/` and the SDK's package and examples. `name` must equal the directory.
2. `.github/actions/setup-<name>/action.yml` installing the toolchain, with a `publish` input that
   performs the registry's trusted-publishing login.
3. An `if: matrix.sdk == '<name>'` step using it in `ci.yml` and `release.yml`.
4. A `[packages.<name>]` entry in `knope.toml` with its `versioned_files` and `changelog`.
5. A row in the README's SDK table. A trusted publisher on the registry is a maintainer's task; say it
   is still needed in your change.

No other SDK's files change.

## Commands

```sh
just                          # list recipes
just sdk rust check           # also: test, build-examples, package, version, sync-wit
just plan origin/main         # the SDKs CI would run for your changes
just check-changeset origin/main   # what changeset.yml will say about your changes
knope --validate              # validate knope.toml and the change files
```

`just sdk <name> package` and `cargo package` refuse an uncommitted tree; that is expected before you
commit.

## Working conventions

- Run the affected SDK's `check` and `test` before you say a change is done, and `knope --validate` if
  you touched `knope.toml` or `.changeset/`. If you could not run something, say so.
- `scripts/` is Python 3.11+ with the standard library only; keep it that way, and keep it testable
  without network or a Drift install.
- Workflows pin actions by major version and request the least `permissions` that work; do not widen
  them, and do not add long-lived secrets. crates.io, npm and PyPI use trusted publishing (OIDC).
- Do not commit credentials, `.env` files or build output (`target/`, `node_modules/`, `dist/`).
- Match the style of the code around you. Do not add dependencies without a reason you can state in
  the pull request.
- Documentation is plain: short sentences, say what is, and mark what is not done yet as not done. Do not
  describe planned behaviour as if it works.
- Proposals in `Documentation/proposals/` are dated and written against a Drift commit. Do not rewrite
  one to match what you built; record a decision as an ADR, or add a note saying how it was resolved.

## Not done yet

So you do not mistake absence for a bug:

- `wit/wit.lock.json` has `drift:plugins@1.0.0` only; a version Drift publishes later is added to it as
  its own change.
- The Rust crate has `DataValue` and its `data-value` conversion, a `Manifest` builder, and the generated
  types for the `data-values` interface only. It has no bindings for the rest of the WIT and no helpers
  for events or plugin config. A plugin runs `wit-bindgen` for its own world; see
  `Documentation/decisions/0002-rust-bindings-and-manifest.md`. There is one example, `text-plugin`, and
  `build-examples` builds each example to `target/plugins/<name>.plugin`. There is no `cargo generate`
  template.
- `conformance/data-values/` has vectors. `conformance/suites/` has none, and the end-to-end CI job that
  runs suites against a pinned Drift bundle is a TODO in `ci.yml`.
- The plugin guide has not been moved here from Drift.
- The first release must be given a version: `prepare-release.yml` with `rust=<version>`.
