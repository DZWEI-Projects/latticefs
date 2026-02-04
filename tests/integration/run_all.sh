#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "Building CLI..."
cargo build -p cli

if [[ -z "${LFS_BIN:-}" ]]; then
  export LFS_BIN="$ROOT/target/debug/lfs"
fi
export LFS_KEY_PASSWORD="${LFS_KEY_PASSWORD:-test-pass}"

tests=(
  test_add_and_retrieve.sh
  test_versioning.sh
  test_view_queries.sh
  test_sharing_workflow.sh
  test_import_export.sh
  test_policy_enforcement.sh
  test_fuse_mount.sh
)

for test in "${tests[@]}"; do
  echo "==> $test"
  "$ROOT/tests/integration/$test"
done

echo "All integration tests completed."
