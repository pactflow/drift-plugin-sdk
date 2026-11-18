#!/usr/bin/env python3
"""Vendor the WIT version an SDK targets into <sdk>/wit/.

    sync-wit.py sdks/rust

Reads `wit` from <sdk>/sdk.toml, finds that version in wit/wit.lock.json and refuses to continue if it
is not there. Downloads the file from where Drift publishes it and refuses it if its SHA-256 differs
from the lock. Set DRIFT_WIT_BASE_URL to read from somewhere else.

With --check nothing is downloaded: it fails if the vendored file is missing or differs from the lock.
"""

import argparse
import hashlib
import json
import os
import tomllib
import urllib.request
from pathlib import Path

from sdks import ROOT, fail

PACKAGE = "drift-plugins"  # the directory under the base URL: package drift:plugins
FILE = "plugin.wit"
DEFAULT_BASE_URL = "https://download.pactflow.io/drift/wit"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("sdk_dir", type=Path)
    parser.add_argument("--check", action="store_true", help="verify the vendored copy against the lock; no network")
    args = parser.parse_args()

    sdk_dir = args.sdk_dir.resolve()
    version = tomllib.loads((sdk_dir / "sdk.toml").read_text())["wit"]
    lock = json.loads((ROOT / "wit" / "wit.lock.json").read_text())
    entry = lock["versions"].get(version)
    if entry is None:
        known = ", ".join(lock["versions"]) or "none"
        fail(
            f"drift:plugins@{version} is not in wit/wit.lock.json (known: {known}). "
            "Add it, with the SHA-256 Drift published, in its own change."
        )
    expected = entry[FILE]
    target = sdk_dir / "wit" / FILE

    if args.check:
        if not target.exists():
            fail(f"{target.relative_to(ROOT)} is missing; run `just sdk {sdk_dir.name} sync-wit`")
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        if actual != expected:
            fail(f"{target.relative_to(ROOT)} has SHA-256 {actual}, the lock has {expected} for {version}")
        print(f"{target.relative_to(ROOT)} matches drift:plugins@{version}")
        return

    base = os.environ.get("DRIFT_WIT_BASE_URL", DEFAULT_BASE_URL).rstrip("/")
    url = f"{base}/{PACKAGE}/{version}/{FILE}"
    with urllib.request.urlopen(url, timeout=60) as response:
        data = response.read()
    actual = hashlib.sha256(data).hexdigest()
    if actual != expected:
        fail(f"{url} has SHA-256 {actual}, the lock has {expected}; not vendoring it")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    print(f"vendored drift:plugins@{version} to {target.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
