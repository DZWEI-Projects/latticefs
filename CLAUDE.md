# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LatticeFS is a post-file filesystem with immutable, versioned objects, content-addressed storage, semantic graph links, and query-backed views. It's a Rust-based system with:
- **base**: Core Rust library (storage, model, query, views, FUSE, import/export)
- **cli**: Rust CLI binary (`lfs` command)
- **gui**: React + TypeScript frontend + Tauri backend (under active development)
- **services**: Go services for share/sync (future)
- **specs**: Protocol specifications and PRD

## Build Commands

### CLI builds

```bash
# CLI without FUSE support
cargo build -p cli

# CLI with FUSE support
cargo build -p cli --features latticefs-base/fuse

# Release build
cargo build -p cli --release
```

### GUI builds

```bash
# Development server (Vite)
cd gui && npm run dev

# Tauri development (hot-reload)
cd gui && npm run dev:tauri

# Build Tauri app
cd gui && npm run build:tauri

# Build frontend only
cd gui && npm run build
```

### All workspace

```bash
# Check entire workspace
cargo check --workspace

# Build all (Rust only)
cargo build --workspace
```

## Test Commands

### CLI end-to-end smoke test

```bash
cargo test -p cli cli_flow_basic -- --nocapture
```

### Run all tests

```bash
cargo test --workspace
```

### Frontend tests

```bash
cd gui && npm run test
cd gui && npm run test:watch
```

### Property-based tests (base lib)

```bash
cargo test -p base --test property_tests
```

## Development Notes

### Architecture Layers

The codebase has a clear separation:

1. **base** (`base/src/`): Core library with modules for:
   - `model`: Object/Version/Link types, versioning DAG
   - `storage`: ChunkStore, MetadataStore, hash computation
   - `query`: Query parser, evaluator, and semantics (find by meaning)
   - `views`: BuiltinView, DynamicView, ViewStore (query-backed views)
   - `crypto`: Ed25519 signing, AES-GCM encryption, capability system
   - `policy`: PolicyEngine, QuotaEnforcer, RateLimiter
   - `fuse`: Read-only FUSE mount (optional via feature flag)
   - `import`: File import with metadata extraction
   - `ipc`: Inter-process communication protocol
   - `config`: Configuration schema and defaults
   - `events`: EventBus and audit logging

2. **cli** (`cli/src/`): Command-line interface:
   - `commands/`: Individual command implementations (init, add, import, view, export, etc.)
   - `commands/common.rs`: Shared utilities like `open_repo()`
   - Uses Clap for CLI parsing and Indicatif for progress bars

3. **gui** (`gui/`): Tauri + React desktop application:
   - React frontend in `gui/src/` with TypeScript and shadcn/ui components
   - Tauri backend in `gui/src-tauri/` (Rust) that links to base library
   - Component structure: `pages/`, `components/`, `hooks/`, `data/`
   - Built with Vite, TailwindCSS, React Hook Form, shadcn/ui

### Key Design Patterns

- **Versioning**: Every change creates a new version; objects are immutable
- **Content-addressed**: Data identified by hash (Blake3)
- **Query-backed views**: Views defined by semantic queries, not folder hierarchies
- **Semantic linking**: Graph-based relationships between objects
- **FUSE mounting**: Read-only filesystem view (optional, requires `--fuse` flag)

### Repository Structure

The repository uses Sled (embedded database) for local storage. Configuration lives in `$LATTICE_HOME` (defaults to `~/.latticefs`) and can be overridden with `LFS_KEY_PASSWORD` environment variable.

### Environment Variables

- `LATTICE_HOME`: Override default repository location
- `LFS_KEY_PASSWORD`: Password for key encryption (required for init)
- `XDG_CONFIG_HOME`: Config directory override (used in tests)

### Testing Setup

- CLI tests in `cli/tests/cli_flow.rs` use `tempfile` for isolated test environments
- Tests set up temporary `LATTICE_HOME` and `XDG_CONFIG_HOME` directories
- Use `assert_cmd` and `predicates` crates for CLI assertions
- The `cli_flow_basic` test exercises init → add → query → export workflow

### Frontend Stack

- **React 18** with TypeScript
- **Tauri 2** for cross-platform desktop (Rust bridge)
- **shadcn/ui** (Radix UI primitives + TailwindCSS)
- **React Hook Form** + **Zod** for forms
- **React Router** for navigation
- **TanStack React Query** for state management
- **Vite** for bundling
- **Vitest** for testing

### Common Development Tasks

**Starting fresh CLI development**: Open repo with `commands::common::open_repo()`, access LatticeRepo methods, return Result<()>

**Adding CLI command**: Create `commands/new_cmd.rs`, add handler in `commands/mod.rs`, add variant to Command enum, wire in `main.rs`

**Frontend component**: Create in `gui/src/components/`, import from `gui/src/lib/` utility modules, use shadcn button/form/dialog as base

**Running with debug output**: Use `-v` or `-vv` flags (global), or set `RUST_LOG=debug` for CLI

## Documentation

Key docs in `docs/`:
- **cli.md**: Full command reference
- **cli-recipes.md**: "Do This → Run That" workflows
- **architecture.md**: System design details
- **views.md**: Query-backed view semantics
- **storage-layout.md**: On-disk format
- **fuse.md**: FUSE mounting and troubleshooting
- **config.md**: Configuration schema
- **identity.md**: Ed25519 key management

## Linting & Formatting

```bash
cd gui && npm run lint
```

For Rust, use standard `rustfmt` (cargo handles this).

## Important Constraints

- **FUSE is opt-in**: Mount requires `--fuse` flag and build with `fuse` feature
- **Immutable objects**: Once stored, objects cannot be modified
- **Content-addressed**: Hash is primary identifier, not file paths
- **Single repo per directory**: `LATTICE_HOME` points to one repository location
