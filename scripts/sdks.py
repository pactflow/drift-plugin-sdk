"""Helpers shared by the scripts in this directory. Python 3.11+, standard library only."""

import json
import os
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SDKS_DIR = ROOT / "sdks"
STATUSES = ("experimental", "beta", "stable")


def load_sdks() -> dict[str, dict]:
    """Every SDK, by name, from sdks/*/sdk.toml. The `dir` key is added."""
    sdks = {}
    for path in sorted(SDKS_DIR.glob("*/sdk.toml")):
        config = tomllib.loads(path.read_text())
        name = config.get("name")
        if name != path.parent.name:
            fail(f"{path.relative_to(ROOT)}: name {name!r} must equal the directory name {path.parent.name!r}")
        if config.get("status") not in STATUSES:
            fail(f"{path.relative_to(ROOT)}: status must be one of {', '.join(STATUSES)}")
        if not config.get("wit"):
            fail(f"{path.relative_to(ROOT)}: wit is required")
        config["dir"] = path.parent
        sdks[name] = config
    return sdks


def changed_files(base: str, diff_filter: str | None = None) -> list[str]:
    """Paths changed between `base` and HEAD, relative to the repository root."""
    cmd = ["git", "diff", "--name-only"]
    if diff_filter:
        cmd.append(f"--diff-filter={diff_filter}")
    cmd.append(f"{base}...HEAD")
    out = subprocess.run(cmd, cwd=ROOT, check=True, capture_output=True, text=True).stdout
    return [line for line in out.splitlines() if line]


def write_output(key: str, value: object) -> None:
    """Print a result for a person and, in GitHub Actions, set it as a step output."""
    text = value if isinstance(value, str) else json.dumps(value)
    print(f"{key}={text}")
    if "GITHUB_OUTPUT" in os.environ:
        with open(os.environ["GITHUB_OUTPUT"], "a") as f:
            f.write(f"{key}={text}\n")


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    sys.exit(1)
