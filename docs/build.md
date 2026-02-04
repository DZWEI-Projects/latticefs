# Build Guide

## Repository layout
```
/ (repo root)
├── base/       # Rust library: storage, model, query, views, FUSE, import/export
├── cli/        # Rust CLI binary (lfs)
├── services/   # Go services (share/sync)
├── specs/      # PRD and protocol specifications
├── docs/       # Documentation
```

## Build process (Rust)

### Default development build (fast, unoptimized)
```bash
cargo build -p cli
```
- Uses the **dev** profile (unoptimized, debuginfo).
- Output binaries go to `target/debug/`.

### Build with FUSE support
```bash
cargo build -p cli --features latticefs-base/fuse
```
- Enables the FUSE mount code path.
- Runtime still requires `--fuse` when mounting (see `docs/fuse.md`).

### Release build (optimized)
```bash
cargo build -p cli --release
```
- Uses the **release** profile (optimized, minimal debuginfo).
- Output binaries go to `target/release/`.

## Cargo build profiles

Cargo profiles control optimization, debug info, and other compiler settings.

### Common built-in profiles
- **dev** (default)
  - Unoptimized, fast builds, full debuginfo.
  - Used by `cargo build`.
- **release**
  - Optimized, slower builds, minimal debuginfo.
  - Used by `cargo build --release`.
- **test**
  - Similar to dev, used for `cargo test`.
- **bench**
  - Similar to release, used for `cargo bench`.

### Enable a profile
- Dev (default):
  ```bash
  cargo build
  ```
- Release:
  ```bash
  cargo build --release
  ```
- Tests:
  ```bash
  cargo test
  ```

### Custom profiles
You can add custom profiles in the repo root `Cargo.toml`:
```toml
[profile.profile-name]
opt-level = 2
lto = false
```
Then build with:
```bash
cargo build --profile profile-name
```

## Outputs and where to find files
- Rust binary outputs: `target/debug/` or `target/release/`
- CLI binary: `target/debug/lfs` or `target/release/lfs`
- Config and storage (default): `~/.latticefs/`
  - `config.toml`, `chunks/`, `meta/`, `logs/`

## Useful commands
```bash
# init repo
lfs init

# import files
lfs import ~/Documents --tag project:demo

# create a view
lfs view create "Projects" --query "tag:project"
```

## Integration tests

Integration tests live under `tests/integration/` and expect the `LFS_BIN` environment
variable to point at the CLI binary.

Run everything:
```bash
cargo build -p cli
export LFS_BIN="$(pwd)/target/debug/lfs"
./tests/integration/run_all.sh
```

Run a single script:
```bash
cargo build -p cli
export LFS_BIN="$(pwd)/target/debug/lfs"
./tests/integration/test_add_and_retrieve.sh
```
