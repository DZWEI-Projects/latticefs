# CLI Reference

This document lists every CLI command, subcommand, and its arguments, with examples.

## Global flags
- `-v`, `-vv` — increase verbosity
- `--repo <path>` — override repository root
- `--fuse` — enable FUSE operations (required for `mount`)

Example:
```bash
lfs --repo /tmp/latticefs -v status
```

## System commands

### `lfs init`
Initialize a repository and write default config.

Example:
```bash
lfs init
```

### `lfs status`
Show repository statistics.

Example:
```bash
lfs status
```

### `lfs gc`
Garbage collect unreferenced chunks.

Example:
```bash
lfs gc
```

### `lfs verify [<ref>] [--deep]`
Verify data integrity for all objects or a specific object.

Arguments:
- `<ref>` — object ID or alias (optional)
- `--deep` — verify all versions instead of current only

Example:
```bash
lfs verify --deep
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

### `lfs import <path> [--tag <key:value>...]`
Import a directory or file tree.

Arguments:
- `<path>` — directory or file
- `--tag <key:value>` — attach tags (repeatable)

Example:
```bash
lfs import ~/Documents --tag project:demo
```

### `lfs tag <ref> <key:value>...`
Add tags to an object.

Example:
```bash
lfs tag <object-id> priority:high
```

### `lfs untag <ref> <key>`
Remove a tag by key.

Example:
```bash
lfs untag <object-id> priority
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

### `lfs cat <ref>`
Print object content to stdout.

Example:
```bash
lfs cat <object-id>
```

## Versioning

### `lfs versions <ref> [--graph]`
List versions for an object.

Arguments:
- `--graph` — include parent references

Example:
```bash
lfs versions <object-id> --graph
```

### `lfs diff <ref@v1> <ref@v2>` or `lfs diff <ref> <v1> <v2>`
Diff two versions (text or binary). The versions can be from the **same object** or **different objects**.

- Use `lfs diff <ref@v1> <ref@v2>` to compare any two versions (even across objects).
- Use `lfs diff <ref> <v1> <v2>` as shorthand when both versions belong to the same object.

Example:
```bash
lfs diff <object-id>@v1 <object-id>@v2
```

### `lfs restore <ref> <version>`
Create a new version from a prior version (restores content).

Example:
```bash
lfs restore <object-id> v1
```

### `lfs checkout <ref@version>`
Set the object’s current version pointer.

Example:
```bash
lfs checkout <object-id>@v2
```

## Views

### `lfs view create <name> --query '<lql>'`
Create a dynamic view.

Example:
```bash
lfs view create "Projects" --query "tag:project"
```

### `lfs view list`
List built‑in and dynamic views.

Example:
```bash
lfs view list
```

### `lfs view delete <name>`
Delete a dynamic view.

Example:
```bash
lfs view delete "Projects"
```

### `lfs view explain <ref> [--query '<lql>'] [--view <name>]`
Explain why an object matches a query or view.

Example:
```bash
lfs view explain <object-id> --query "tag:project:phoenix"
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

### `lfs share snapshot <view-name> --to <pubkey> [--cap <perm>] [--expires <dur>]`
Create and share a view snapshot.

Example:
```bash
lfs share snapshot "Projects" --to <did:key:...>
```

### `lfs shares list`
List stored share capabilities.

Example:
```bash
lfs shares list
```

### `lfs revoke <capability-id|token> [--reason <text>]`
Revoke a UCAN capability.

Example:
```bash
lfs revoke <capability-cid>
```

## Policies

### `lfs policy create <name> --template <project-collab|personal|compliance>`
Create a policy from a template.

Example:
```bash
lfs policy create project-collab --template project-collab
```

### `lfs policy apply <ref> <policy-name>`
Attach a policy to an object.

Example:
```bash
lfs policy apply <object-id> project-collab
```

### `lfs policy remove <ref> <policy-name>`
Remove a policy from an object.

Example:
```bash
lfs policy remove <object-id> project-collab
```

## Trust & quarantine

### `lfs trust get <ref>`
Get trust level for an object.

Example:
```bash
lfs trust get <object-id>
```

### `lfs trust set <ref> <trusted|untrusted|quarantined|approved|score>`
Set trust level for an object.

Example:
```bash
lfs trust set <object-id> quarantined
```

### `lfs quarantine list`
List quarantined objects.

Example:
```bash
lfs quarantine list
```

## Export

### `lfs export <ref|view> --output <path> [--mode <tree|archive>]`
Export a single object or a view.

Arguments:
- `--mode tree` — export to directory tree (default)
- `--mode archive` — export to tar archive

Example:
```bash
lfs export <object-id> --output ~/Exports/out.bin
```

## FUSE mount

### `lfs --fuse mount [<mount-point>]`
Mount the read‑only filesystem projection.

Example:
```bash
lfs --fuse mount ~/Lattice
```

### `lfs unmount [<mount-point>]`
Unmount the filesystem.

Example:
```bash
lfs unmount ~/Lattice
```
