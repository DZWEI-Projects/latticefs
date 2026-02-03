# Versioning in LatticeFS

This document explains how versioning works and walks through two hands-on examples.

## How versioning works
- Every object has a stable object ID and a list of versions.
- Each version points to immutable content (a content-addressed chunk manifest).
- Objects have a **current version** pointer that the CLI uses by default.
- Versions are **immutable**. A new version is created instead of mutating an old one.

CLI commands involved:
- `lfs versions <object-id>` — list versions
- `lfs diff <object-id>@v1 <object-id>@v2` — compare two versions of the same object
- `lfs checkout <object-id>@v2` — set the current version pointer
- `lfs restore <object-id> v1` — create a new version from an older one

## Tutorial 1: Basic version lifecycle (restore + checkout)
This tutorial shows the mechanics of listing, restoring, and switching versions.

1. Initialize a repo and add a file:
```bash
lfs init
lfs add ./notes.txt
```

2. List versions:
```bash
lfs versions <object-id>
```

3. Create a new version from an older one:
```bash
lfs restore <object-id> v1
```
This creates a **new version** that points to the same content as `v1` and sets it as the latest.

4. Compare versions:
```bash
lfs diff <object-id>@v1 <object-id>@v2
```
If the restored version is identical, the diff will be empty.

5. Move the current pointer:
```bash
lfs checkout <object-id>@v1
```
Now `v1` is the current version for operations like `lfs get` or `lfs export`.

## Tutorial 2: Representing edits using lineage links
The CLI does not have a direct “update object content” command. When you edit a file and import it again, you get a **new object** with its own version history. You can connect these objects with a `replaces` link to represent lineage.

1. Import the original file and tag it:
```bash
lfs add ./report.md --tag doc:report
```
Record the object ID printed.

2. Edit the file locally, then import it again (new object):
```bash
lfs add ./report.md --tag doc:report
```
Record the new object ID printed.

3. Link the new object to the old one:
```bash
lfs link <new-object-id> replaces <old-object-id>
```

4. Use a view to gather all versions of this document:
```bash
lfs view create "Report" --query "tag:doc:report"
```

5. Export the view to see the set of objects (filenames are object IDs):
```bash
lfs export "Report" --output /tmp/report-objects --mode tree
ls /tmp/report-objects
```

Notes:
- `lfs diff` only compares versions within the **same object**. For cross-object diffs, export both and use an external diff tool.
- This linkage pattern gives you an explicit “this replaces that” chain in the graph while keeping objects immutable.
