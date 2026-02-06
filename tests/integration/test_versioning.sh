#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require_lfs_bin

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
message_output="$("$LFS_BIN" --repo "$repo" message set "$object_id" --message "late note")"
if ! echo "$message_output" | grep -q "Set message for"; then
  echo "Expected message update confirmation, got: $message_output"
  exit 1
fi
versions="$("$LFS_BIN" --repo "$repo" versions "$object_id")"
count="$(echo "$versions" | grep -c '^v')"

if [[ "$count" -lt 2 ]]; then
  echo "Expected at least 2 versions, got $count"
  exit 1
fi
