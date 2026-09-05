#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
TEST_ROOT="$(mktemp -d /tmp/mambosite-install-test.XXXXXX)"

cleanup() {
    if [[ -d "$TEST_ROOT" && "$TEST_ROOT" == /tmp/mambosite-install-test.* ]]; then
        rm -rf -- "$TEST_ROOT"
    fi
}
trap cleanup EXIT

mkdir -p "$TEST_ROOT/bin"
MAMBOSITE_BIN_DIR="$TEST_ROOT/bin" "$SCRIPT_DIR/install.sh" >/dev/null

[[ -L "$TEST_ROOT/bin/mbsite" ]]
[[ -L "$TEST_ROOT/bin/mambosite" ]]
[[ "$(readlink -f "$TEST_ROOT/bin/mbsite")" == "$SCRIPT_DIR/mambosite.sh" ]]
[[ "$(readlink -f "$TEST_ROOT/bin/mambosite")" == "$SCRIPT_DIR/mambosite.sh" ]]

mkdir -p "$TEST_ROOT/conflict"
printf 'keep\n' > "$TEST_ROOT/conflict/mbsite"
if MAMBOSITE_BIN_DIR="$TEST_ROOT/conflict" "$SCRIPT_DIR/install.sh" >/dev/null 2>&1; then
    echo "installer should refuse a non-symlink command target" >&2
    exit 1
fi
[[ "$(cat "$TEST_ROOT/conflict/mbsite")" == "keep" ]]
[[ ! -e "$TEST_ROOT/conflict/mambosite" ]]

echo "MamboSite installer checks passed"
