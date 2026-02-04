# Versioning in LatticeFS

This document explains how versioning works and walks through two hands-on examples.

## How versioning works

- Every object has a stable object ID and a list of versions.
- Each version points to immutable content (a content-addressed chunk manifest).
- Objects have a **current version** pointer that the CLI uses by default.
- Versions are **immutable**. A new version is created instead of mutating an old one.

CLI commands involved:

- `lfs revise <object-id> <file> [-m <message>]` — create a new version from file content
- `lfs state set <object-id>[@version] <state>` — set version workflow state
- `lfs versions <object-id>` — list versions
- `lfs diff <object-id>@v1 <object-id>@v2` — compare two versions of the same object
- `lfs checkout <object-id>@v2` — set the current version pointer
- `lfs restore <object-id> v1` — create a new version from an older one

Version states:

- `draft` — default for new versions
- `review` — pending review
- `approved` — accepted/vetted
- `discarded` — auto-set when a draft is superseded by a new version
- `sealed` — locks the object against further updates
- `archived` — deprecated

Auto-advance rules when a new version is created:

- If the previous version is `review`, it becomes `approved`.
- If the previous version is `draft`, it becomes `discarded`.

Locking rule:

- If the current version is `sealed`, creating a new version fails with a clear error.

## Tutorial 1: Basic version lifecycle (restore + checkout)

This tutorial shows the mechanics of listing, restoring, and switching versions.

- Initialize a repo and add a file:

```bash
lfs init
lfs add ./notes.txt
```

- List versions:

```bash
lfs versions <object-id>
```

- Create a new version from an older one:

```bash
lfs restore <object-id> v1
```

This creates a **new version** that points to the same content as `v1` and sets it as the latest.

- Compare versions:

```bash
lfs diff <object-id>@v1 <object-id>@v2
```

If the restored version is identical, the diff will be empty.

- Move the current pointer:

```bash
lfs checkout <object-id>@v1
```

Now `v1` is the current version for operations like `lfs get` or `lfs export`.

## Tutorial 2: Updating object content with new versions

This tutorial shows a way to update content while keeping the same object ID: create a new **version** with `lfs revise`.

- Import the original file and tag it:

```bash
lfs add ./report.md --tag doc:report
```

Record the object ID printed.

- Edit the file locally, then create a new version:

```bash
lfs revise <object-id> ./report.md -m "add summary section"
```

Or pipe content from stdin:

```bash
cat ./report.md | lfs revise <object-id> --stdin -m "add summary section"
```

- List versions to confirm:

```bash
lfs versions <object-id>
```

- Diff versions:

```bash
lfs diff <object-id>@v1 <object-id>@v2
```

Notes:

- `lfs revise` creates a **new version** under the same object ID (no duplicate objects).
- Linking objects with `lfs link ... replaces ...` is still useful to represent **lineage across distinct objects**, but it does **not** create a new version.
- You can set states explicitly with `lfs state set`, but auto-advance still applies when creating a new version.
