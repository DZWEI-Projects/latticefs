#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require_lfs_bin

tmp="$(mktemp -d)"
export XDG_CONFIG_HOME="$tmp/config"
repo="$tmp/repo"
mkdir -p "$repo"

printf "alpha\n" > "$tmp/a.txt"
printf "beta\n" > "$tmp/b.txt"

"$LFS_BIN" --repo "$repo" init
output_a="$("$LFS_BIN" --repo "$repo" add "$tmp/a.txt" --tag project:demo)"
object_a="$(echo "$output_a" | awk '{print $3}')"
"$LFS_BIN" --repo "$repo" add "$tmp/b.txt" --tag project:other"

"$LFS_BIN" --repo "$repo" view create Demo --query "tag:project:demo"
views="$("$LFS_BIN" --repo "$repo" view list)"
echo "$views" | grep -q "Demo: tag:project:demo"

"$LFS_BIN" --repo "$repo" view explain "$object_a" --view Demo >/dev/null
