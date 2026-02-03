# LatticeFS

LatticeFS is a post-file filesystem with immutable, versioned objects, content-addressed storage, semantic graph links, and query-backed views. It ships as a Rust CLI and (optionally) exposes a read-only FUSE mount.

## Build

### CLI only (no FUSE)
```bash
cargo build -p cli
```

### CLI with FUSE support
```bash
cargo build -p cli --features latticefs-base/fuse
```

## Run

Initialize and import:
```bash
lfs init
lfs import ~/Documents --tag project:demo
```

Create and list views:
```bash
lfs view create "Projects" --query "tag:project"
lfs view list
```

Export data:
```bash
lfs export <object-id> --output ~/Exports/out.bin
```

## FUSE usage (read-only)

FUSE is **optional** and must be explicitly enabled at runtime with `--fuse`. This is a safety guard so the CLI won’t attempt mounting unless you ask for it.

```bash
# Requires build with fuse feature
lfs --fuse mount ~/Lattice
```

If you run `lfs mount` without `--fuse`, it will fail with a clear error message. If you build without the fuse feature, mounting will also fail and instruct you to rebuild with the feature.

For details and OS-specific requirements, see `docs/fuse.md`.

## Tests

Run the CLI end-to-end smoke test:
```bash
cargo test -p cli cli_flow_basic -- --nocapture
```
