# CLI Recipes (Do This → Run That)

This guide maps common goals to the exact CLI commands, flags, and arguments you should use. It also compares each task to the “old way” you’d do it in a traditional filesystem so the shift in mental model is clear.

## Start a new repo
Old way: you might create a new folder and start dropping files into it.

LatticeFS way: initialize a repo to track objects, metadata, and views.

Goal: initialize LatticeFS in the current directory.

```bash
lfs init
```

## Point the CLI at a specific repo
Old way: `cd` into a folder and work there.

LatticeFS way: keep your shell where you are, but point the CLI at a repo path.

Goal: operate on a repo somewhere else.

```bash
lfs --repo /path/to/repo status
```

## Import a single file
Old way: copy a file into a folder and hope you remember where it went.

LatticeFS way: import it as an immutable, versioned object.

Goal: add one file as an object.

```bash
lfs add ./report.pdf
```

Add tags at import time:

```bash
lfs add ./report.pdf --tag project:phoenix --tag owner:benn
```

## Import a folder tree
Old way: copy a whole directory to a new location.

LatticeFS way: import the tree so every file becomes an object with metadata.

Goal: ingest a directory recursively.

```bash
lfs import ~/Documents
```

Add tags to all imported objects:

```bash
lfs import ~/Documents --tag project:demo
```

## Attach or remove tags
Old way: encode metadata in folder names or filenames.

LatticeFS way: add structured tags you can query later.

Goal: add tags later.

```bash
lfs tag <object-id> priority:high owner:benn
```

Goal: list tags on an object.

```bash
lfs tags <object-id>
```

Goal: remove a tag key.

```bash
lfs untag <object-id> priority
```

## Retrieve object content
Old way: open the file from its path.

LatticeFS way: retrieve content by object ID (or via a view).

Goal: export an object to a file.

```bash
lfs get <object-id> --output ~/Downloads/out.bin
```

Goal: print object content to stdout.

```bash
lfs cat <object-id>
```

## Read extracted metadata
Old way: inspect EXIF/ID3 with separate tools or scripts.

LatticeFS way: read extracted metadata and text directly from the repo.

Goal: view extracted metadata (auto tags + text).

```bash
lfs meta <object-id>
```

Notes:
`lfs tags` shows all tags (including user tags). `lfs meta` focuses on auto‑extracted metadata and text.
EXIF and ID3 appear as `auto:exif:*` and `auto:id3:*` tags.

Goal: show all tags, not just auto tags.

```bash
lfs meta <object-id> --tags --all-tags
```

## See original import names
Goal: list objects in a view and see decoded `auto:filename_b64` / `auto:relpath_b64` tags.

```bash
lfs stats view-objects "All" --all-tags
```

Goal: show encoded + decoded values (debugging).

```bash
lfs stats view-objects "All" --all-tags --raw-tags
```

## Get an object checksum
Old way: export an object to a file, then hash it with a separate tool.

LatticeFS way: ask the CLI for the object checksum directly.

Goal: get the content hash for the current version.

```bash
lfs stats checksum <object-id>
```

Goal: get the checksum for a specific version.

```bash
lfs stats checksum <object-id>@v2
```

## See object versions
Old way: keep “report-final-final.pdf” copies.

LatticeFS way: a single object tracks all versions.

Goal: list versions.

```bash
lfs versions <object-id>
```

Goal: list versions including parent graph.

```bash
lfs versions <object-id> --graph
```

## Compare two versions
Old way: manually diff two files.

LatticeFS way: diff two versions of the same object.

Goal: diff two versions of the same object.

```bash
lfs diff <object-id>@v1 <object-id>@v2
```

## Restore or pin a version
Old way: copy an old file back over a newer one.

LatticeFS way: restore or pin versions explicitly.

Goal: restore content from an older version (new version is created).

```bash
lfs restore <object-id> v1
```

Goal: move “current” pointer to a specific version.

```bash
lfs checkout <object-id>@v2
```

## Update object content (new version)
Old way: edit a file in place, with no history unless you make a copy.

LatticeFS way: create a new **version** under the same object ID.

Goal: write new content for an existing object.

```bash
lfs revise <object-id> ./report.md -m "fix typos"
```

Goal: write new content from stdin (useful in pipelines).

```bash
cat ./report.md | lfs revise <object-id> --stdin -m "fix typos"
```

## Set version state / lock updates
Old way: use manual conventions like “FINAL” in filenames.

LatticeFS way: set explicit version states.

Goal: mark a version as review.

```bash
lfs state set <object-id>@v2 review
```

Goal: seal the current version (prevents new versions).

```bash
lfs state set <object-id> sealed
```

Note: when a version is `sealed`, attempts to create a new version will fail.

## Create and use views
Old way: build directory trees as “views” (often duplicating files).

LatticeFS way: define query-backed views without copying data.

Goal: create a view from a query.

```bash
lfs view create "Projects" --query "tag:project"
```

Goal: list views.

```bash
lfs view list
```

Goal: delete a view.

```bash
lfs view delete "Projects"
```

## Explain why something matches
Old way: guess why a file ended up in a folder.

LatticeFS way: ask the system for the exact match explanation.

Goal: explain why an object matches a query.

```bash
lfs view explain <object-id> --query "tag:project:phoenix"
```

Goal: explain why an object matches a named view.

```bash
lfs view explain <object-id> --view "Projects"
```

## Link objects (semantic graph)
Old way: put related items in the same folder or in a README.

LatticeFS way: create explicit, typed links between objects.

Goal: link two objects with a typed relationship.

```bash
lfs link <object-a> derived-from <object-b>
```

Available link types: `derived-from`, `references`, `belongs-to`, `replaces`, `related`.

## Export data
Old way: copy files out of a folder tree.

LatticeFS way: export objects or entire views.

Goal: export a single object to a file.

```bash
lfs export <object-id> --output ~/Exports/out.bin
```

Goal: export a view to a directory tree.

```bash
lfs export "Projects" --output ~/Exports/projects --mode tree
```

Goal: export a view to a tar archive.

```bash
lfs export "Projects" --output ~/Exports/projects.tar --mode archive
```

## Share and revoke access
Old way: email a file or share a folder.

LatticeFS way: issue a capability token scoped by permissions and time.

Goal: share a single object (UCAN token).

```bash
lfs share <object-id> --to <did:key:...> --cap read --expires 7d
```

Goal: share a view snapshot.

```bash
lfs share snapshot "Projects" --to <did:key:...>
```

Goal: list stored shares.

```bash
lfs shares list
```

Goal: revoke a capability.

```bash
lfs revoke <capability-id>
```

## Policies
Old way: set rules informally (“don’t share this”).

LatticeFS way: attach explicit policies to objects.

Goal: create a policy from a template.

```bash
lfs policy create project-collab --template project-collab
```

Goal: attach a policy to an object.

```bash
lfs policy apply <object-id> project-collab
```

Goal: remove a policy.

```bash
lfs policy remove <object-id> project-collab
```

## Trust & quarantine
Old way: move suspicious files to a quarantine folder.

LatticeFS way: mark trust states and query them.

Goal: check trust status.

```bash
lfs trust get <object-id>
```

Goal: set trust status.

```bash
lfs trust set <object-id> quarantined
```

Goal: list quarantined objects.

```bash
lfs quarantine list
```

## Verify integrity
Old way: rely on backups and hope files are intact.

LatticeFS way: verify content hashes and version integrity.

Goal: verify all current objects.

```bash
lfs verify
```

Goal: deep verification (all versions).

```bash
lfs verify --deep
```

Goal: verify a single object.

```bash
lfs verify <object-id>
```

## Garbage collection
Old way: manually delete old copies and hope nothing breaks.

LatticeFS way: GC removes unreferenced chunks safely.

Goal: remove unreferenced chunks.

```bash
lfs gc
```

## FUSE mount (read‑only)
Old way: access files directly via the filesystem.

LatticeFS way: project a read‑only filesystem view for compatibility tools.

Goal: mount the read‑only view. Requires build with fuse feature **and** `--fuse`.

```bash
lfs --fuse mount ~/Lattice
```

Goal: unmount.

```bash
lfs unmount ~/Lattice
```

## Common troubleshooting
If you see “mount requires --fuse”, rerun with `--fuse`.
If you see “built without FUSE support”, rebuild:

```bash
cargo build -p cli --features latticefs-base/fuse
```
