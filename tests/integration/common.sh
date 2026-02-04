#!/usr/bin/env bash
set -euo pipefail

require_lfs_bin() {
  if [[ -z "${LFS_BIN:-}" ]]; then
    cat <<'EOF'
LFS_BIN is not set.
Fix:
  1) Build the CLI: cargo build -p cli
  2) Export the path to the binary:
     export LFS_BIN="$(pwd)/target/debug/lfs"
  3) Re-run this test script.
EOF
    exit 2
  fi

  if [[ ! -x "$LFS_BIN" ]]; then
    cat <<EOF
LFS_BIN is set to '$LFS_BIN', but the binary is missing or not executable.
Fix:
  1) Build the CLI: cargo build -p cli
  2) Ensure LFS_BIN points to the lfs binary:
     export LFS_BIN="$(pwd)/target/debug/lfs"
EOF
    exit 2
  fi
}
