# 0003. How the release process differs from the proposal

Status: accepted, 2026-11-24.

It departs from §5 of [the proposal](../proposals/repository-and-releases.md#5-releases) in three
places. They were found by running the release steps on a copy of the repository, as a release would,
before any release was made. [Releasing](../releasing.md) is the procedure.

## Decisions

1. **`Cargo.lock` is a versioned file of the `rust` package.** knope bumped only the crate's
   `Cargo.toml`, leaving `Cargo.lock` at the old version. `cargo package --locked`, which CI runs and
   `publish` is, then fails on the release pull request: the lock file needs updating. knope can bump
   the crate's entry in the lock file, so it does, in the same commit. The lock file is committed
   because the example plugins are built from it.
2. **`release.yml` publishes before it tags.** The proposal tags first. A tag is how the plan job knows
   a version is released, so a publish that fails after the tag leaves a version that looks released
   and is never tried again, with the tag to be deleted by hand. The first run is the likeliest to fail
   (an environment or trusted publisher mis-set). The other order fails the other way: published, tag
   not made. That is rarer, a publish cannot be repeated, and the fix is to push the tag; it is in
   [Releasing](../releasing.md#when-it-goes-wrong). How an SDK whose release is its tag (Go)
   fits is for when it is added.
3. **`prepare-release.yml` builds knope's override arguments with `scripts/override-versions.py`.**
   The workflow's input is `<package>=<version>`. knope accepts that only when `knope.toml` has more
   than one package, and wants the bare version when it has one, so the workflow as proposed failed
   for the one SDK there is and would have worked only once a second was added. The script reads the
   packages from `knope.toml`, so it needs no change when one is added, and rejects a name or version
   that does not make sense with a message that says which.

## Not decided here

crates.io has no pending trusted publisher: the first version of a crate is published by a person with
a token, and only then can the workflow be made its publisher. The workflow does not try to do it. It
is a maintainer's step, with the steps in [Releasing](../releasing.md#one-time-setup).
