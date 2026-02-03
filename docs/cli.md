# CLI Reference

This document lists every CLI command, subcommand, and its arguments, with examples and example output.

## Global flags
- `-v`, `-vv` — increase verbosity
- `--repo <path>` — override repository root
- `--fuse` — enable FUSE operations (required for `mount`)

Example:
```bash
lfs --repo /tmp/latticefs -v status
```

Example output:
```text
Objects: 3
Versions: 3
Chunks: 12
Chunk bytes: 98312
```

## System commands

### `lfs init`
Initialize a repository and write default config.

Example:
```bash
lfs init
```

Example output:
```text
Initialized repository at /path/to/repo
```

### `lfs status`
Show repository statistics.

Example:
```bash
lfs status
```

Example output:
```text
Objects: 12
Versions: 15
Chunks: 84
Chunk bytes: 2459012
```

### `lfs gc`
Garbage collect unreferenced chunks.

Example:
```bash
lfs gc
```

Example output:
```text
GC removed 0 chunks
```

### `lfs verify [<ref>] [--deep]`
Verify data integrity for all objects or a specific object.

Arguments:
- `<ref>` — object ID or alias (optional)
- `--deep` — verify all versions instead of current only

Example (all objects, current versions):
```bash
lfs verify
```

Example output:
```text
Verified 12 objects
```

Example (single object, deep):
```bash
lfs verify <object-id> --deep
```

Example output:
```text
Verified <object-id>
```

## Object management

### `lfs add <file> [--tag <key:value>...]`
Import a single file as a new object.

Arguments:
- `<file>` — file path
- `--tag <key:value>` — attach tags (repeatable)

Example:
```bash
lfs add ./report.pdf --tag project:phoenix
```

Example output:
```text
Added object <object-id>
```

### `lfs import <path> [--tag <key:value>...]`
Import a directory or file tree.

Arguments:
- `<path>` — directory or file
- `--tag <key:value>` — attach tags (repeatable)

Example:
```bash
lfs import ~/Documents --tag project:demo
```

Example output:
```text
Import completed successfully
```

### `lfs tag <ref> <key:value>...`
Add tags to an object.

Example:
```bash
lfs tag <object-id> priority:high owner:benn
```

Example output:
```text
Tagged <object-id>
```

### `lfs untag <ref> <key>`
Remove a tag by key.

Example:
```bash
lfs untag <object-id> priority
```

Example output:
```text
Untagged <object-id>
```

### `lfs link <object-a> <link-type> <object-b>`
Create a typed link between objects.

Link types:
- `derived-from`
- `references`
- `belongs-to`
- `replaces`
- `related`

Example:
```bash
lfs link <a> derived-from <b>
```

Example output:
```text
Linked <a> -> <b> (derived-from)
```

### `lfs get <ref> --output <path> [--ucan <token>]`
Write object content to a file.

Arguments:
- `<ref>` — object ID or alias
- `--output <path>` — output file path
- `--ucan <token>` — UCAN token for read access (optional)

Example:
```bash
lfs get <object-id> --output ~/Downloads/out.bin
```

Example output:
```text
Wrote /Users/you/Downloads/out.bin
```

### `lfs cat <ref>`
Print object content to stdout.

Example:
```bash
lfs cat <object-id>
```

Example output:
```text
hello latticefs
```

## Versioning

### `lfs revise <ref> <file> [-m <message>]` or `lfs revise <ref> --stdin [-m <message>]`
Create a **new version** for an existing object using the content of a file or stdin.

Example:
```bash
lfs revise <object-id> ./report.md -m "fix typos"
```

Example output:
```text
Revised <object-id> to new version <version-id>
```

Example (stdin):
```bash
cat ./report.md | lfs revise <object-id> --stdin -m "fix typos"
```

Example output:
```text
Revised <object-id> to new version <version-id>
```

### `lfs versions <ref> [--graph]`
List versions for an object.

Arguments:
- `--graph` — include parent references

Example:
```bash
lfs versions <object-id> --graph
```

Example output:
```text
v1 <version-id> parent=none size=1234 state=approved
v2 <version-id> parent=<version-id> size=1240 state=approved
```

### `lfs diff <ref@v1> <ref@v2>` or `lfs diff <ref> <v1> <v2>`
Diff two versions (text or binary). The versions can be from the **same object** or **different objects**.

- Use `lfs diff <ref@v1> <ref@v2>` to compare any two versions (even across objects).
- Use `lfs diff <ref> <v1> <v2>` as shorthand when both versions belong to the same object.

Example (explicit refs):
```bash
lfs diff <object-id>@v1 <object-id>@v2
```

Example output:
```text
--- left
+++ right
@@
-hello latticefs
+hello latticefs v2
```

Example (shorthand for same object):
```bash
lfs diff <object-id> v1 v2
```

Example output:
```text
No differences
```

Example (different objects):
```bash
lfs diff <object-a>@v1 <object-b>@v1
```

Example output:
```text
Binary diff: left=1024 bytes right=980 bytes
First differing byte at offset 12
```

### `lfs restore <ref> <version>`
Create a new version from a prior version (restores content).

Example:
```bash
lfs restore <object-id> v1
```

Example output:
```text
Restored <object-id> to new version <version-id>
```

### `lfs checkout <ref@version>`
Set the object’s current version pointer.

Example:
```bash
lfs checkout <object-id>@v2
```

Example output:
```text
Checked out <object-id> to <version-id>
```

## Views

### `lfs view create <name> --query '<lql>'`
Create a dynamic view.

Example:
```bash
lfs view create "Projects" --query "tag:project"
```

Example output:
```text
Created view Projects
```

### `lfs view list`
List built‑in and dynamic views.

Example:
```bash
lfs view list
```

Example output:
```text
Built-in views:
- Recent: Objects updated within the last 7 days
- Projects: Objects tagged as projects
- Drafts: Objects in draft state
- Pending Review: Objects pending review
- Approved: Approved objects
- All Objects: All objects in the repository

Dynamic views:
- Projects: tag:project
```

### `lfs view delete <name>`
Delete a dynamic view.

Example:
```bash
lfs view delete "Projects"
```

Example output:
```text
Deleted view Projects
```

### `lfs view explain <ref> [--query '<lql>'] [--view <name>]`
Explain why an object matches a query or view.

Example (explicit query):
```bash
lfs view explain <object-id> --query "tag:project:phoenix"
```

Example output:
```text
Object <object-id>: MATCHED
✓ tag:project:phoenix (actual: tag:project:phoenix)
```

Example (named view):
```bash
lfs view explain <object-id> --view "Projects"
```

Example output:
```text
Object <object-id>: MATCHED
✓ tag:project (actual: tag:project:phoenix)
```

## Sharing

### `lfs share <ref> --to <pubkey> [--cap <perm>] [--expires <dur>]`
Share an object using a UCAN token.

Arguments:
- `--to <pubkey>` — recipient DID:key or hex public key
- `--cap <perm>` — permission (read|write|comment|share|admin), default `read`
- `--expires <dur>` — duration like `7d`, default `7d`

Example:
```bash
lfs share <object-id> --to <did:key:...> --cap read --expires 7d
```

Example output:
```text
Shared object <object-id>
CID: <capability-cid>
UCAN: <ucan-token>
```

### `lfs share snapshot <view-name> --to <pubkey> [--cap <perm>] [--expires <dur>]`
Create and share a view snapshot.

Example:
```bash
lfs share snapshot "Projects" --to <did:key:...>
```

Example output:
```text
Snapshot shared. CID: <capability-cid>
UCAN: <ucan-token>
```

### `lfs shares list`
List stored share capabilities.

Example:
```bash
lfs shares list
```

Example output:
```text
<capability-cid>
  subject: did:key:z6M...
  perms: latticefs:read
```

Example output (none):
```text
No shares
```

### `lfs revoke <capability-id|token> [--reason <text>]`
Revoke a UCAN capability.

Example:
```bash
lfs revoke <capability-cid>
```

Example output:
```text
Revoked capability <capability-cid>
```

## Policies

### `lfs policy create <name> --template <project-collab|personal|compliance>`
Create a policy from a template.

Example:
```bash
lfs policy create project-collab --template project-collab
```

Example output:
```text
Created policy project-collab
```

### `lfs policy apply <ref> <policy-name>`
Attach a policy to an object.

Example:
```bash
lfs policy apply <object-id> project-collab
```

Example output:
```text
Applied policy project-collab to <object-id>
```

### `lfs policy remove <ref> <policy-name>`
Remove a policy from an object.

Example:
```bash
lfs policy remove <object-id> project-collab
```

Example output:
```text
Removed policy project-collab from <object-id>
```

## Trust & quarantine

### `lfs trust get <ref>`
Get trust level for an object.

Example:
```bash
lfs trust get <object-id>
```

Example output:
```text
<object-id>: 25 (quarantined)
```

### `lfs trust set <ref> <trusted|untrusted|quarantined|approved|score>`
Set trust level for an object.

Example:
```bash
lfs trust set <object-id> quarantined
```

Example output:
```text
Set trust <object-id> -> 0
```

### `lfs quarantine list`
List quarantined objects.

Example:
```bash
lfs quarantine list
```

Example output:
```text
<object-id>
<object-id>
```

Example output (none):
```text
No quarantined objects
```

## Export

### `lfs export <ref|view> --output <path> [--mode <tree|archive>]`
Export a single object or a view.

Arguments:
- `--mode tree` — export to directory tree (default)
- `--mode archive` — export to tar archive

Example (single object):
```bash
lfs export <object-id> --output ~/Exports/out.bin
```

Example output:
```text
Exported <object-id>
```

Example (view to directory):
```bash
lfs export "Projects" --output ~/Exports/projects --mode tree
```

Example output:
```text
Exported Projects
```

Example (view to archive):
```bash
lfs export "Projects" --output ~/Exports/projects.tar --mode archive
```

Example output:
```text
Exported Projects
```

## FUSE mount

### `lfs --fuse mount [<mount-point>]`
Mount the read‑only filesystem projection.

Example:
```bash
lfs --fuse mount ~/Lattice
```

Example output:
```text
(no output on success)
```

### `lfs unmount [<mount-point>]`
Unmount the filesystem.

Example:
```bash
lfs unmount ~/Lattice
```

Example output:
```text
(no output on success)
```
