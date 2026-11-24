#!/usr/bin/env python3
"""Turn prepare-release.yml's `versions` input into the arguments for `knope prepare-release`.

    override-versions.py "rust=0.1.0 python=0.2.0"

Prints the arguments on one line. knope wants `<package>=<version>` when knope.toml has
several packages and the bare version when it has one, so this reads the packages from knope.toml and
gives each case what it wants. A package that knope.toml does not have, a version that is not
`major.minor.patch` (with an optional pre-release), or a package given twice is an error, here, rather
than knope's less clear one.
"""

import re
import sys
import tomllib

from sdks import ROOT, fail

VERSION = re.compile(r"\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?")


def main() -> None:
    if len(sys.argv) != 2:
        fail('usage: override-versions.py "<package>=<version> ..."')
    packages = list(tomllib.loads((ROOT / "knope.toml").read_text()).get("packages", {}))

    versions: dict[str, str] = {}
    for item in sys.argv[1].split():
        name, sep, version = item.partition("=")
        if not sep:
            fail(f"{item!r} must be <package>=<version>, for example {packages[0]}=0.1.0")
        if name not in packages:
            fail(f"{name!r} is not a package in knope.toml ({', '.join(packages)})")
        if name in versions:
            fail(f"{name!r} is given twice")
        if not VERSION.fullmatch(version):
            fail(f"{version!r} is not a version like 0.1.0")
        versions[name] = version

    # `--override-version 0.1.0` is an error with several packages, and `rust=0.1.0` is one with a single.
    print(" ".join(
        f"--override-version {version if len(packages) == 1 else f'{name}={version}'}"
        for name, version in versions.items()
    ))


if __name__ == "__main__":
    main()
