# Listing Objects

This document explains how to list objects in NeuralFS, including “all objects” and filtered subsets (per‑tag, per‑state, etc.).

## Is there a direct "list objects" command?

There is **no dedicated `lfs objects list` command** in the CLI. However, you can list objects in a view directly using the **`lfs stats view-objects`** command (see [CLI Reference](cli.md#lfs-stats-view-objects-name---all-tags---raw-tags) for details).

For more extensive operations, listing is done through **views** and **exports**, which are backed by Lattice Query Language (LQL).

In practice, you can list objects by:

- using `lfs stats view-objects <view-name>` to print object IDs and tags directly to stdout,
- exporting a view to a directory (filenames are object IDs), or
- browsing a view via the read‑only FUSE mount.

This keeps listing consistent with the query system, and avoids adding a separate listing path that would bypass query semantics.

## Direct listing with stats view-objects

To quickly list objects in a view without exporting, use `lfs stats view-objects`:

```bash
lfs stats view-objects "All"
```

This prints each object ID to stdout, one per line.

To include all tags (including system/auto tags):

```bash
lfs stats view-objects "All" --all-tags
```

For full details and options, see the [CLI Reference](cli.md#lfs-stats-view-objects-name---all-tags---raw-tags).

## List all objects

Use the built‑in `All` view and export it.

```bash
lfs export "All" --output /tmp/lfs-all --mode tree
ls /tmp/lfs-all
```

The filenames in `/tmp/lfs-all` are object IDs.

## List objects by tag

Create a view for a tag query and export it.

```bash
lfs view create "Projects" --query "tag:project"
lfs export "Projects" --output /tmp/lfs-projects --mode tree
ls /tmp/lfs-projects
```

For a specific tag value:

```bash
lfs view create "Project:demo" --query "tag:project:demo"
lfs export "Project:demo" --output /tmp/lfs-demo --mode tree
```

## List objects by trust/state

You can query trust or state the same way.

```bash
lfs view create "Drafts" --query "state:draft"
lfs export "Drafts" --output /tmp/lfs-drafts --mode tree
```

```bash
lfs view create "Trusted" --query "trust >= 80"
lfs export "Trusted" --output /tmp/lfs-trusted --mode tree
```

## List objects by time window

Use the `updated` field in LQL:

```bash
lfs view create "Recent" --query "updated within 7d SORT updated DESC"
lfs export "Recent" --output /tmp/lfs-recent --mode tree
```

## List objects via FUSE (read‑only)

If you build with FUSE support and use `--fuse`, you can browse views as directories:

```bash
lfs --fuse mount ~/Lattice
ls ~/Lattice/views/all\ objects
```

## Summary

- Direct listing is done through **views**.
- The **built‑in `All` view** gives you everything.
- Exporting a view gives you a filesystem‑friendly listing (object IDs as filenames).
- The FUSE mount is a read‑only way to browse views if you want a familiar filesystem surface.
