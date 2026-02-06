# CLI Reference

This document lists every CLI command, subcommand, and its arguments, with examples and example output.

## Localization

LatticeFS automatically detects your operating system locale and displays built-in view names and descriptions in your language. Currently supported:

- **English** (en_*)
- **German** (all other locales, default fallback)

Built-in view names can be referenced in either language:

| English | German |
|---------|--------|
| recent | neueste |
| projects | projekte |
| drafts | entwürfe |
| review / pending review | zur prüfung |
| approved | genehmigt |
| all / all objects | alle objekte |

Examples throughout this document show English output, but German systems will see localized names automatically.

## Global flags
- `-v`, `-vv` — increase verbosity
- `--repo <path>` — override repository root
- `--fuse` — enable FUSE operations (required for `mount`)
- If `--repo` is omitted and the current directory contains `.latticefs.toml` with `repo.auto_load = true`, the CLI uses `.latticefs/` under the current directory as the repo root.

Example:
```bash
lfs --repo /tmp/latticefs -v status
```

Auto-load example:
```bash
cat > .latticefs.toml <<'EOF'
[repo]
auto_load = true
EOF

# Uses .latticefs under current directory as repo root (same as: lfs --repo "$PWD/.latticefs" status)
lfs status
```

Example output:
```text
Objects: 3
Versions: 3
Chunks: 12
Chunk bytes: 98312
```

## CLI help text

The CLI includes built-in usage guidance for every command. Use:

```bash
lfs --help
lfs <command> --help
```

Example:
```bash
lfs view --help
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

## Info commands

### `lfs info checksum <ref>`
Show the content checksum for an object (BLAKE3 merkle root).

Arguments:
- `<ref>` — object ID or alias (optionally `@version`)

Example:
```bash
lfs info checksum <object-id>
```

Example output:
```text
Object: <object-id>
Version: <version-id>
Algorithm: BLAKE3 (chunk merkle root)
Chunk root: <hash>
Manifest: <hash>
Size bytes: 12345
Chunks: 12
```

### `lfs info object <ref> [--all-versions]`
Show summary statistics for a single object.

Arguments:
- `<ref>` — object ID or alias
- `--all-versions` — include per-version details

Example:
```bash
lfs info object <object-id> --all-versions
```

### `lfs info view <name|id>`
Show statistics for a built-in or dynamic view. Built-in view names can be provided in English or German.

Examples:
```bash
lfs info view recent
lfs info view neueste  # German equivalent
```

### `lfs info view-objects <name|id> [--all-tags] [--raw-tags]`
List objects for a view with minimal tag output.

Notes:
- By default, auto/system tags are hidden. Use `--all-tags` to include them.
- Tags ending in `_b64` are base64url-decoded for display. Use `--raw-tags` to show both encoded and decoded values.
- Built-in view names are accepted in both English and German.

Example:
```bash
lfs info view-objects recent --all-tags
# or in German:
lfs info view-objects neueste --all-tags
```

### `lfs info views`
Summarize all built-in and dynamic views. Built-in views are displayed in the system locale (English or German).

Example:
```bash
lfs info views
```

Example output (German locale):
```text
Built-in views:
- Neueste: 5 objects
- Projekte: 3 objects
- Entwürfe: 2 objects
- Zur Prüfung: 1 objects
- Genehmigt: 10 objects
- Alle Objekte: 15 objects
```

### `lfs info policy <name>`
Show policy details and how many objects reference it.

Example:
```bash
lfs info policy compliance
```

### `lfs info policies`
Summarize policy counts and list names.

Example:
```bash
lfs info policies
```

### `lfs info shares`
Summarize shared capabilities (total, active/expired, permissions).

Example:
```bash
lfs info shares
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

### `lfs tags <ref>`
List **all tags** for an object (user tags, system tags, and auto-extracted tags).

Example:
```bash
lfs tags <object-id>
```

Example output:
```text
project:phoenix
owner:benn
```

Example output (none):
```text
No tags
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

### `lfs meta <ref> [--tags] [--text] [--all-tags]`
Show extracted metadata for an object.

By default it prints **auto‑extracted tags** (`auto:*`) and **extracted text** if present.
Use `--all-tags` to include non‑auto tags too.
EXIF and ID3 are surfaced as `auto:exif:*` and `auto:id3:*` tags.
Source names captured during import are stored as `auto:filename_b64` and `auto:relpath_b64` (base64url-encoded).

Example:
```bash
lfs meta <object-id>
```

Example output:
```text
Tags:
- auto:mimetype:text/plain
- auto:text:true

Text:
hello latticefs
```

Example (tags only, include user tags too):
```bash
lfs meta <object-id> --tags --all-tags
```

Example output:
```text
Tags:
- auto:mimetype:text/plain
- auto:text:true
- project:phoenix
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

### `lfs state set <ref>[@version] <state>`
Set the workflow state for a specific version (defaults to current version when no `@version` is provided).

Valid states: `draft`, `review`, `approved`, `discarded`, `sealed`, `archived`.
Setting a version to `sealed` prevents new versions from being created while it is current.

Example (explicit version):
```bash
lfs state set <object-id>@v2 review
```

Example output:
```text
Set state draft -> review for <version-id>
```

Example (current version):
```bash
lfs state set <object-id> sealed
```

Example output:
```text
Set state draft -> sealed for <version-id>
```

### `lfs message set <ref>[@version] (--clear | -m <message>)`
Update (or clear) the commit message for a version. You must provide either `--clear` or `-m/--message`. By default the current version is used when no `@version` is provided.

Example (explicit version):
```bash
lfs message set <object-id>@v2 -m "add summary"
```

Example output:
```text
Set message for <version-id>
```

Example (clear message):
```bash
lfs message set <object-id> --clear
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

**Localization Note**: Built-in view names and descriptions are automatically localized based on your operating system locale. The system supports English and German, with German as the default fallback for non-English locales. View names can be referenced in either language (e.g., "recent" or "neueste").

### `lfs view create <name> --query '<lql>'`
Create a dynamic view.

Example:
```bash
lfs view create "Projects" --query "tag:project"
```

Example output:
```text
Created view Projects (<view-id>)
```

### `lfs view list`
List built‑in and dynamic views.

Example:
```bash
lfs view list
```

Example output (English locale):
```text
Built-in views:
- Recent: Objects updated within the last 7 days
- Projects: Objects tagged as projects
- Drafts: Objects in draft state
- Pending Review: Objects pending review
- Approved: Approved objects
- All Objects: All objects in the repository

Dynamic views:
- Projects (id: <view-id>): tag:project
```

Example output (German locale):
```text
Built-in views:
- Neueste: Objekte, die in den letzten 7 Tagen aktualisiert wurden
- Projekte: Objekte, die als Projekte gekennzeichnet sind
- Entwürfe: Objekte im Entwurfsstadium
- Zur Prüfung: Objekte, die auf Prüfung warten
- Genehmigt: Genehmigte Objekte
- Alle Objekte: Alle Objekte im Repository

Dynamic views:
- Projects (id: <view-id>): tag:project
```

Notes:
- Dynamic view IDs are UUIDs and can be used anywhere a view name is accepted.
- Built-in view names can be referenced in either English or German (e.g., `lfs info view recent` or `lfs info view neueste`).

### `lfs view delete <name|id>`
Delete a dynamic view.

Example:
```bash
lfs view delete "Projects"
```

Example output:
```text
Deleted view Projects (<view-id>)
```

### `lfs view explain <ref> [--query '<lql>'] [--view <name|id>]`
Explain why an object matches a query or view.

Note: Built-in view names are accepted in both English and German.

Example (explicit query):
```bash
lfs view explain <object-id> --query "tag:project:phoenix"
```

Example output:
```text
Object <object-id>: MATCHED
✓ tag:project:phoenix (actual: tag:project:phoenix)
```

Example (named view, English):
```bash
lfs view explain <object-id> --view "Projects"
lfs view explain <object-id> --view <view-id>
```

Example (named view, German):
```bash
lfs view explain <object-id> --view "Projekte"
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

### `lfs share snapshot <view-name|view-id> --to <pubkey> [--cap <perm>] [--expires <dur>]`
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

### `lfs export <ref|view-name|view-id> --output <path> [--mode <tree|archive>]`
Export a single object or a view.

Arguments:
- `--mode tree` — export to directory tree (default)
- `--mode archive` — export to tar archive

Notes:
- If you pass a view ID (UUID), the CLI will export that view and print its name and ID.
- Built-in view names are accepted in both English and German.
- Object references can include version specifiers (e.g., `@v1` or `@<version-id>`) to export a specific version.

Example (single object, current version):
```bash
lfs export <object-id> --output ~/Exports/out.bin
```

Example output:
```text
Exported <object-id>
```

Example (single object, specific version):
```bash
lfs export <object-id>@v1 --output ~/Exports/out_v1.bin
lfs export <object-id>@<version-id> --output ~/Exports/out_version.bin
```

Example output:
```text
Exported <object-id>@v1
```

Example (view to directory, English):
```bash
lfs export "Projects" --output ~/Exports/projects --mode tree
lfs export <view-id> --output ~/Exports/projects --mode tree
```

Example (view to directory, German):
```bash
lfs export "Projekte" --output ~/Exports/projekte --mode tree
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

## File Watcher Commands

### `lfs edit <reference> [--no-watch] [-m <message>]`
Export an object to the watch directory and open it in the default editor. When the watcher daemon is running, saves are automatically versioned.

Example:
```bash
lfs edit a1b2c3d4-5678-90ab-cdef-1234567890ab
lfs edit my-alias
lfs edit my-alias --no-watch
```

Example output:
```text
Exported to /tmp/latticefs-open/a1b2c3d4-..._report.pdf
Registered with watcher daemon (auto-versioning enabled)
```

### `lfs watchd start [--foreground]`
Start the watcher daemon. By default runs as a background process.

Example:
```bash
lfs watchd start
lfs watchd start --foreground
lfs watchd --repo ./myrepo start --foreground  # Use a specific repo
```

Example output:
```text
Watcher daemon started (pid 12345)
```

### `lfs watchd stop`
Stop the running watcher daemon.

Example:
```bash
lfs watchd stop
```

Example output:
```text
Shutdown request sent to watcher daemon
```

### `lfs watchd status`
Show watcher daemon status and list of watched files.

Example:
```bash
lfs watchd status
```

Example output:
```text
Watcher daemon: running (pid 12345)
Watch directory: /tmp/latticefs-open
Watched files: 1

OBJECT ID                              NAME                           PATH
a1b2c3d4-...                           report.pdf                     /tmp/latticefs-open/a1b2c3d4-..._report.pdf
```
