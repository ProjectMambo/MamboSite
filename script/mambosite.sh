#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

exec cargo run --quiet \
    --manifest-path "$PROJECT_DIR/Cargo.toml" \
    --package mambosite-cli \
    -- "$@"
