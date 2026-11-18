# Organising, releasing and versioning the Drift plugin SDKs

A proposal. It covers where the plugin WIT lives and how it is published and versioned, how this
repository is laid out so that a new language touches nothing that already exists, and how its
packages are released. Once it is agreed, its decisions become ADRs.

It was written on 2026-11-17 against Drift `b5683c4`.

## Contents

1. [What an external plugin author is missing](#1-what-an-external-plugin-author-is-missing)
2. [The WIT stays in Drift and Drift publishes it](#2-the-wit-stays-in-drift-and-drift-publishes-it)
3. [Versioning the WIT](#3-versioning-the-wit)
4. [Repository layout](#4-repository-layout)
5. [Releases](#5-releases)
6. [What goes in an SDK](#6-what-goes-in-an-sdk)
7. [Order of work](#7-order-of-work)
8. [Open questions](#8-open-questions)

## 1. What an external plugin author is missing

- **The contract.** It is `Plugins/plugin-manager/wit/plugin.wit` in the private `pactflow/drift`:
  one file, package `drift:plugins` with no version. Drift's plugins reference it by relative path
  (`Plugins/reports-plugin/src/bindings.rs`). It has evolved by adding interfaces with a fallback in
  the host (`operations-v2` beside `operations`), and the manifest's `interface-version` is `0` in
  every plugin and nothing checks it.
- **Help with `data-value`.** WIT has no recursive types, so values are flattened into a node list
  addressed by index. The conversion, `pack_unpack.rs`, is copied into 17 of Drift's plugins, about
  13,000 lines in all, and converts to `drift_models::DataValue`, which comes from the private
  `pactflow` registry along with the other `Shared/` crates.
- **Documentation.** `Documentation/Plugins.md` is good, but it is in the private repository and links
  to the WIT there.

So the minimum is: the WIT at a stable public URL with a version, a library per language that does
the `data-value` work and the boilerplate, and a guide that points at public things.

## 2. The WIT stays in Drift and Drift publishes it

The source of truth stays `Plugins/plugin-manager/wit/plugin.wit`. This repository consumes released
versions of it; it never edits it.

The alternative, moving the WIT here and having Drift pin it, keeps the contract reviewed in public
and lets a WIT change and its SDK change land in one pull request. It is not proposed because the WIT
only means something once the host implements it: every change would be a PR here, a release here,
then a PR in Drift, and a WIT could be published that no released Drift supports. With the WIT
published by Drift's release, a published version is always one that a released host implements.

Drift's release pipeline publishes it beside the bundle:

```
s3://download.pactflow.io/drift/wit/
  index.json                    every published version, its SHA-256, and the first bundle
                                version whose host implements it
  drift-plugins/1.0.0/
    plugin.wit                  the text, as plugin-manager has it
    drift-plugins.wasm          the same package binary-encoded (`wasm-tools component wit --wasm`),
                                which wkg, jco and componentize-py read
    SHA256SUMS
```

- A version is immutable: the step fails if its prefix exists, as the bundle step does. There is no
  `latest/`; a WIT is pinned, never floated, and `index.json` is how a tool finds the newest.
- The WIT gets its own tag in Drift, `drift-plugins-wit-v<version>`, and its own version, separate
  from the bundle's (§3).
- The bundle's release record (`Bundles/releases/<version>.json`) and `components.json` record the
  WIT version the host implements, and `drift --version` prints it. This repository's compatibility
  table is generated from those, not written by hand.
- `plugin.wit` gets an SPDX licence header, since it will be distributed outside the repository it
  lives in.
- Later, the same package can be pushed to an OCI registry (`ghcr.io/pactflow`) with `wkg publish`,
  so `wkg get drift:plugins@1.0.0` works. S3 is enough to start.

**What Drift's release tooling needs.** The WIT is a releasable thing that is not a crate, and
`prepare-release.rb` finds components through `cargo metadata`. It also has its own bump, which is
not the bundle's: a new interface is a bundle `patch` but a WIT `minor`. Proposed: changesets gain an
optional `wit: none | patch | minor | major`, which `changeset-validate.rb` requires when `affects`
names a path under `Plugins/plugin-manager/wit/`, and `prepare-release` releases the WIT when any
consumed changeset has one.

## 3. Versioning the WIT

The package gets a semver version, `package drift:plugins@1.0.0;`. The bundle's `YYMM.minor.patch`
says when something shipped, not what is compatible, so the WIT is versioned on its own.

| Change | Bump |
|---|---|
| Comments only | patch (published, so the guide's references match) |
| A new interface, or a new world | minor |
| A new function in an interface the host provides (`logger`, `config`, `http`, …) | minor: a plugin that calls it needs a host of at least that minor |
| A new function in an interface a plugin exports | minor, only if the host falls back when it is absent, as it does for `operations-v2` |
| A new record field, variant case or enum case; any change to a signature; removing anything | major |

The last row is the one that surprises: component-model types are structural and exact, so adding a
field to a record is a breaking change, unlike adding one to JSON. Where a type must stay open it
should say so in its shape, as `capability-type`'s `OTHER(u16)` and the `metadata` lists already do.

**The first version.** Proposed: `1.0.0`. Seventeen plugins already run on this contract, its
evolution so far has been additive, and in the component model a `0.x` line is compatible only
within one minor (wasmtime's linker matches `0.2.1` with `0.2.5`, not with `0.3.0`), so `0.x` would
make every minor a break. §8 asks this.

**Adding the version renames everything.** With a version, the plugin's imports and exports are
named `drift:plugins/logger@1.0.0` and so on. The host links imports and finds exports by the
unversioned names (`Plugins/plugin-manager/src/plugins.rs:681` looks up
`"drift:plugins/plugin-manifest"`), and every plugin in the bundle and every `.plugin` on a user's
disk uses them. So before the first versioned plugin exists, the host must accept both: link imports
under both names, and look exports up by either. wasmtime's `Linker` resolves semver-compatible
imports; whether `get_export_index` does the same for exports is the first thing to spike. This is
the one Drift host change the public release waits for.

**`interface-version`.** It becomes the WIT major, set by the SDK from the WIT it vendors, and the
host refuses a plugin whose major it does not implement with a message naming both, instead of
failing later at a typed-function lookup. `0`, what every plugin reports now, means the unversioned
WIT and is treated as `1`.

## 4. Repository layout

The rule that keeps it generic: **nothing at the root belongs to a language.** There is no root
`Cargo.toml`, `package.json` or `go.mod`. Each SDK is a directory its own toolchain treats as the
project root, and the root holds only what is language-neutral.

```
drift-plugin-sdk/
  README.md                the SDK table and the compatibility table (WIT version → Drift bundle)
  Documentation/           the language-neutral plugin guide (Drift's Plugins.md, moved here),
                           proposals and decisions
  wit/
    wit.lock.json          the published WIT versions this repository uses, with their SHA-256
  conformance/
    data-values/           vectors: a value and its WIT `data-value` encoding, both directions
    suites/                Drift test cases each SDK's example plugins are run under
  sdks/
    rust/
      sdk.toml             what CI needs to know about this SDK
      justfile             the recipes every SDK provides (below)
      wit/                 the WIT this SDK targets, vendored by `just sync-wit`
      CHANGELOG.md
      drift-plugin-sdk/    the crate
      examples/
    typescript/            the same shape, with its own toolchain
    python/
    go/
  .changeset/
  knope.toml
  justfile                 delegates: `just sdk rust test`
  .github/
    actions/setup-<sdk>/   one composite action per SDK that installs its toolchain
    workflows/             ci.yml, changeset.yml, prepare-release.yml, release.yml
```

Every SDK provides the same `just` recipes, which is the whole interface CI and the release workflow
use:

| Recipe | Does |
|---|---|
| `check` | formatting and lint |
| `test` | unit tests, and the `conformance/data-values` vectors |
| `build-examples` | builds each example to `<name>.plugin` in `target/plugins/` |
| `package` | builds what would be published, without publishing it |
| `publish` | publishes; only `release.yml` runs it |
| `sync-wit` | vendors the WIT version `sdk.toml` names, refusing one whose SHA-256 is not in `wit/wit.lock.json` |

and a small `sdk.toml`:

```toml
name = "rust"          # the package name in change files and knope.toml
wit = "1.0.0"          # the WIT this SDK targets
status = "beta"        # experimental | beta | stable, shown in the README's table
```

**Why each SDK vendors its own WIT.** A published package has to contain it: `cargo publish`, npm and
PyPI cannot reach `../../wit`. It also lets SDKs move to a new WIT at their own pace; the Rust SDK can
target `1.3` while Python is on `1.2`. CI fails if a vendored copy differs from the published file.

**CI.** One workflow. A `plan` job reads `sdks/*/sdk.toml` and the changed paths and emits a matrix of
the SDKs to run; a change under `wit/`, `conformance/` or `.github/` runs them all. Each matrix job
uses `.github/actions/setup-<sdk>` and runs the recipes. The end-to-end job downloads a Drift bundle
pinned by version and checked against its `SHA256SUMS`, installs the example plugins into its plugin
directory, and runs `conformance/suites`: the only test that proves an SDK's output loads in Drift. A
nightly run against `latest/` catches a host that breaks a published WIT before users do.

**Adding a language** is: `sdks/<name>/` with its `sdk.toml`, `justfile` and `CHANGELOG.md`;
`.github/actions/setup-<name>/`; a `[packages.<name>]` entry in `knope.toml`; and a trusted publisher
on its registry. No other SDK's files change.

## 5. Releases

The idea is Drift's: each PR records its release intent, a release PR turns the gathered intent into
versions and changelogs, and tags are made after it merges. The difference is that nothing here is
bundled. Each SDK is released on its own, to its own registry, at its own version, so the tool is
[knope](https://knope.tech), as Janus uses it (`pact-janus/knope.toml`, with an engine and two SDKs,
including a `gradle.properties` version read by regex).

**Change files** are knope's, in `.changeset/`:

```md
---
rust: minor
python: patch
---

# Add a builder for the plugin manifest
```

Compared with Drift's:

- They name each package they move and by how much, instead of one `bundle:` level.
- No `affects:`; the packages are named. `changeset.yml` fails a PR that changes `sdks/<name>/`
  without a change file naming `<name>`, unless it has the `no-release` label (a refactor, a test).
  Documentation, CI and `wit/` changes need no change file.
- No required `jira:`, because outside contributors have no JIRA. A maintainer puts the ticket in
  the body.

**The flow**, as Janus runs it:

1. A PR adds its change file (`knope document-change`).
2. `prepare-release.yml`, run by hand, runs `knope prepare-release` and pushes a `release` branch; a
   maintainer opens the PR from it, so CI runs on it. A first release, or any version that is a
   decision rather than a sum, is given as `<package>=<version>`.
3. Merging it runs `release.yml`: for each package whose version has no tag, it tags, runs the SDK's
   `just publish` in a job bound to a `release` environment, and makes a GitHub release with the
   changelog entry.

**Tags** are `<package>/v<version>`, `rust/v0.3.0`. Go is the exception that shapes the rule: a module
in a subdirectory resolves only from a tag named for its path, `sdks/go/v0.3.0`, and from major 2 its
module path needs a `/v2` suffix. So the tag format is per package, and the Go one is its directory.
Whether knope makes Go's tag itself, at the version we pin, is to check; if not, `release.yml` does.

**Publishing credentials.** crates.io, npm and PyPI all take trusted publishing from GitHub Actions
(OIDC), so no long-lived token is stored for them. npm provenance and PyPI attestations come with it.
Maven Central, if a JVM SDK comes, still needs a token and a signing key in the `release` environment.

**Three versions, kept apart.** An SDK's version, the WIT's and Drift's bundle version say different
things. Each SDK's README and release notes name the WIT it targets, and the root README maps WIT
versions to the first Drift bundle that implements them.

## 6. What goes in an SDK

Rust first, because every existing plugin is Rust and they are the test of whether it is complete:

- bindings generated from the vendored WIT, re-exported so a plugin does not run `wit-bindgen` itself;
- a value type with the `data-value` conversion, the code 17 plugins copy, tested against
  `conformance/data-values`;
- a manifest builder that fills in `interface-version`, and helpers for events and plugin config;
- a `cargo generate` template, examples, and a recipe that builds a `.plugin`.

It does not re-export `drift-models`. Publishing that crate would make Drift's test-case model, used
at 101 import sites in its plugins, a public API. When Drift's own plugins can move onto the public
crate, the 17 copies go, and the SDK has proved itself on real plugins.

## 7. Order of work

1. **Drift**: the spike on versioned export names; the host accepting versioned and unversioned names;
   `drift:plugins@1.0.0`; the `wit:` changeset field; the release step that publishes the WIT and
   records its version in the bundle.
2. **Here**: the layout, `knope.toml`, the CI `plan` job, `changeset.yml`, and the plugin guide moved
   out of Drift.
3. The Rust SDK, one example, and the end-to-end suite against a pinned Drift.
4. Further languages, each as §4's list.

## 8. Open questions

1. **`1.0.0` or `0.1.0`** for the first published WIT (§3).
2. **Worlds per kind of plugin.** Today's `plugin` world exports everything, and each plugin writes a
   world of its own. Publishing worlds for the common kinds (a data format, an operation file, an
   event listener) would make an SDK simpler to use. New worlds are a minor change, so this need not
   wait, but naming them before `1.0.0` is cheaper.
3. **The Rust value type.** Its own, as §6 proposes, or a slimmed public `drift-models`?
4. **Where the guide is published**: GitHub Pages from this repository, or PactFlow's docs site.
5. **Which languages after Rust**, and which PactFlow supports rather than accepts from the community.
   `sdk.toml`'s `status` is where that shows.
6. **Installing a third-party plugin.** `drift component install` installs only what PactFlow's
   `components.json` vouches for. Is a third-party plugin a file dropped into `~/.drift/plugins`, or
   does `drift component` learn to install from another manifest? Not this repository's decision,
   but its first users will ask.
7. **JIRA** in change files: optional in the body, as proposed, or required for maintainers' PRs?
