# Language-neutral. Everything an SDK does is in its own justfile; this one only delegates.

set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# run one of an SDK's recipes: `just sdk rust test`
sdk name +recipe:
    just --justfile sdks/{{name}}/justfile --working-directory sdks/{{name}} {{recipe}}

# the SDKs CI would run for the changes since <base>: `just plan origin/main`
plan base="":
    python3 scripts/plan.py {{ if base == "" { "--all" } else { "--base " + base } }}

# fail if the change files, or the changes without one, are wrong: `just check-changeset origin/main`
check-changeset base labels="":
    python3 scripts/check-changeset.py --base {{base}} --labels "{{labels}}"
