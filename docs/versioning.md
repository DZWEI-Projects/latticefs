# Versioning in LatticeFS

This document explains how versioning works and walks through two hands-on examples.

## How versioning works

- Every object has a stable object ID and a list of versions.
- Each version points to immutable content (a content-addressed chunk manifest).
- Objects have a **current version** pointer that the CLI uses by default.
- Versions are **immutable**. A new version is created instead of mutating an old one.

CLI commands involved:

- `lfs revise <object-id> <file> [-m <message>]` — create a new version from file content
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

1. List versions:

```bash
lfs versions <object-id>
```

1. Create a new version from an older one:

```bash
lfs restore <object-id> v1
```

This creates a **new version** that points to the same content as `v1` and sets it as the latest.

1. Compare versions:

```bash
lfs diff <object-id>@v1 <object-id>@v2
```

If the restored version is identical, the diff will be empty.

1. Move the current pointer:

```bash
lfs checkout <object-id>@v1
```

Now `v1` is the current version for operations like `lfs get` or `lfs export`.

## Tutorial 2: Updating object content with new versions

This tutorial shows a way to update content while keeping the same object ID: create a new **version** with `lfs revise`.

1. Import the original file and tag it:

```bash
lfs add ./report.md --tag doc:report
```

Record the object ID printed.

1. Edit the file locally, then create a new version:

```bash
lfs revise <object-id> ./report.md -m "add summary section"
```

Or pipe content from stdin:
```bash
cat ./report.md | lfs revise <object-id> --stdin -m "add summary section"
```

1. List versions to confirm:

```bash
lfs versions <object-id>
```

1. Diff versions:

```bash
lfs diff <object-id>@v1 <object-id>@v2
```

Notes:

- `lfs revise` creates a **new version** under the same object ID (no duplicate objects).
- Linking objects with `lfs link ... replaces ...` is still useful to represent **lineage across distinct objects**, but it does **not** create a new version.
