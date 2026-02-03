# CLI Identities (Ed25519)

## Why the CLI uses identities
LatticeFS uses **capability-based security** (UCAN tokens). To issue and validate capabilities, the CLI needs a cryptographic identity. The CLI uses an Ed25519 keypair as its local identity.

This identity is used to:
- **Sign UCAN tokens** when sharing objects or snapshots.
- **Sign revocations** when access is revoked.
- **Represent the actor** in object metadata (created_by, tags, links, etc.).
- **Prove authorship** of actions recorded in audit logs (future phase).

## What Ed25519 is
Ed25519 is a modern elliptic-curve signature scheme that provides:
- Fast signing and verification
- Strong security (widely used and well‑vetted)
- Compact public keys and signatures

## Where identities are stored
The CLI stores the private key using the OS keyring when available:
- **macOS**: Keychain
- **Linux**: Secret Service / GNOME Keyring
- **Windows**: Credential Manager

If the keyring isn’t available, LatticeFS can fall back to an encrypted file store (Argon2id + AES‑GCM).

## When identities are created/loaded
The CLI loads or creates an identity when a command needs to sign data, including:
- `lfs add`
- `lfs import`
- `lfs link`
- `lfs restore`
- `lfs trust set`
- `lfs share`
- `lfs revoke`

Commands that only read data (e.g., `lfs get`, `lfs export`, `lfs view list`) do not access the keyring.

## Keychain prompts
On macOS, the first time the CLI accesses the keychain, the system may prompt for your login password. This is expected and ensures the private key remains protected by the OS.
