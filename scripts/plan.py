#!/usr/bin/env python3
"""Decide which SDKs CI runs. Sets the outputs `matrix` (for a job's strategy) and `any`.

    plan.py --base origin/main

An SDK runs when something under sdks/<name>/ changed. A change to wit/, conformance/, .github/,
scripts/ or the root justfile runs every SDK. Without --base, or with --all, every SDK runs.
"""

import argparse

from sdks import changed_files, load_sdks, write_output

SHARED = ("wit/", "conformance/", ".github/", "scripts/")
SHARED_FILES = ("justfile",)


def plan(names: list[str], changed: list[str] | None) -> list[str]:
    if changed is None or any(p.startswith(SHARED) or p in SHARED_FILES for p in changed):
        return names
    return [n for n in names if any(p.startswith(f"sdks/{n}/") for p in changed)]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--base", help="a git ref to compare HEAD with")
    parser.add_argument("--all", action="store_true", help="run every SDK")
    args = parser.parse_args()

    # A new branch's push event has an all-zero `before`; there is nothing to compare with.
    base = None if args.all or not args.base or set(args.base) == {"0"} else args.base
    names = plan(list(load_sdks()), changed_files(base) if base else None)
    write_output("matrix", {"include": [{"sdk": n} for n in names]})
    write_output("any", "true" if names else "false")


if __name__ == "__main__":
    main()
