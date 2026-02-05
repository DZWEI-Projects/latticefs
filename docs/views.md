# Views

Views are query-backed projections over objects. They let you group, filter, and export data without copying it.

## Built-in Views

LatticeFS includes several built-in views that are always available:

| View | Query | Description |
|------|-------|-------------|
| Recent | `updated within 7d SORT updated DESC LIMIT 100` | Objects updated in the last 7 days |
| Projects | `tag:project SORT updated DESC` | Objects tagged as projects |
| Drafts | `state:draft SORT updated DESC` | Objects in draft state |
| Review | `state:review SORT updated DESC` | Objects pending review |
| Approved | `state:approved SORT updated DESC` | Approved objects |
| All | `trust >= 0` | All objects in the repository |

**Localization**: Built-in view names and descriptions are automatically displayed in your system's locale. English and German are currently supported, with German as the default for non-English locales. Both English and German names work in CLI commands (e.g., `lfs stats view recent` or `lfs stats view neueste`). See [Localization](i18n-localization.md) for details.

## What you can do with views
- Create a view from an LQL query
- List built‑in and dynamic views
- Delete a dynamic view
- Explain why an object matches a view
- Export a view to a directory or archive
- Share a view snapshot

## Create a view
```bash
lfs view create "Projects" --query "tag:project"
```

## List views
```bash
lfs view list
```

This shows both built‑in and dynamic views. Dynamic views include their UUID IDs, which you can use anywhere a view name is accepted.

## Delete a view
```bash
lfs view delete "Projects"
lfs view delete <view-id>
```

Only dynamic views can be deleted.

## Explain matches for a view
```bash
lfs view explain <object-id> --view "Projects"
lfs view explain <object-id> --view <view-id>
```

## Export a view (to “see” its contents)
```bash
lfs export "Projects" --output /tmp/projects --mode tree
lfs export <view-id> --output /tmp/projects --mode tree
ls /tmp/projects
```

The filenames are object IDs. Use `lfs get` to retrieve content for any ID.

## Share a view snapshot
```bash
lfs share snapshot "Projects" --to <did:key:...>
lfs share snapshot <view-id> --to <did:key:...>
```

## Is there a view “update” command?
There is **no explicit update command**. Views are defined by a name and a query; to change it, delete and recreate:

```bash
lfs view delete "Projects"
lfs view create "Projects" --query "tag:project AND state:approved"
```

## How to “view a view” (see what belongs to it)
You have two supported ways:

1. Export it to a directory (filenames are object IDs):
```bash
lfs export "Projects" --output /tmp/projects --mode tree
ls /tmp/projects
```

2. Browse it via FUSE (read‑only):
```bash
lfs --fuse mount ~/Lattice
ls ~/Lattice/Projects
```

There is **no CLI command that prints a list of object IDs** directly. The canonical way to enumerate a view is to export it or browse via FUSE.
