#!/usr/bin/env python3
"""Fail a pull request that changes an SDK without a change file naming it.

    check-changeset.py --base origin/main --labels "no-release,docs"

Every change file in .changeset/ must also parse and name only SDKs that exist. A pull request with the
`no-release` label (a refactor, a test) needs no change file for its SDK changes.
"""

import argparse
import re
from pathlib import Path

from sdks import ROOT, changed_files, fail, load_sdks

LEVELS = ("major", "minor", "patch")


def parse_change_file(text: str) -> dict[str, str]:
    """The `package: level` pairs of a change file's front matter."""
    match = re.match(r"---\n(.*?)\n---\n", text, re.DOTALL)
    if not match:
        raise ValueError("missing front matter")
    packages = {}
    for line in match.group(1).splitlines():
        if not line.strip():
            continue
        name, sep, level = (part.strip() for part in line.partition(":"))
        if not sep or level not in LEVELS:
            raise ValueError(f"expected `<package>: major|minor|patch`, found {line!r}")
        packages[name] = level
    if not packages:
        raise ValueError("names no package")
    return packages


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--base", required=True, help="a git ref to compare HEAD with")
    parser.add_argument("--labels", default="", help="the pull request's labels, comma separated")
    args = parser.parse_args()

    known = set(load_sdks())
    named: set[str] = set()
    for path in changed_files(args.base, "AM"):
        if not (path.startswith(".changeset/") and path.endswith(".md")):
            continue
        try:
            packages = parse_change_file((ROOT / path).read_text())
        except ValueError as e:
            fail(f"{path}: {e}")
        unknown = set(packages) - known
        if unknown:
            fail(f"{path}: unknown package {', '.join(sorted(unknown))}; known: {', '.join(sorted(known))}")
        named |= set(packages)

    if "no-release" in {label.strip() for label in args.labels.split(",")}:
        print("labelled no-release: no change file needed")
        return

    touched = {n for n in known for p in changed_files(args.base) if p.startswith(f"sdks/{n}/")}
    missing = sorted(touched - named)
    if missing:
        fail(
            f"changes under sdks/{', sdks/'.join(missing)} need a change file naming "
            f"{', '.join(missing)} (run `knope document-change`), or the no-release label"
        )
    print("change files are in order")


if __name__ == "__main__":
    main()
