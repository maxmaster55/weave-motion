#!/bin/sh
# Portable test helper: runs the whole workspace test suite.
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

cd "$repo_root"

exec cargo test --workspace "$@"
