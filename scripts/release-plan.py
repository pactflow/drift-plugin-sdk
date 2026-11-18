#!/usr/bin/env python3
"""List the SDKs whose version has no tag yet. Sets the outputs `matrix` and `any`.

    release-plan.py

The version is what `just version` prints in the SDK's directory. The tag is
`<tag-prefix>v<version>`, where `tag-prefix` is in sdk.toml and defaults to `<name>/`, so `rust/v0.3.0`.
Go sets it to its directory (`sdks/go/`), because a module in a subdirectory resolves only from a tag
named for its path. Version 0.0.0 is an SDK that has never been released, and is skipped.
"""

import subprocess

from sdks import ROOT, load_sdks, write_output


def main() -> None:
    include = []
    for name, config in load_sdks().items():
        version = subprocess.run(
            ["just", "--justfile", "justfile", "version"],
            cwd=config["dir"], check=True, capture_output=True, text=True,
        ).stdout.strip()
        if version == "0.0.0":
            continue
        tag = f"{config.get('tag-prefix', f'{name}/')}v{version}"
        exists = subprocess.run(
            ["git", "rev-parse", "--verify", "--quiet", f"refs/tags/{tag}"],
            cwd=ROOT, capture_output=True,
        ).returncode == 0
        if not exists:
            include.append({"sdk": name, "version": version, "tag": tag})
    write_output("matrix", {"include": include})
    write_output("any", "true" if include else "false")


if __name__ == "__main__":
    main()
