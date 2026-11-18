# Contributing

## Working on an SDK

Each SDK is its own project under `sdks/<name>/` with its own toolchain. From the repository root:

```sh
just sdk rust check            # formatting and lint
just sdk rust test             # unit tests and conformance vectors
just sdk rust build-examples   # builds each example to a .plugin
```

You need [just](https://just.systems), Python 3.11+ and the SDK's own toolchain. Add
[knope](https://knope.tech) to write change files.

## Change files

A pull request that changes `sdks/<name>/` records its release intent in a change file in `.changeset/`.
Create one with `knope document-change`, or write it by hand:

```md
---
rust: minor
python: patch
---

# Add a builder for the plugin manifest
```

- The front matter names each package the change moves, and by how much (`major`, `minor`, `patch`).
- The first heading is the changelog entry. Anything below it is the body; put the ticket there, if there
  is one.
- A pull request with no release impact (a refactor, a test) gets the `no-release` label instead.
- Documentation, CI and `wit/` changes need no change file.

## The WIT

The WIT is not edited here. Drift owns `Plugins/plugin-manager/wit/plugin.wit` and publishes each
version; this repository records the versions it uses in `wit/wit.lock.json` and each SDK vendors the
one it targets with `just sdk <name> sync-wit`.

## Releases

Maintainers release. `prepare-release.yml` (run by hand) pushes a `release` branch with the versions and
changelogs; merging its pull request runs `release.yml`, which tags, publishes and creates the GitHub
release for each SDK whose version has no tag. Tags are `<package>/v<version>`, for example `rust/v0.3.0`.
