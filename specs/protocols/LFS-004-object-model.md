# LFS-004: Object Model Protocol

**Status:** Draft
**Version:** 0.1.0
**Date:** 2026-02-03
**Authors:** NeuralFS Team

---

## Abstract

This document specifies the NeuralFS object model, including object types, version DAG semantics, graph relationships, tagging system, and state transitions. The object model replaces hierarchical filesystems with a semantic graph where objects are discovered through relationships and metadata.

---

## 1. Introduction

### 1.1 Motivation

Traditional filesystems organize data hierarchically:

- Files in directories
- Single parent (no multi-parent links)
- Location-based discovery (`/path/to/file`)
- Implicit relationships

Latt iceFS uses a graph model:

- Objects with versioned content
- Multiple relationships (DAG, not tree)
- Query-based discovery (LQL)
- Explicit, typed relationships

### 1.2 Core Concepts

- **Object**: Immutable identity with mutable versions
- **Version**: Snapshot of content at a point in time
- **Link**: Typed relationship between objects
- **Tag**: Key-value metadata for classification
- **State**: Lifecycle stage (draft → review → approved → archived)

### 1.3 Immutability Guarantee

**Content is immutable.** Once a version is created, its content never changes. Modifications create new versions.

**Benefits:**

- Safe concurrent access
- Instant rollback
- Cryptographic integrity
- Audit trail

---

## 2. Object Identity

### 2.1 Object ID

Every object has a unique identifier:

```rust
struct ObjectID(Uuid);  // UUID v7 (time-ordered)
```

**UUID v7 Format:**

```
01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e
└─ 48-bit timestamp ─┘ └─ random ─┘
```

**Properties:**

- Globally unique (collision probability < 2^-64)
- Sortable by creation time
- No central coordination required

### 2.2 Object Structure

```rust
struct Object {
    id: ObjectID,
    created_at: Timestamp,
    created_by: ActorID,
    object_type: ObjectType,
    current_version: VersionID,
    versions: Vec<VersionID>,
    tags: Vec<Tag>,
    links: Vec<Link>,
    policy_refs: Vec<PolicyID>,
    metadata_partition: MetadataPartition,
}
```

### 2.3 Object Types

```rust
enum ObjectType {
    Blob,      // Arbitrary binary data (files)
    Tree,      // Collection of objects (future: directories)
    Commit,    // Symbolic commit point (future: git-like)
}
```

**MVP:** Only `Blob` is implemented.

---

## 3. Versioning

### 3.1 Version Identity

```rust
struct VersionID(Uuid);  // UUID v7
```

Versions are also time-ordered UUIDs.

### 3.2 Version Structure

```rust
struct Version {
    id: VersionID,
    object_id: ObjectID,
    parent_version: Option<VersionID>,
    chunk_root: Hash,
    manifest_ref: Hash,
    created_at: Timestamp,
    created_by: ActorID,
    state: State,
    encrypted: bool,
    encryption_key_ref: Option<KeyID>,
    size_bytes: u64,
    chunk_count: u32,
    commit_message: Option<String>,
}
```

**Invariants:**

- `parent_version` forms a DAG (no cycles)
- First version has `parent_version = None`
- `chunk_root` is BLAKE3 Merkle root (LFS-001)
- `manifest_ref` points to ChunkManifest (LFS-001)

### 3.3 Version DAG

```
v1 ← v2 ← v3 ← v4     (linear history)

v1 ← v2 ← v3
      ↖     ↓
        v2' ← v4      (branching: v2' is alternate version)
```

**DAG Properties:**

- Acyclic: No version can be its own ancestor
- Connected: All versions reachable from current_version
- Ordered: Topological sort always possible

### 3.4 Version Creation

```rust
async fn create_version(
    object_id: ObjectID,
    parent: Option<VersionID>,
    content: &[u8],
    author: ActorID,
    message: Option<String>,
) -> Result<Version> {
    // 1. Chunk content
    let chunks = chunk_data(content);  // FastCDC (LFS-001)

    // 2. Store chunks
    let chunk_refs = store_chunks(&chunks).await?;

    // 3. Build manifest
    let manifest = ChunkManifest {
        version: 1,
        total_size: content.len() as u64,
        chunk_size_avg: 16384,
        chunks: chunk_refs.clone(),
        merkle_root: compute_merkle_root(&chunk_refs),
    };

    let manifest_hash = store_manifest(&manifest).await?;

    // 4. Create version
    let version = Version {
        id: VersionID::new_v7(),
        object_id,
        parent_version: parent,
        chunk_root: manifest.merkle_root,
        manifest_ref: manifest_hash,
        created_at: Timestamp::now(),
        created_by: author,
        state: State::Draft,
        encrypted: false,
        encryption_key_ref: None,
        size_bytes: content.len() as u64,
        chunk_count: chunks.len() as u32,
        commit_message: message,
    };

    // 5. Verify DAG acyclicity
    if let Some(parent_id) = parent {
        if is_ancestor(&version.id, &parent_id).await? {
            return Err(Error::CyclicVersion);
        }
    }

    // 6. Store version
    store_version(&version).await?;

    // 7. Update object's current_version
    update_object_version(object_id, version.id).await?;

    Ok(version)
}
```

---

## 4. States

### 4.1 State Enum

```rust
enum State {
    Draft,      // Work in progress
    Review,     // Under review
    Approved,   // Finalized
    Archived,   // No longer active
}
```

### 4.2 State Transitions

```
       ┌──────────────┐
       │    Draft     │
       └──────┬───────┘
              │
              ↓
       ┌──────────────┐
       │   Review     │
       └──────┬───────┘
              │
              ↓
       ┌──────────────┐
       │  Approved    │
       └──────┬───────┘
              │
              ↓
       ┌──────────────┐
       │  Archived    │
       └──────────────┘

Additional transitions:
  Draft → Archived (abandon)
  Review → Draft (send back)
  Approved → Archived (deprecate)
```

### 4.3 State Semantics

| State | Meaning | Immutable? | Discoverable? |
|-------|---------|------------|---------------|
| Draft | Work in progress | No | By creator only |
| Review | Awaiting approval | Partially | By reviewers |
| Approved | Finalized | Yes | Fully searchable |
| Archived | Deprecated | Yes | Hidden by default |

### 4.4 State Constraints

Policies can enforce state-based rules:

```yaml
# Example policy
allow:
  - read
deny:
  - write-new-version if state=approved
require:
  - approval-from: [alice, bob] for state=approved
```

---

## 5. Tags

### 5.1 Tag Structure

```rust
struct Tag {
    key: String,
    value: String,
    created_at: Timestamp,
    created_by: ActorID,
}
```

**Tag Format:** `<namespace>:<key>:<value>`

Examples:

```
tag:project:phoenix
tag:priority:high
tag:department:engineering
tag:status:active
```

### 5.2 Tag Hierarchies

Tags form namespaces:

```
tag:project
├── tag:project:phoenix
│   ├── tag:project:phoenix:deliverables
│   └── tag:project:phoenix:research
└── tag:project:apollo
```

**Query Semantics:**

- `tag:project` matches all project tags
- `tag:project:phoenix` matches phoenix and sub-tags
- `tag:project:phoenix:deliverables` matches exact tag only

### 5.3 Reserved Namespaces

```
sys:*       # System tags (immutable)
user:*      # User-defined tags
auto:*      # Auto-generated tags (EXIF, etc.)
```

### 5.4 Tag Operations

```rust
// Add tag
async fn add_tag(object_id: ObjectID, key: &str, value: &str) -> Result<()>;

// Remove tag
async fn remove_tag(object_id: ObjectID, key: &str) -> Result<()>;

// Query by tag
async fn query_by_tag(tag: &str) -> Result<Vec<ObjectID>>;
```

### 5.5 Tag Indexing

Tags MUST be indexed using an inverted index:

```
tag:project:phoenix → [obj1, obj2, obj5]
tag:priority:high   → [obj2, obj3, obj6]
tag:type:document   → [obj1, obj3, obj7]
```

**Index Structure:**

```rust
struct TagIndex {
    index: sled::Tree,  // tag → Vec<ObjectID>
}
```

---

## 6. Links (Graph Relationships)

### 6.1 Link Structure

```rust
struct Link {
    id: LinkID,
    source: ObjectID,
    target: ObjectID,
    link_type: LinkType,
    created_at: Timestamp,
    created_by: ActorID,
    metadata: Option<HashMap<String, String>>,
}
```

### 6.2 Link Types

```rust
enum LinkType {
    DerivedFrom,   // A is derived from B (e.g., PDF → report.docx)
    References,    // A references B (citation, dependency)
    BelongsTo,     // A belongs to collection B
    Replaces,      // A replaces B (successor)
    Related,       // General relationship
}
```

### 6.3 Link Semantics

| Link Type | Direction | Transitivity | Example |
|-----------|-----------|--------------|---------|
| DerivedFrom | A → B | No | `report.pdf` → `report.docx` |
| References | A → B | No | `paper.md` → `fig1.png` |
| BelongsTo | A → B | Yes | `file.txt` → `project/` |
| Replaces | A → B | Yes | `v2.pdf` → `v1.pdf` |
| Related | A ↔ B | No | `design.md` ↔ `impl.rs` |

### 6.4 Link Operations

```rust
// Create link
async fn create_link(
    source: ObjectID,
    target: ObjectID,
    link_type: LinkType,
) -> Result<Link>;

// Remove link
async fn remove_link(link_id: LinkID) -> Result<()>;

// Query links
async fn get_outgoing_links(object_id: ObjectID) -> Result<Vec<Link>>;
async fn get_incoming_links(object_id: ObjectID) -> Result<Vec<Link>>;
```

### 6.5 Graph Traversal

```rust
// Direct links (1-hop)
async fn references(object_id: ObjectID) -> Result<Vec<ObjectID>> {
    get_incoming_links(object_id).await?
        .into_iter()
        .filter(|l| l.link_type == LinkType::References)
        .map(|l| l.source)
        .collect()
}

// Transitive closure (all reachable objects)
async fn closure(object_id: ObjectID) -> Result<Vec<ObjectID>> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([object_id]);

    while let Some(current) = queue.pop_front() {
        if visited.contains(&current) {
            continue;  // Cycle detection
        }
        visited.insert(current);

        let links = get_outgoing_links(current).await?;
        for link in links {
            queue.push_back(link.target);
        }
    }

    Ok(visited.into_iter().collect())
}
```

---

## 7. Metadata Partitioning

### 7.1 Partitions

```rust
enum MetadataPartition {
    Private,    // Visible only to owner
    Shared,     // Visible to capability holders
    Public,     // Searchable by anyone
}
```

### 7.2 Partition Semantics

| Partition | Encrypted | Searchable | Shareable |
|-----------|-----------|------------|-----------|
| Private | Yes | Owner only | No |
| Shared | Yes | Capability holders | Via UCAN |
| Public | No | Everyone | Via UCAN or public |

### 7.3 Metadata Fields by Partition

**Private:**

- Encryption keys
- Personal notes
- Sensitive tags (e.g., `user:personal:medical`)

**Shared:**

- Project tags (e.g., `tag:project:phoenix`)
- Collaboration metadata
- Version history

**Public:**

- MIME type
- File size
- Creation timestamp
- Public tags (e.g., `tag:license:MIT`)

### 7.4 Partition Selection

```rust
fn determine_partition(object: &Object) -> MetadataPartition {
    // Check for sensitive tags
    if object.tags.iter().any(|t| t.key.starts_with("user:personal")) {
        return MetadataPartition::Private;
    }

    // Check for public tags
    if object.tags.iter().any(|t| t.key.starts_with("public:")) {
        return MetadataPartition::Public;
    }

    // Default: shared (most collaboration)
    MetadataPartition::Shared
}
```

---

## 8. Object Lifecycle

### 8.1 Creation

```rust
async fn create_object(
    content: &[u8],
    object_type: ObjectType,
    author: ActorID,
) -> Result<Object> {
    // 1. Generate ID
    let object_id = ObjectID::new_v7();

    // 2. Create initial version
    let version = create_version(object_id, None, content, author, None).await?;

    // 3. Create object
    let object = Object {
        id: object_id,
        created_at: Timestamp::now(),
        created_by: author,
        object_type,
        current_version: version.id,
        versions: vec![version.id],
        tags: vec![],
        links: vec![],
        policy_refs: vec![],
        metadata_partition: MetadataPartition::Shared,
    };

    // 4. Store object
    store_object(&object).await?;

    Ok(object)
}
```

### 8.2 Modification

```rust
async fn modify_object(
    object_id: ObjectID,
    new_content: &[u8],
    author: ActorID,
    message: String,
) -> Result<Version> {
    // 1. Load object
    let mut object = load_object(object_id).await?;

    // 2. Create new version
    let new_version = create_version(
        object_id,
        Some(object.current_version),
        new_content,
        author,
        Some(message),
    ).await?;

    // 3. Update object
    object.current_version = new_version.id;
    object.versions.push(new_version.id);
    store_object(&object).await?;

    Ok(new_version)
}
```

### 8.3 Deletion

**Deletion removes references, not content.**

```rust
async fn delete_object(object_id: ObjectID) -> Result<()> {
    // 1. Remove from indexes
    remove_from_tag_index(object_id).await?;
    remove_from_type_index(object_id).await?;

    // 2. Mark object as archived
    let mut object = load_object(object_id).await?;
    let version = load_version(object.current_version).await?;

    let archived_version = Version {
        state: State::Archived,
        ..version
    };

    store_version(&archived_version).await?;

    // 3. Chunks remain in storage (GC will collect later)
    Ok(())
}
```

### 8.4 Garbage Collection

```rust
async fn garbage_collect() -> Result<GCStats> {
    // 1. Mark: Find all reachable versions
    let mut reachable_versions = HashSet::new();
    for object in all_objects().await? {
        for version_id in &object.versions {
            reachable_versions.insert(*version_id);
        }
    }

    // 2. Mark: Find all reachable chunks
    let mut reachable_chunks = HashSet::new();
    for version_id in &reachable_versions {
        let version = load_version(*version_id).await?;
        let manifest = load_manifest(&version.manifest_ref).await?;
        for chunk_ref in &manifest.chunks {
            reachable_chunks.insert(chunk_ref.hash);
        }
    }

    // 3. Sweep: Remove unreachable chunks
    let mut removed_chunks = 0;
    for chunk_hash in all_stored_chunks().await? {
        if !reachable_chunks.contains(&chunk_hash) {
            remove_chunk(&chunk_hash).await?;
            removed_chunks += 1;
        }
    }

    Ok(GCStats {
        removed_chunks,
        bytes_freed: removed_chunks * 16384,  // Estimate
    })
}
```

---

## 9. Queries and Views

### 9.1 View Definition

```rust
struct View {
    name: String,
    query: String,  // LQL query (LFS-002)
    created_at: Timestamp,
    created_by: ActorID,
}
```

### 9.2 Built-in Views

```rust
// Recent: Last 100 updated objects
View {
    name: "Recent",
    query: "SORT updated DESC LIMIT 100",
}

// Projects: All project-tagged objects
View {
    name: "Projects",
    query: "tag:project",
}

// ByType: Group by MIME type
View {
    name: "ByType",
    query: "GROUP BY type",  // Future
}
```

### 9.3 Dynamic Views

Views are **dynamic**: They reflect current graph state.

```rust
async fn evaluate_view(view: &View) -> Result<Vec<ObjectID>> {
    let query = parse_lql(&view.query)?;
    execute_query(query).await
}
```

### 9.4 View Snapshots

For sharing, create **immutable snapshots**:

```rust
struct ViewSnapshot {
    view_name: String,
    snapshot_at: Timestamp,
    object_ids: Vec<ObjectID>,
    content_hashes: Vec<Hash>,
}
```

Snapshots freeze view results for sharing via UCANs (LFS-003).

---

## 10. Encryption

### 10.1 Per-Version Encryption

Each version can be encrypted independently:

```rust
struct EncryptedVersion {
    version: Version,
    encrypted_manifest: Vec<u8>,
    nonce: [u8; 12],
    key_ref: KeyID,
}
```

**Encryption:** AES-256-GCM (LFS-003)

**Key Management:**

- Keys stored in OS keyring
- Keys wrapped in UCAN tokens for sharing
- Key rotation via new versions

### 10.2 Encryption Workflow

```rust
async fn encrypt_version(
    version: &Version,
    encryption_key: &[u8; 32],
) -> Result<EncryptedVersion> {
    // 1. Load manifest
    let manifest = load_manifest(&version.manifest_ref).await?;

    // 2. Encrypt manifest
    let nonce = generate_nonce();
    let ciphertext = aes_gcm_encrypt(&manifest, encryption_key, &nonce)?;

    // 3. Store encrypted manifest
    let encrypted_manifest_hash = store_blob(&ciphertext).await?;

    Ok(EncryptedVersion {
        version: Version {
            encrypted: true,
            encryption_key_ref: Some(KeyID::from_hash(encryption_key)),
            manifest_ref: encrypted_manifest_hash,
            ..version.clone()
        },
        encrypted_manifest: ciphertext,
        nonce,
        key_ref: KeyID::from_hash(encryption_key),
    })
}
```

---

## 11. Policies

### 11.1 Policy Structure

```rust
struct Policy {
    id: PolicyID,
    name: String,
    version: u32,
    allow: Vec<Permission>,
    deny: Vec<Permission>,
    require: Vec<Requirement>,
    retain: Option<Duration>,
    external_share: bool,
}
```

### 11.2 Policy Application

Policies attach to objects:

```rust
struct Object {
    policy_refs: Vec<PolicyID>,
    // ...
}
```

Multiple policies: **Most restrictive wins** (LFS-003).

### 11.3 Example Policies

```yaml
# Project Collaboration Policy
name: project-collab
allow:
  - read
  - write-new-version
  - comment
deny:
  - delete
require:
  - approval-from: [lead-architect]
retain: 7y
external-share: false
```

---

## 12. Consistency Guarantees

### 12.1 ACID Properties

**Atomicity:** Object and version creation is atomic (sled transactions).

**Consistency:** DAG invariants enforced (no cycles).

**Isolation:** Concurrent modifications create separate versions.

**Durability:** All writes synced to disk (sled guarantees).

### 12.2 Concurrent Modifications

Two actors modifying the same object:

```
t0: Object v1
t1: Alice creates v2 (parent=v1)
t2: Bob creates v2' (parent=v1)

Result: Branching DAG
v1 ← v2
  ↖
    v2'
```

**Conflict Resolution:** Application-level (LQL queries can surface branches).

---

## 13. Performance Considerations

### 13.1 Indexing

- **Tag Index:** Inverted index (tag → objects)
- **Type Index:** MIME type → objects
- **Time Index:** Timestamp → objects (sorted)
- **Link Index:** Bidirectional (source → targets, target → sources)

### 13.2 Caching

- LRU cache for objects (10k entries)
- LRU cache for versions (50k entries)
- LRU cache for manifests (10k entries)

### 13.3 Lazy Loading

- Load versions on-demand (not all versions on object load)
- Load links on-demand (not all links on object load)

---

## 14. Test Vectors

### 14.1 Object Creation

```rust
let object = create_object(b"Hello, world!", ObjectType::Blob, alice).await?;
assert_eq!(object.versions.len(), 1);
assert_eq!(object.current_version, object.versions[0]);
```

### 14.2 Versioning

```rust
let v2 = modify_object(object.id, b"Hello, NeuralFS!", alice, "Update greeting").await?;
assert_eq!(v2.parent_version, Some(v1.id));
```

### 14.3 Cycle Detection

```rust
// Attempt to create cycle
let result = create_version(object.id, Some(v3.id), content, alice, None).await;
assert!(matches!(result, Err(Error::CyclicVersion)));
```

---

## Appendix A: Schema (sled keys)

```
objects/{object_id} → bincode(Object)
versions/{version_id} → bincode(Version)
manifests/{hash} → bincode(ChunkManifest)
tags/{tag_key} → Vec<ObjectID>
links/{link_id} → bincode(Link)
policies/{policy_id} → bincode(Policy)
```

---

**End of LFS-004**
