Perfect — good call on renaming `core` → `base`. That avoids namespace confusion and makes architectural discussions cleaner.

Below is a **single unified, handoff-ready PRD** that:

✅ incorporates your rename (`base`)
✅ merges _everything_ from the original PRD
✅ includes CLI, LQL, STRIDE, migration, automation
✅ includes language/framework decisions
✅ is structured for an autonomous coding agent
✅ ordered in build-sequence logic
✅ written as a real engineering artifact

This is intentionally long and explicit.

You can drop this directly into `/specs/PRD.md`.

---

# LatticeFS

## Product Requirements Document (Unified Engineering Edition)

Version: 0.1
Status: Buildable MVP Spec
Audience: Autonomous coding agent + human architects

---

# 1. Executive Summary

LatticeFS is a **post-file filesystem**.

Instead of folders and mutable files, it uses:

- Immutable versioned objects
- Content-addressed storage
- Semantic graph relationships
- Dynamic views (queries instead of directories)
- Capability-based security
- Policy-driven lifecycle
- Provenance-aware history

Traditional paths exist only as projections.

Meaning, time, trust, and sharing are first-class primitives.

LatticeFS is implemented entirely in **user space**, exposing compatibility via FUSE.

---

# 2. Design Goals

### Primary

1. Find data by meaning, not location
2. Universal versioning
3. Zero-trust sharing via capabilities
4. Local-first operation
5. Explainable state
6. Strong cryptographic integrity
7. AI/automation ready
8. Survive decades of format churn

### Non-goals

- Kernel filesystem
- Replacing POSIX semantics internally
- Magical AI inference
- Distributed federation (future phase)

---

# 3. Repository Layout

```
latticefs/
├── base/        # Rust: storage, DAG, policies, views, FUSE
├── cli/         # Rust CLI
├── services/    # Go: sync/share daemon
├── specs/       # PRD + protocols
├── docs/
└── Cargo.toml
```

---

# 4. Language & Framework Choices

## Primary: Rust (`base`, `cli`)

Used for:

- Chunk store
- Object graph
- Version DAG
- Policy engine
- View query engine
- Capability crypto
- FUSE mount

Reasons:

- Memory safety
- Strong typing for policy correctness
- Excellent crypto ecosystem
- Deterministic performance
- Safe concurrency

## Secondary: Go (`services`)

Used for:

- Sync orchestration
- Share gateway
- Admin APIs
- Telemetry

Reasons:

- Fast iteration
- Network services excellence
- Simple concurrency
- Easy deployment

## Minimal C (platform glue only)

Only for unavoidable OS hooks.

Must be:

- Small
- Audited
- Wrapped behind Rust FFI

---

# 5. Core Mental Model

Everything is an Object.

Objects form a graph.

Views are queries over that graph.

Security is capability-based.

History is immutable.

Deletion removes references, not content.

---

# 6. Data Model

## Object

```rust
struct Object {
  id: Hash,
  object_type: ObjectType,
  versions: Vec<VersionID>,
  tags: Vec<Tag>,
  links: Vec<Link>,
  policy_refs: Vec<PolicyID>,
}
```

## Version

```rust
struct Version {
  id: VersionID,
  parent: Option<VersionID>,
  chunk_root: Hash,
  created_at: Timestamp,
  created_by: Actor,
  state: State,
}
```

States:

- draft
- review
- approved
- archived

## Link Types

- DerivedFrom
- References
- BelongsTo
- Replaces

---

# 7. Storage Architecture

## Chunk Store

- BLAKE3 hashing
- Variable chunk size
- Deduplicated

Layout:

```
.latticefs/
├── chunks/aa/bb/hash
├── objects/
├── versions/
├── policies/
├── caps/
└── index/
```

Metadata stored via `sled`.

---

# 8. Views (Folders Are Dead)

Views are dynamic projections.

They behave like directories but are queries.

Examples:

- Recent
- Projects
- Shared
- ByType

Mounted via FUSE:

```
/views/
/projects/
/recent/
/by-type/
```

Writes create versions.

Deletes remove references.

---

# 9. LQL — Lattice Query Language

Human-readable graph query DSL.

### Primitives

- `tag:project:phoenix`
- `state:approved`
- `type:pdf`
- `trust>=medium`
- `updated within 7d`

### Boolean

AND OR NOT

### Traversal

```
references(<ref>)
closure(<ref>)
```

### Sorting

```
SORT updated DESC
GROUP BY type
LIMIT 200
```

### Explainability

Every result supports:

```
lfs view explain <ref>
```

Must output predicates + traversal path.

---

# 10. CLI

Examples:

```bash
lfs add report.pdf --tag project:phoenix
lfs tag <ref> topic:risk
lfs link <a> derived-from <b>
lfs versions <ref>
lfs diff <ref>@v3 <ref>@v7
lfs view create "Phoenix" --query 'tag:project:phoenix'
lfs share <ref> --cap read --to bob --expires 7d
lfs revoke <cap>
lfs policy apply <ref> project-collab
lfs trust set <ref> untrusted
lfs export "Phoenix" --mode tree
```

---

# 11. Capability Security Model

Sharing uses cryptographic tokens:

- scoped (read/write/comment)
- time-limited
- revocable
- optionally device-bound

No folder sharing.

Only object sharing.

Capabilities wrap object encryption keys.

---

# 12. Encryption

- AES-GCM per object
- Ed25519 identities
- Wrapped keys per capability

Metadata partitions:

- private
- share-visible
- public

---

# 13. Trust & Quarantine

Trust states:

- trusted
- untrusted
- quarantined
- approved

Executables blocked until approved.

Downloaded content defaults to untrusted.

---

# 14. Policy Engine

Declarative:

```yaml
allow:
  - read
  - write-new-version
retain: 7y
external-share: false
require:
  approval-for-state: approved
```

Policies attach to objects or views.

Never grant — only restrict.

---

# 15. STRIDE Threat Model

| Threat      | Mitigation             |
| ----------- | ---------------------- |
| Spoofing    | signed identities      |
| Tampering   | content hashes         |
| Repudiation | signed logs            |
| Disclosure  | scoped capabilities    |
| DoS         | quotas                 |
| Privilege   | capability enforcement |

Special protections:

- Ransomware: append-only + anomaly detection
- Token theft: revocation + device binding
- Metadata leaks: partitioned visibility

---

# 16. Automation

Event stream:

- object_created
- version_added
- share_issued
- policy_violation

Agents operate under explicit capabilities.

No silent actions.

All agent changes recorded as provenance.

---

# 17. Migration Plan

### Phase 0

Readonly indexing of existing trees.

### Phase 1

Hybrid folders.

### Phase 2

Default-save to LatticeFS.

### Phase 3

Team rollout.

### Phase 4

Legacy archive.

Export always available.

---

# 18. MVP Scope

### Required

✅ Chunk store
✅ Object model
✅ Versions
✅ Tags + links
✅ Views
✅ CLI
✅ FUSE
✅ Capabilities
✅ Policies
✅ Quarantine

### Deferred

❌ Federation
❌ UI
❌ Semantic diffs
❌ Hardware attestation

---

# 19. Build Order

1. Chunk store
2. Objects
3. Versions
4. Tags/links
5. Views/LQL
6. CLI
7. FUSE
8. Capabilities
9. Policies
10. Trust

---

# 20. Metrics

- Time-to-find
- Share mistakes
- Dedup ratio
- Rollback success
- Malware blocks

---

# Final Statement

LatticeFS is not a filesystem.

It is:

> A cryptographically verifiable semantic object graph
> that happens to expose POSIX paths for compatibility.

---

If you want next:

🔥 Rust module boundaries
🔥 exact LQL grammar
🔥 capability token format
🔥 protobuf sync protocol
🔥 FUSE inode strategy

Just say.
