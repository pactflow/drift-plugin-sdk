# Releasing

For maintainers. The design is in [the proposal](proposals/repository-and-releases.md#5-releases), and
[0003](decisions/0003-release-process.md) records where the process departs from it.

Each SDK is released on its own, to its own registry, at its own version. Nobody edits a version, a
changelog or a tag by hand: knope sets the versions from the change files in `.changeset/`, and the
`release` workflow publishes and tags.

## One-time setup

Not done yet for the Rust SDK. Until it is, `release.yml` cannot publish.

1. **The `release` environment.** In the repository's settings, create the environment `release`. Require
   a reviewer, and allow deployments from `main` only. The workflow's publish job is bound to it, and
   crates.io checks its name.
2. **Claim the crate name.** crates.io has no pending trusted publisher: a trusted publisher can be added
   only to a crate that exists, and the first version of a crate cannot be published by a workflow. So a
   maintainer publishes once by hand, with a personal API token that has the `publish-new` scope:

   ```sh
   git switch main && git pull           # a clean tree
   cd sdks/rust
   cargo publish -p drift-plugin-sdk --locked
   ```

   The version is `0.0.0`, which is what the tree holds until the first release. Revoke the token
   afterwards. `cargo yank --version 0.0.0 drift-plugin-sdk` hides it from new projects, if you prefer.
   The workflow skips a version `0.0.0`, so this does not clash with the first real release.
3. **The trusted publisher.** On crates.io, in the crate's settings under Trusted Publishing, add a
   GitHub publisher: owner `pactflow`, repository `drift-plugin-sdk`, workflow `release.yml`, environment
   `release`. Once it works you can turn on "Require trusted publishing", which stops tokens publishing
   the crate at all.

A language added later has its own version of this. Check whether its registry takes a trusted publisher
before the package exists, and say in the change that adds the SDK what is still needed.

## Each release

1. In Actions, run **Prepare release** on `main`. It reads the change files and pushes a `release` branch
   with the versions, the changelogs and the change files removed.
   - The first release of an SDK is a decision, not a sum, because the version is `0.0.0`. Give it in
     the `versions` input, as `rust=0.1.0`. Leave `versions` empty afterwards.
2. Open a pull request from `release` to `main`. Read the changelog entry; it becomes the GitHub
   release's notes. When CI passes, merge it.
3. Merging runs **Release**. Its `plan` job finds each SDK whose version has no tag. For each, the
   `release` job waits for a reviewer to approve the `release` environment, publishes, pushes the tag
   `<package>/v<version>` (`rust/v0.1.0`), and creates the GitHub release.

## When it goes wrong

- **The publish failed.** There is no tag, so nothing is lost: fix the cause and re-run the failed
  job only ("Re-run failed jobs"), so that an SDK that has published is not published again.
- **It published but the tag failed.** The registry has the version and the repository does not know.
  Tag the release commit by hand (`git tag -a rust/v0.1.0 -m "rust 0.1.0" <commit>`, then push it), and
  create the GitHub release from the changelog entry (`scripts/changelog-entry.py`). A version
  cannot be published twice, so re-running the job would fail.
- **The release branch has the wrong versions.** Close its pull request and run Prepare release again. It
  force-pushes `release`.
- **A change file was wrong after it merged.** Fix it with a new pull request before running Prepare
  release; the change files are the source of the changelog.

To see what a release would do without making one, from a clean checkout: `knope prepare-release
--dry-run --override-version 0.1.0` for one package (with several, `<package>=<version>`).
