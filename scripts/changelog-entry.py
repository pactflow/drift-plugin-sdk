#!/usr/bin/env python3
"""Print one version's entry from a knope changelog, for a GitHub release's notes.

    changelog-entry.py sdks/rust/CHANGELOG.md 0.3.0
"""

import re
import sys

from sdks import fail


def main() -> None:
    if len(sys.argv) != 3:
        fail("usage: changelog-entry.py <changelog> <version>")
    path, version = sys.argv[1:]
    with open(path) as f:
        text = f.read()
    match = re.search(rf"^## {re.escape(version)}\b.*?$(.*?)(?=^## |\Z)", text, re.DOTALL | re.MULTILINE)
    if not match:
        fail(f"{path} has no entry for {version}")
    print(match.group(1).strip())


if __name__ == "__main__":
    main()
