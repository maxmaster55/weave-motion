#!/bin/sh
# Portable build helper.
#
#   ./scripts/build.sh              # build the portable core + demos
#   ./scripts/build.sh --release    # optimized build
#   ./scripts/build.sh -p weave-web # build a specific package
#
# The script only relies on POSIX sh + cargo, so it behaves identically on
# Linux, macOS and Windows (via git-bash / WSL).
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

cd "$repo_root"

exec cargo build --workspace "$@"
