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
"$LFS_BIN" --repo "$repo" add "$tmp/b.txt" --tag project:other

"$LFS_BIN" --repo "$repo" view create Demo --query "tag:project:demo"
views="$("$LFS_BIN" --repo "$repo" view list)"
echo "$views" | grep -q "Demo (id:"

"$LFS_BIN" --repo "$repo" view explain "$object_a" --view Demo >/dev/null

# Nested view: child query is composed with parent query using logical AND
"$LFS_BIN" --repo "$repo" view create AllProjects --query "tag:project"
"$LFS_BIN" --repo "$repo" view create DemoOnly --parent AllProjects --query "tag:project:demo"
"$LFS_BIN" --repo "$repo" view explain "$object_a" --view AllProjects/DemoOnly >/dev/null

# Ambiguous bare-name references should fail when sibling-unique names are reused under different parents
"$LFS_BIN" --repo "$repo" view create OtherProjects --query "tag:project"
"$LFS_BIN" --repo "$repo" view create DemoOnly --parent OtherProjects --query "tag:project:other"
if "$LFS_BIN" --repo "$repo" view explain "$object_a" --view DemoOnly >/dev/null 2>&1; then
  echo "expected ambiguous view reference to fail"
  exit 1
fi

# Delete policies for views with children
"$LFS_BIN" --repo "$repo" view create ParentToDelete --query "tag:project"
"$LFS_BIN" --repo "$repo" view create ChildToDetach --parent ParentToDelete --query "tag:project:demo"
if "$LFS_BIN" --repo "$repo" view delete ParentToDelete >/dev/null 2>&1; then
  echo "expected delete without child policy to fail"
  exit 1
fi
"$LFS_BIN" --repo "$repo" view delete ParentToDelete --detach-children

"$LFS_BIN" --repo "$repo" view create ParentCascade --query "tag:project"
"$LFS_BIN" --repo "$repo" view create ChildCascade --parent ParentCascade --query "tag:project:demo"
"$LFS_BIN" --repo "$repo" view delete ParentCascade --cascade
if "$LFS_BIN" --repo "$repo" view explain "$object_a" --view ChildCascade >/dev/null 2>&1; then
  echo "expected cascaded child view to be deleted"
  exit 1
fi
