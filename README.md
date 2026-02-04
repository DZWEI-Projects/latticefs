# LatticeFS

LatticeFS is a post-file filesystem with immutable, versioned objects, content-addressed storage, semantic graph links, and query-backed views. It ships as a Rust CLI and (optionally) exposes a read-only FUSE mount.

## Repo layout

```
/ (repo root)
├── base/       # Rust library: storage, model, query, views, FUSE, import/export
├── cli/        # Rust CLI binary (lfs)
├── services/   # Go services (share/sync)
├── specs/      # PRD and protocol specifications
├── docs/       # Documentation
```

## Build

### CLI only (no FUSE)

```bash
cargo build -p cli
```

### CLI with FUSE support

```bash
cargo build -p cli --features latticefs-base/fuse
```

### Release build

```bash
cargo build -p cli --release
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

## Docs

- [Documentation index](docs/index.md)
- [Build process and Cargo profiles](docs/build.md)
- [CLI command reference](docs/cli.md)
- [CLI recipes (Do This → Run That)](docs/cli-recipes.md)
- [Versioning guide](docs/versioning.md)
- [Listing objects](docs/object-listing.md)
- [Views](docs/views.md)
- [Guided workflows](docs/workflows.md)
- [FUSE setup and troubleshooting](docs/fuse.md)
- [Storage layout](docs/storage-layout.md)
- [Storage encoding map](docs/storage-encoding.md)
- [Configuration](docs/config.md)
- [CLI identities](docs/identity.md)

## Tests

Run the CLI end-to-end smoke test:

```bash
cargo test -p cli cli_flow_basic -- --nocapture
```

**shrinnnggg**
