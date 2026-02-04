#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require_lfs_bin

tmp="$(mktemp -d)"
export XDG_CONFIG_HOME="$tmp/config"
repo="$tmp/repo"
mkdir -p "$repo"

printf "policy\n" > "$tmp/policy.txt"
printf "policy update\n" > "$tmp/policy_v2.txt"

"$LFS_BIN" --repo "$repo" init
output="$("$LFS_BIN" --repo "$repo" add "$tmp/policy.txt")"
object_id="$(echo "$output" | awk '{print $3}')"

"$LFS_BIN" --repo "$repo" policy create compliance-test --template compliance
"$LFS_BIN" --repo "$repo" policy apply "$object_id" compliance-test

if "$LFS_BIN" --repo "$repo" revise "$object_id" "$tmp/policy_v2.txt"; then
  echo "Expected policy to block revision, but revise succeeded"
  exit 1
fi
