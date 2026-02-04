#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require_lfs_bin

tmp="$(mktemp -d)"
export XDG_CONFIG_HOME="$tmp/config"
repo="$tmp/repo"
mkdir -p "$repo"

"$LFS_BIN" --repo "$repo" init

mount_point="$tmp/mount"
mkdir -p "$mount_point"

set +e
err="$("$LFS_BIN" --repo "$repo" mount "$mount_point" 2>&1 >/dev/null)"
status=$?
set -e

if [[ $status -eq 0 ]]; then
  echo "FUSE mount succeeded; unmount manually if needed."
  exit 0
fi

if echo "$err" | grep -q "FUSE support not enabled"; then
  echo "FUSE not enabled; skipping mount test."
  exit 0
fi

if echo "$err" | grep -qi "fusermount\|fuse"; then
  echo "FUSE not available in environment; skipping mount test."
  exit 0
fi

echo "$err"
exit 1
