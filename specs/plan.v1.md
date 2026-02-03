# LatticeFS PRD Enrichment Plan

## Overview
This plan enriches the existing PRD at [specs/PRD.md](../../../specs/PRD.md) with implementation-specific details gathered from architecture discussions. The goal is to create a complete, unambiguous specification ready for autonomous implementation.

## Current State
- ✅ Comprehensive architectural PRD exists
- ✅ Project structure initialized (Rust workspace + Go module)
- ✅ Core dependencies declared in Cargo.toml
- ❌ No functional code implemented yet
- ❌ Implementation details missing from PRD

## Implementation Decisions Made

### Core Architecture
| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Chunking Algorithm** | FastCDC | Best balance of dedup ratio and performance |
| **FUSE Inode Strategy** | Deterministic hash-based | Stable across remounts, more robust |
| **LQL Parser** | Hand-written recursive descent | Simple, debuggable, no external parser deps |
| **Capability Format** | UCAN tokens | JWT-like, delegatable, IPFS ecosystem standard |
| **Error Handling** | Domain-specific typed errors (thiserror) | Type safety across module boundaries |
| **Metadata Privacy** | Three-tier (private/shared/public) | Per PRD security model |
| **Policy Evaluation** | Most restrictive wins | Safe default, never accidentally grants |
| **Event Streaming** | In-process tokio channels | Simple, fast, good for MVP |

### CLI & UX
| Decision | Choice | Rationale |
|----------|--------|-----------|
| **CLI Framework** | clap with derive macros | Industry standard, automatic help/completions |
| **CLI Verbosity** | Terse with --verbose flag | Good for both humans and scripts |
| **Storage Location** | $HOME/.latticefs | Standard pattern, override via env var |
| **Testing Strategy** | Unit tests + property-based (proptest) | Core logic coverage + invariant checking |

### Security & Storage
| Decision | Choice | Rationale |
|----------|--------|-----------|
| **DoS Protection** | Simple per-user quotas + rate limits | Config-driven, good for MVP |
| **Key Management** | Ed25519 keypair + OS keyring | Secure, platform-native storage |
| **Share Model** | Object + view snapshot sharing | More than basic, less than full dynamic |
| **Migration Strategy** | CLI import with metadata extraction | Essential for onboarding real users |
| **Metadata Extraction** | Basic + EXIF/ID3 + text content | Media metadata + text prep for ML (v2) |

### Services & Sync
| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Go Services API** | gRPC with Protocol Buffers | Type-safe, streaming, multi-language support |
| **Rust ↔ Go IPC** | Unix domain sockets | Fast local IPC, works cross-platform |
| **Sync Protocol** | Custom (CRDT-based for DAG) | Conflict-free graph merging |
| **MVP Sync Scope** | Basic HTTP share server | Share via capabilities, no bi-directional sync yet |

### MVP Scope Refinements
| Feature | MVP Status | Notes |
|---------|------------|-------|
| **FUSE Mount** | ✅ Read-only | Browse views, write via CLI |
| **Encryption** | ✅ Full (Ed25519 + AES-GCM) | Keys in OS keyring |
| **Import** | ✅ `lfs import` command | Metadata extraction, chunking, tagging |
| **Sync Service** | ✅ HTTP share server | Capability-based object sharing |
| **Full CRDT Sync** | ❌ Deferred to v1.1+ | MVP is local + HTTP share only |
| **Embeddings/ML** | ❌ Deferred to v2 | Text extraction in MVP, no vectors yet |

## PRD Additions Needed

### 1. New Section: Implementation Specifications

#### 1.1 FastCDC Chunking Details
```
Algorithm: FastCDC (Fast Content-Defined Chunking)
Average chunk size: 16KB
Min chunk size: 8KB
Max chunk size: 64KB
Hash function: BLAKE3
Window size: 64 bytes
Mask bits: 13 bits (for avg 16KB)
```

#### 1.2 FUSE Inode Strategy
```
Inode generation: BLAKE3(object_id)[0..8] as u64
Collision handling: Linear probing in sled index
Inode cache: LRU cache (10k entries)
Special inodes:
  - 1: root (/)
  - 2: /views
  - 3: /projects
  - 4: /recent
  - 5+: dynamic query results
```

#### 1.3 LQL Grammar (EBNF)
```ebnf
query      = expr (SORT sort_expr)? (LIMIT number)?
expr       = term ((AND | OR) term)*
term       = predicate | NOT term | "(" expr ")"
predicate  = tag_pred | state_pred | type_pred | trust_pred | time_pred | traverse
tag_pred   = "tag:" identifier (":" identifier)*
state_pred = "state:" ("draft"|"review"|"approved"|"archived")
type_pred  = "type:" mimetype
trust_pred = "trust" ("=" | ">=" | "<=") trust_level
time_pred  = "updated" ("within"|"before"|"after") duration
traverse   = "references(" ref ")" | "closure(" ref ")"
sort_expr  = field ("ASC"|"DESC")
```

#### 1.4 UCAN Token Format
```rust
// UCAN = User Controlled Authorization Network
struct UCANToken {
    issuer: PublicKey,        // Ed25519 public key
    audience: PublicKey,       // Recipient's public key
    subject: ObjectID,         // Object being shared
    capabilities: Vec<Cap>,    // [read, write, comment]
    not_before: Timestamp,
    expires_at: Timestamp,
    facts: HashMap<String, Value>,  // Optional context
    proof: Vec<UCANToken>,     // Delegation chain
    signature: Signature,      // Ed25519 signature
}
```

#### 1.5 Storage Layout
```
$HOME/.latticefs/
├── config.toml                # User config
├── chunks/                    # Content-addressed chunks
│   ├── aa/                   # First 2 hex digits
│   │   ├── bb/              # Next 2 hex digits
│   │   │   └── <blake3>    # Actual chunk data
├── meta/                      # sled database
│   ├── objects/              # Object metadata
│   ├── versions/             # Version DAG
│   ├── policies/             # Policy definitions
│   ├── capabilities/         # Issued tokens
│   └── index/                # Search index
├── keys/                      # Encrypted key material
│   └── identity.key          # Ed25519 keypair (OS keyring reference)
└── logs/                      # Audit log
    └── events.jsonl          # Append-only event log
```

### 2. New Section: Module Architecture

```
base/
├── src/
│   ├── lib.rs               # Public API
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── chunks.rs        # FastCDC chunking + storage
│   │   ├── content.rs       # Content addressing (BLAKE3)
│   │   └── metadata.rs      # sled-backed metadata store
│   ├── model/
│   │   ├── mod.rs
│   │   ├── object.rs        # Object, Version structs
│   │   ├── link.rs          # Graph links
│   │   ├── tag.rs           # Tag system
│   │   └── policy.rs        # Policy data structures
│   ├── policy/
│   │   ├── mod.rs
│   │   ├── engine.rs        # Policy evaluation (most restrictive wins)
│   │   └── quota.rs         # Rate limiting + disk quotas
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── identity.rs      # Ed25519 keypair management
│   │   ├── encryption.rs    # AES-GCM per-object encryption
│   │   ├── capability.rs    # UCAN token implementation
│   │   └── keyring.rs       # OS keyring integration
│   ├── query/
│   │   ├── mod.rs
│   │   ├── parser.rs        # Hand-written LQL parser
│   │   ├── evaluator.rs     # Query execution over graph
│   │   └── explain.rs       # Explainability ("why this result?")
│   ├── views/
│   │   ├── mod.rs
│   │   ├── dynamic.rs       # Dynamic view implementation
│   │   ├── snapshot.rs      # View snapshots for sharing
│   │   └── builtin.rs       # Recent, Projects, ByType, etc.
│   ├── fuse/
│   │   ├── mod.rs
│   │   ├── mount.rs         # FUSE filesystem implementation
│   │   ├── inode.rs         # Hash-based inode mapping
│   │   └── readonly.rs      # Read-only FUSE ops (MVP)
│   ├── import/
│   │   ├── mod.rs
│   │   ├── scanner.rs       # Filesystem walker
│   │   ├── metadata.rs      # EXIF, ID3, text extraction
│   │   └── chunker.rs       # FastCDC chunking for import
│   ├── events/
│   │   ├── mod.rs
│   │   ├── bus.rs           # Tokio mpsc event bus
│   │   └── types.rs         # Event type definitions
│   └── error.rs             # Domain-specific error types

cli/
├── src/
│   ├── main.rs              # Entry point
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── add.rs           # lfs add
│   │   ├── tag.rs           # lfs tag
│   │   ├── link.rs          # lfs link
│   │   ├── versions.rs      # lfs versions
│   │   ├── diff.rs          # lfs diff
│   │   ├── view.rs          # lfs view create/list/delete
│   │   ├── share.rs         # lfs share
│   │   ├── revoke.rs        # lfs revoke
│   │   ├── policy.rs        # lfs policy
│   │   ├── trust.rs         # lfs trust
│   │   ├── export.rs        # lfs export
│   │   ├── import.rs        # lfs import
│   │   └── mount.rs         # lfs mount / unmount
│   └── output.rs            # Formatted output, verbosity control

services/
├── cmd/
│   └── lfs-share/
│       └── main.go          # HTTP share server entry point
├── internal/
│   ├── api/
│   │   ├── server.go        # gRPC server
│   │   └── handlers.go      # Share request handlers
│   ├── sync/
│   │   ├── protocol.go      # CRDT sync protocol (future)
│   │   └── merge.go         # Graph merge logic (future)
│   └── share/
│       ├── http.go          # HTTP share endpoint
│       └── capability.go    # Capability verification
└── proto/
    └── share.proto          # gRPC API definitions
```

### 3. New Section: Dependencies to Add

#### Rust (add to base/Cargo.toml)
```toml
clap = { version = "4.5", features = ["derive", "env"] }
fastcdc = "3.1"              # FastCDC chunking
keyring = "2.3"              # OS keyring access
mime_guess = "2.0"           # MIME type detection
kamadak-exif = "0.5"         # EXIF parsing
id3 = "1.13"                 # ID3 tag parsing
pdf-extract = "0.7"          # PDF text extraction
lopdf = "0.33"               # PDF parsing
proptest = "1.4"             # Property-based testing (dev)
tempfile = "3.10"            # Test fixtures (dev)
```

#### Rust (add to cli/Cargo.toml)
```toml
latticefs-base = { path = "../base" }
clap = { version = "4.5", features = ["derive", "env", "color"] }
anyhow = "1.0"
tokio = { version = "1.36", features = ["full"] }
tracing-subscriber = "0.3"
```

#### Go (add to services/go.mod)
```go
require (
    google.golang.org/grpc v1.61.0
    google.golang.org/protobuf v1.32.0
    github.com/golang/protobuf v1.5.3
)
```

### 4. New Section: Configuration Schema

```toml
# $HOME/.latticefs/config.toml

[storage]
path = "~/.latticefs"
cache_size_mb = 512
max_chunk_size_kb = 64

[quota]
max_storage_gb = 100
max_operations_per_minute = 1000
burst_allowance = 100

[fuse]
mount_point = "~/Lattice"
readonly = true              # MVP: read-only FUSE
allow_other = false

[crypto]
algorithm = "aes-256-gcm"
key_derivation = "argon2id"
keyring_service = "latticefs"

[share]
http_port = 8771
max_concurrent_shares = 10
default_ttl_days = 7

[import]
extract_exif = true
extract_id3 = true
extract_text = true
create_embeddings = false    # v2 feature

[logging]
level = "info"
format = "json"
audit_log = "~/.latticefs/logs/events.jsonl"
```

### 5. New Section: CLI Command Reference

```bash
# Object management
lfs add <file> --tag <key:value>...
lfs tag <ref> <key:value>...
lfs untag <ref> <key>
lfs link <object-a> <link-type> <object-b>
lfs get <ref> --output <path>
lfs cat <ref>[@version]

# Versioning
lfs versions <ref> [--graph]
lfs diff <ref>@v1 <ref>@v2
lfs restore <ref> <version>
lfs checkout <ref>@version

# Views
lfs view create <name> --query '<lql>'
lfs view list
lfs view delete <name>
lfs view explain <ref>        # Why is this in the view?

# Sharing (HTTP share server)
lfs share <ref> --cap <read|write> --to <pubkey> --expires <duration>
lfs share snapshot <view-name> --to <pubkey>
lfs revoke <capability-id>
lfs shares list

# Policies
lfs policy create <name> --template <project-collab|personal|compliance>
lfs policy apply <ref> <policy-name>
lfs policy remove <ref> <policy-name>

# Trust
lfs trust get <ref>
lfs trust set <ref> <trusted|untrusted|quarantined|approved>
lfs quarantine list

# Import/Export
lfs import <path> [--tag <key:value>...]
lfs export <ref> --output <path> [--mode <tree|archive>]

# FUSE mounting
lfs mount [<mount-point>]     # Default: ~/Lattice
lfs unmount [<mount-point>]

# System
lfs init                      # Initialize repo
lfs status                    # Show repo stats
lfs gc                        # Garbage collect orphaned chunks
lfs verify                    # Check integrity
```

### 6. New Section: Testing Strategy

#### Unit Tests (Property-Based with proptest)
```rust
// Test invariants that must always hold

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        // Chunking is deterministic
        #[test]
        fn chunking_is_deterministic(data: Vec<u8>) {
            let chunks1 = chunk_data(&data);
            let chunks2 = chunk_data(&data);
            prop_assert_eq!(chunks1, chunks2);
        }

        // Deduplication works
        #[test]
        fn duplicate_chunks_share_storage(data: Vec<u8>) {
            let obj1 = store.add(&data);
            let obj2 = store.add(&data);
            let size_before = store.total_size();
            // Should not increase storage
            prop_assert_eq!(size_before, store.total_size());
        }

        // Policy evaluation is monotonic
        #[test]
        fn policies_never_grant_access(policies: Vec<Policy>) {
            let perms1 = evaluate(&policies[0..1]);
            let perms2 = evaluate(&policies);
            // Adding policies only restricts
            prop_assert!(perms2.is_subset_of(&perms1));
        }

        // Version DAG is acyclic
        #[test]
        fn version_dag_has_no_cycles(versions: Vec<Version>) {
            for v in &versions {
                prop_assert!(!has_ancestor_cycle(v));
            }
        }
    }
}
```

#### Integration Tests
```bash
# Test full workflows
tests/
├── test_add_and_retrieve.sh
├── test_versioning.sh
├── test_view_queries.sh
├── test_sharing_workflow.sh
├── test_import_export.sh
├── test_policy_enforcement.sh
└── test_fuse_mount.sh
```

### 7. New Section: Build Order (Detailed)

#### Phase 1: Foundation (Week 1-2)
1. **Error types** (`base/src/error.rs`)
   - Define domain-specific error enums
   - Implement conversions and context

2. **Storage primitives** (`base/src/storage/`)
   - Implement FastCDC chunking
   - BLAKE3 content addressing
   - Chunk store with deduplication
   - sled metadata store

3. **Data model** (`base/src/model/`)
   - Object, Version, Link, Tag structs
   - Serialization (serde)
   - Version DAG implementation

#### Phase 2: Core Logic (Week 3-4)
4. **Crypto** (`base/src/crypto/`)
   - Ed25519 identity management
   - OS keyring integration
   - AES-GCM encryption
   - UCAN token implementation

5. **Query engine** (`base/src/query/`)
   - LQL parser (recursive descent)
   - Query evaluator
   - Explainability

6. **Views** (`base/src/views/`)
   - Dynamic view system
   - Built-in views (Recent, Projects, etc.)
   - View snapshots

#### Phase 3: CLI & FUSE (Week 5-6)
7. **CLI commands** (`cli/src/commands/`)
   - Implement all commands from §5
   - Verbosity control
   - Progress indicators for long ops

8. **FUSE mount** (`base/src/fuse/`)
   - Hash-based inode mapping
   - Read-only operations
   - View projection

9. **Import/Export** (`base/src/import/`)
   - Filesystem scanner
   - Metadata extraction (EXIF, ID3, text)
   - Export to tree/archive

#### Phase 4: Policies & Sharing (Week 7-8)
10. **Policy engine** (`base/src/policy/`)
    - Policy evaluation (most restrictive wins)
    - Quota enforcement
    - Rate limiting

11. **Event system** (`base/src/events/`)
    - Tokio mpsc event bus
    - Event types
    - Logging

12. **Go share server** (`services/`)
    - gRPC API definition
    - HTTP share endpoint
    - Capability verification
    - Unix socket IPC with Rust

#### Phase 5: Testing & Polish (Week 9-10)
13. **Property-based tests**
    - Chunking determinism
    - Deduplication correctness
    - Policy monotonicity
    - DAG acyclicity

14. **Integration tests**
    - Full workflow tests
    - FUSE testing
    - Multi-process coordination

15. **Documentation**
    - CLI help text
    - Examples
    - Architecture diagrams

### 8. New Section: Success Criteria

#### MVP is complete when:
- ✅ User can `lfs import ~/Documents` and files are chunked/stored
- ✅ User can `lfs view create "Projects" --query 'tag:project'`
- ✅ User can `lfs mount ~/Lattice` and browse views read-only
- ✅ User can `lfs share <ref> --to <pubkey>` and recipient can fetch
- ✅ User can `lfs versions <ref>` and see full history
- ✅ User can `lfs diff <ref>@v1 <ref>@v2` and see changes
- ✅ User can `lfs policy apply <ref> project-collab`
- ✅ User can `lfs trust set <ref> quarantined` and execution is blocked
- ✅ All property-based tests pass (1000+ random inputs)
- ✅ Integration test suite passes (full workflows)

#### Performance targets:
- Import 1GB of mixed files in < 30 seconds
- Query results return in < 100ms for 100k objects
- FUSE read latency < 10ms for cached chunks
- Share server handles 100 concurrent requests
- Deduplication ratio > 1.5x for typical document sets

### 9. New Section: Known Limitations (MVP)

**Out of Scope for MVP:**
- ❌ Bi-directional sync between devices (CRDT protocol designed but not implemented)
- ❌ Write operations through FUSE (read-only mount only)
- ❌ Semantic embeddings / ML-powered search
- ❌ Web UI (CLI only)
- ❌ Hardware attestation for trust levels
- ❌ Distributed federation
- ❌ Semantic diffs (structural diffing)
- ❌ Multi-user quotas (single-user only)
- ❌ Real-time collaboration

**Technical Debt Accepted for MVP:**
- Simple LRU cache (not optimized eviction)
- No incremental garbage collection
- Basic anomaly detection for ransomware
- Text extraction without NLP preprocessing

## Critical Files to Create/Modify

### New Files (50+ files)
All files in the module architecture (§2) need to be created from scratch.

### Modified Files
1. `Cargo.toml` - Add new dependencies
2. `base/Cargo.toml` - Add base dependencies
3. `cli/Cargo.toml` - Add CLI dependencies
4. `services/go.mod` - Add Go dependencies
5. `specs/PRD.md` - Add all enrichments from this plan

### New Directories
```
base/src/storage/
base/src/model/
base/src/policy/
base/src/crypto/
base/src/query/
base/src/views/
base/src/fuse/
base/src/import/
base/src/events/
cli/src/commands/
services/cmd/lfs-share/
services/internal/api/
services/internal/share/
services/proto/
tests/integration/
```

## Verification Plan

### Manual Testing
1. Initialize fresh repo: `lfs init`
2. Import sample files: `lfs import ~/test-data`
3. Create view: `lfs view create "Images" --query 'type:image/*'`
4. Mount FUSE: `lfs mount ~/Lattice && ls ~/Lattice/views/Images`
5. Create share: `lfs share <ref> --to <pubkey> --expires 1h`
6. Verify policy: `lfs policy apply <ref> project-collab && lfs get <ref>` (should respect policy)
7. Check quarantine: `lfs trust set <ref> quarantined && ~/Lattice/views/Recent/<ref>` (should fail if executable)

### Automated Testing
```bash
cargo test --all                    # Unit + property tests
cargo test --all -- --ignored       # Slow integration tests
./tests/integration/run_all.sh      # Full workflow tests
cargo clippy --all-targets          # Linting
cargo fmt --check                   # Formatting
cargo audit                         # Security advisories
```

### Performance Testing
```bash
./bench/import_benchmark.sh         # Import 1GB test data
./bench/query_benchmark.sh          # Query 100k objects
./bench/fuse_benchmark.sh           # FUSE read latency
./bench/share_benchmark.sh          # Share server concurrency
```

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| FastCDC edge cases | Property testing with random inputs |
| FUSE stability | Extensive testing, read-only reduces risk |
| Key management bugs | OS keyring is battle-tested |
| Policy bypass | Formal model + extensive tests |
| Performance issues | Benchmarking from day 1 |
| Scope creep | Strict MVP boundary, defer to v1.1+ |

## Success Metrics

**Quantitative:**
- All 50+ unit tests pass
- All 10+ integration tests pass
- Property tests pass with 1000+ random inputs
- Import throughput > 30 MB/s
- Query latency < 100ms (p99)

**Qualitative:**
- Can migrate real Documents folder
- Views feel instant to navigate
- Sharing "just works" with capabilities
- Policy enforcement is intuitive
- CLI help is clear and complete

---

## Next Steps After Plan Approval

1. **Update PRD** - Add all sections from this plan to `specs/PRD.md`
2. **Create module structure** - Scaffold all directories and mod.rs files
3. **Add dependencies** - Update all Cargo.toml and go.mod files
4. **Begin Phase 1** - Start with storage layer (chunking + content addressing)
5. **Set up CI** - GitHub Actions for test/lint/benchmark on every PR

This plan represents approximately **10 weeks of focused development** for a single engineer, or **4-6 weeks with a small team**.
