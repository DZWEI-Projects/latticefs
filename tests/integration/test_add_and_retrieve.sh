#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require_lfs_bin

tmp="$(mktemp -d)"
export XDG_CONFIG_HOME="$tmp/config"
repo="$tmp/repo"
mkdir -p "$repo"

printf "hello latticefs\n" > "$tmp/hello.txt"

"$LFS_BIN" --repo "$repo" init
output="$("$LFS_BIN" --repo "$repo" add "$tmp/hello.txt" --tag project:demo)"
object_id="$(echo "$output" | awk '{print $3}')"

"$LFS_BIN" --repo "$repo" get "$object_id" --output "$tmp/out.txt"
diff -u "$tmp/hello.txt" "$tmp/out.txt"
