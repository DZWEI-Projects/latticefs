#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require_lfs_bin

tmp="$(mktemp -d)"
export XDG_CONFIG_HOME="$tmp/config"
repo="$tmp/repo"
mkdir -p "$repo"

printf "shared\n" > "$tmp/shared.txt"

"$LFS_BIN" --repo "$repo" init
output="$("$LFS_BIN" --repo "$repo" add "$tmp/shared.txt")"
object_id="$(echo "$output" | awk '{print $3}')"

if ! command -v openssl >/dev/null 2>&1; then
  echo "openssl not available; skipping share test"
  exit 0
fi

key_dir="$tmp/keys"
mkdir -p "$key_dir"
openssl genpkey -algorithm ed25519 -out "$key_dir/priv.pem" >/dev/null 2>&1
recipient_hex="$(
  openssl pkey -in "$key_dir/priv.pem" -pubout -outform DER 2>/dev/null \
    | tail -c 32 \
    | xxd -p -c 32
)"

share_output="$("$LFS_BIN" --repo "$repo" share "$object_id" --cap read --to "$recipient_hex" --expires 1h --password "$LFS_KEY_PASSWORD")"
echo "$share_output" | grep -q "UCAN:"
echo "$share_output" | grep -q "CID:"
