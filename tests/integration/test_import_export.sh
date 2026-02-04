#!/usr/bin/env bash
set -euo pipefail

: "${LFS_BIN:?LFS_BIN not set}"

tmp="$(mktemp -d)"
export XDG_CONFIG_HOME="$tmp/config"
repo="$tmp/repo"
mkdir -p "$repo"

mkdir -p "$tmp/input"
printf "first\n" > "$tmp/input/one.txt"
printf "second\n" > "$tmp/input/two.txt"

"$LFS_BIN" --repo "$repo" init
"$LFS_BIN" --repo "$repo" import "$tmp/input" --tag project:demo
"$LFS_BIN" --repo "$repo" view create Demo --query "tag:project:demo"

output_dir="$tmp/exported"
"$LFS_BIN" --repo "$repo" export Demo --output "$output_dir" --mode tree

count="$(find "$output_dir" -type f | wc -l | tr -d ' ')"
if [[ "$count" -lt 2 ]]; then
  echo "Expected at least 2 exported files, got $count"
  exit 1
fi
