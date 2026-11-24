#!/usr/bin/env python3
"""Turn prepare-release.yml's `versions` input into the arguments for `knope prepare-release`.

    override-versions.py "rust=0.1.0 python=0.2.0"

Prints the arguments on one line. knope wants `<package>=<version>` when knope.toml has
several packages and the bare version when it has one, so this reads the packages from knope.toml and
gives each case what it wants. These are errors here, rather than knope's less clear ones: a package that
knope.toml does not have, a package given twice, and a version that is not `major.minor.patch` or
`major.minor.patch-<label>.<number>` with no leading zeroes (`1.0.0-rc.1`; knope takes no other
pre-release).
"""

import re
import sys
import tomllib

from sdks import ROOT, fail

# `major.minor.patch`, or with a pre-release as `-<label>.<number>` (`1.0.0-rc.1`), which is the only
# pre-release knope takes. SemVer's rules hold: no leading zeroes. knope itself accepts `01.2.3`.
_NUMBER = r"0|[1-9]\d*"
VERSION = re.compile(rf"({_NUMBER})\.({_NUMBER})\.({_NUMBER})(-[a-zA-Z][0-9a-zA-Z-]*\.({_NUMBER}))?")


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
