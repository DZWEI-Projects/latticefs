#!/usr/bin/env bash
set -euo pipefail

: "${LFS_BIN:?LFS_BIN not set}"

tmp="$(mktemp -d)"
export XDG_CONFIG_HOME="$tmp/config"
repo="$tmp/repo"
mkdir -p "$repo"

printf "v1\n" > "$tmp/data.txt"
printf "v2\n" > "$tmp/data_v2.txt"

"$LFS_BIN" --repo "$repo" init
output="$("$LFS_BIN" --repo "$repo" add "$tmp/data.txt")"
object_id="$(echo "$output" | awk '{print $3}')"

"$LFS_BIN" --repo "$repo" revise "$object_id" "$tmp/data_v2.txt" --message "update"
versions="$("$LFS_BIN" --repo "$repo" versions "$object_id")"
count="$(echo "$versions" | grep -c '^v')"

if [[ "$count" -lt 2 ]]; then
  echo "Expected at least 2 versions, got $count"
  exit 1
fi
