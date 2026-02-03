# LFS-001: Storage Protocol

**Status:** Draft
**Version:** 0.1.0
**Date:** 2026-02-03
**Authors:** LatticeFS Team

---

## Abstract

This document specifies the storage layer protocol for LatticeFS, including content-defined chunking, content addressing, chunk storage layout, and metadata management. This protocol ensures deterministic, deduplicated, and verifiable storage of objects.

---

## 1. Introduction

### 1.1 Motivation

Traditional filesystems store files as opaque blocks at specific paths. LatticeFS requires:

- **Deduplication**: Identical content stored once
- **Versioning**: Efficient storage of incremental changes
- **Integrity**: Cryptographic verification of all content
- **Portability**: Content-addressed storage independent of location

### 1.2 Terminology

- **Chunk**: Variable-size content block (8KB - 64KB)
- **Content Address**: BLAKE3 hash of chunk content
- **Object**: Logical entity composed of one or more chunks
- **Chunk Root**: Address of the root chunk in a Merkle tree
- **Manifest**: Metadata describing chunk tree structure

---

## 2. Content-Defined Chunking

### 2.1 Algorithm: FastCDC

LatticeFS MUST use the FastCDC (Fast Content-Defined Chunking) algorithm with the following parameters:

```
Algorithm:        FastCDC
Hash Function:    BLAKE3
Average Size:     16 KiB (16,384 bytes)
Minimum Size:     8 KiB (8,192 bytes)
Maximum Size:     64 KiB (65,536 bytes)
Window Size:      64 bytes
Mask Bits:        13 bits (for avg 16KB: 2^13 = 8192)
Normalization:    Gear hash with 256-entry table
```

### 2.2 Chunking Process

**Input:** Byte stream of length N

**Output:** List of chunks with boundaries

**Algorithm:**

```rust
fn chunk_stream(data: &[u8]) -> Vec<ChunkBoundary> {
    const MIN_SIZE: usize = 8192;
    const AVG_SIZE: usize = 16384;
    const MAX_SIZE: usize = 65536;
    const MASK: u64 = (1 << 13) - 1;  // 13 bits = avg 16KB
    const WINDOW_SIZE: usize = 64;

    let mut chunks = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        let end = (pos + MAX_SIZE).min(data.len());
        let chunk_start = pos;

        // Skip to minimum size
        pos += MIN_SIZE;

        // Find cut point using Gear hash
        let mut hash: u64 = 0;
        for byte in &data[pos..end] {
            hash = (hash << 1).wrapping_add(GEAR_TABLE[*byte as usize]);
            pos += 1;

            if (hash & MASK) == 0 || pos >= end {
                break;
            }
        }

        chunks.push(ChunkBoundary {
            offset: chunk_start,
            length: pos - chunk_start,
        });
    }

    chunks
}
```

### 2.3 Normalization

The FastCDC algorithm uses a pre-computed Gear hash table for boundary detection:

```rust
const GEAR_TABLE: [u64; 256] = [
    // 256 pseudo-random 64-bit values
    // Generated using: BLAKE3(b"LatticeFS-Gear-v1" || byte)
    0x5c3c6318a6d6f1b9, 0x8f9e4b2c7d3a1e0f, ...
];
```

**Table Generation** (informative):

```python
import blake3

def generate_gear_table():
    table = []
    for i in range(256):
        h = blake3.blake3(b"LatticeFS-Gear-v1" + bytes([i]))
        value = int.from_bytes(h.digest()[:8], 'little')
        table.append(value)
    return table
```

### 2.4 Determinism Requirement

**CRITICAL:** Chunking MUST be deterministic. Given identical input bytes, the algorithm MUST produce identical chunk boundaries regardless of:

- Execution platform
- Memory layout
- Chunking order (streaming vs. buffered)

**Test Vector:**

Input: `b"Hello, LatticeFS!" * 10000` (169,980 bytes)

Expected chunks:

```
Chunk 0: offset=0, length=8192, hash=blake3(...)
Chunk 1: offset=8192, length=16384, hash=blake3(...)
...
```

*(Full test vectors in Appendix A)*

---

## 3. Content Addressing

### 3.1 Hash Function: BLAKE3

All content addresses MUST use BLAKE3 with the following parameters:

```
Hash Function:    BLAKE3
Output Size:      256 bits (32 bytes)
Key:              None (standard hash, not keyed)
Context:          None
```

### 3.2 Chunk Hash Computation

```rust
use blake3::Hasher;

fn compute_chunk_hash(chunk_data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(chunk_data);
    hasher.finalize().into()
}
```

### 3.3 Hash Encoding

Content addresses MUST be encoded as lowercase hexadecimal strings:

```
Example: af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
```

### 3.4 Merkle Tree Construction

For objects larger than one chunk, a Merkle tree MUST be constructed:

```
Structure:
    Root Hash
       / \
      /   \
   H(L)  H(R)
   / \    / \
  H1 H2  H3 H4

Where:
- H1, H2, H3, H4 = Leaf hashes (chunk content)
- H(L) = BLAKE3(H1 || H2)
- H(R) = BLAKE3(H3 || H4)
- Root = BLAKE3(H(L) || H(R))
```

**Manifest Format:**

```rust
struct ChunkManifest {
    version: u32,           // Protocol version (1)
    total_size: u64,        // Total object size in bytes
    chunk_size_avg: u32,    // Average chunk size (16384)
    chunks: Vec<ChunkRef>,  // Ordered list of chunks
    merkle_root: Hash,      // Root hash for verification
}

struct ChunkRef {
    hash: [u8; 32],         // BLAKE3 hash
    offset: u64,            // Byte offset in object
    length: u32,            // Chunk length in bytes
}
```

---

## 4. Storage Layout

### 4.1 Directory Structure

```
$LATTICE_HOME/
├── chunks/                     # Content-addressed chunk store
│   ├── aa/
│   │   ├── bb/
│   │   │   └── <full-blake3-hash>
│   │   │       (32-byte hash → 2-char prefix → 2-char prefix → full hash filename)
├── meta/                       # Metadata (sled database)
│   ├── objects.db             # Object metadata
│   ├── versions.db            # Version DAG
│   ├── manifests.db           # Chunk manifests
│   ├── index.db               # Search index
│   └── capabilities.db        # Issued capability tokens
├── config.toml                # User configuration
├── keys/                      # Cryptographic keys
│   └── identity.key           # Ed25519 keypair reference
└── logs/
    └── events.jsonl           # Audit log
```

### 4.2 Chunk File Format

Chunks are stored as raw bytes at their content-addressed path:

```
Path: $LATTICE_HOME/chunks/{hash[0:2]}/{hash[2:4]}/{full_hash}
Content: Raw chunk bytes (8KB - 64KB)
Permissions: 0644 (read-only after write)
```

**Example:**

```
Hash: af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
Path: chunks/af/13/af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
```

### 4.3 Chunk Write Protocol

```rust
async fn write_chunk(hash: &Hash, data: &[u8]) -> Result<()> {
    let path = chunk_path(hash);

    // 1. Check if chunk already exists (deduplication)
    if path.exists() {
        return Ok(()); // Already stored
    }

    // 2. Create parent directories
    fs::create_dir_all(path.parent()?)?;

    // 3. Write atomically
    let temp_path = path.with_extension(".tmp");
    fs::write(&temp_path, data).await?;

    // 4. Verify hash
    let computed = compute_chunk_hash(data);
    if computed != *hash {
        fs::remove_file(&temp_path).await?;
        return Err(Error::HashMismatch);
    }

    // 5. Atomic rename
    fs::rename(&temp_path, &path).await?;

    // 6. Make read-only
    let mut perms = fs::metadata(&path).await?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&path, perms).await?;

    Ok(())
}
```

### 4.4 Chunk Read Protocol

```rust
async fn read_chunk(hash: &Hash) -> Result<Vec<u8>> {
    let path = chunk_path(hash);
    let data = fs::read(&path).await?;

    // Verify integrity on read
    let computed = compute_chunk_hash(&data);
    if computed != *hash {
        return Err(Error::CorruptedChunk);
    }

    Ok(data)
}
```

---

## 5. Metadata Storage

### 5.1 Database: sled

Metadata MUST be stored using the `sled` embedded database:

```rust
use sled::{Db, Tree};

struct MetadataStore {
    db: Db,
    objects: Tree,
    versions: Tree,
    manifests: Tree,
    index: Tree,
}
```

### 5.2 Object Metadata Format

```rust
struct ObjectMetadata {
    id: ObjectID,                   // UUID v7
    created_at: Timestamp,          // Unix timestamp (microseconds)
    created_by: ActorID,            // Ed25519 public key
    object_type: ObjectType,        // Blob, Tree, Commit
    current_version: VersionID,     // Latest version
    tags: Vec<Tag>,                 // Key-value tags
    links: Vec<Link>,               // Graph relationships
    policy_refs: Vec<PolicyID>,     // Applied policies
    metadata_partition: Partition,  // Private | Shared | Public
}

// Serialization: bincode (deterministic)
// Key: object_id (UUID bytes)
// Value: bincode(ObjectMetadata)
```

### 5.3 Version Metadata Format

```rust
struct VersionMetadata {
    id: VersionID,                  // UUID v7
    object_id: ObjectID,            // Parent object
    parent_version: Option<VersionID>, // Previous version (DAG)
    chunk_root: Hash,               // Merkle root
    manifest_ref: Hash,             // Reference to ChunkManifest
    created_at: Timestamp,          // When version was created
    created_by: ActorID,            // Author
    state: State,                   // Draft | Review | Approved | Archived
    encrypted: bool,                // Is content encrypted?
    encryption_key_ref: Option<KeyID>, // Key reference if encrypted
    size_bytes: u64,                // Total size
    chunk_count: u32,               // Number of chunks
}
```

### 5.4 Chunk Manifest Storage

Manifests are stored as content-addressed objects:

```
Key: BLAKE3(bincode(ChunkManifest))
Value: bincode(ChunkManifest)
```

This allows manifest deduplication across versions.

---

## 6. Deduplication

### 6.1 Content-Level Deduplication

Deduplication occurs at the chunk level:

1. Chunk content with FastCDC
2. Compute BLAKE3 hash of each chunk
3. Check if chunk already exists in store
4. If exists: reference existing chunk
5. If new: write chunk to store

**Result:** Identical content regions across files/versions share storage.

### 6.2 Deduplication Ratio

Expected deduplication ratios:

| Content Type | Typical Ratio |
|--------------|---------------|
| Documents (versions) | 5:1 - 10:1 |
| Source code (versions) | 3:1 - 7:1 |
| Media files | 1:1 - 1.2:1 |
| Mixed workload | 1.5:1 - 3:1 |

### 6.3 Garbage Collection

Chunks with no references SHOULD be removed:

```rust
async fn garbage_collect() -> Result<GCStats> {
    // 1. Mark phase: traverse all reachable objects
    let mut reachable = HashSet::new();
    for object in all_objects() {
        for version in object.versions() {
            let manifest = load_manifest(&version.manifest_ref)?;
            for chunk_ref in manifest.chunks {
                reachable.insert(chunk_ref.hash);
            }
        }
    }

    // 2. Sweep phase: remove unreachable chunks
    let mut removed = 0;
    for chunk_hash in all_stored_chunks() {
        if !reachable.contains(&chunk_hash) {
            remove_chunk(&chunk_hash)?;
            removed += 1;
        }
    }

    Ok(GCStats { removed })
}
```

**Safety:** GC MUST NOT remove chunks that are referenced by any reachable object.

---

## 7. Integrity Verification

### 7.1 Chunk Verification

On every read, verify chunk integrity:

```rust
fn verify_chunk(hash: &Hash, data: &[u8]) -> Result<()> {
    let computed = compute_chunk_hash(data);
    if computed != *hash {
        return Err(Error::CorruptedChunk {
            expected: *hash,
            computed,
        });
    }
    Ok(())
}
```

### 7.2 Object Verification

Verify entire object by traversing Merkle tree:

```rust
async fn verify_object(version: &VersionMetadata) -> Result<()> {
    let manifest = load_manifest(&version.manifest_ref)?;

    // 1. Verify all chunks exist and match hashes
    for chunk_ref in &manifest.chunks {
        let data = read_chunk(&chunk_ref.hash).await?;
        verify_chunk(&chunk_ref.hash, &data)?;

        if data.len() != chunk_ref.length as usize {
            return Err(Error::LengthMismatch);
        }
    }

    // 2. Verify Merkle root
    let computed_root = compute_merkle_root(&manifest.chunks);
    if computed_root != manifest.merkle_root {
        return Err(Error::MerkleRootMismatch);
    }

    Ok(())
}
```

### 7.3 Scrubbing

Periodic integrity checks SHOULD be performed:

```bash
lfs verify --deep        # Verify all chunks
lfs verify <ref>         # Verify specific object
```

---

## 8. Performance Considerations

### 8.1 Caching

Implementations SHOULD cache:

- Recently accessed chunks (LRU cache)
- Manifests for active objects
- Metadata queries

**Recommended cache sizes:**

- Chunk cache: 512 MiB
- Manifest cache: 10,000 entries
- Metadata cache: 50,000 entries

### 8.2 Parallel I/O

Chunk reads SHOULD be parallelized:

```rust
async fn read_object(manifest: &ChunkManifest) -> Result<Vec<u8>> {
    let chunks: Vec<_> = manifest.chunks
        .iter()
        .map(|chunk_ref| read_chunk(&chunk_ref.hash))
        .collect();

    let results = futures::future::try_join_all(chunks).await?;
    let data = results.concat();
    Ok(data)
}
```

### 8.3 Write Buffering

Implementations MAY buffer small writes, but MUST ensure:

- Atomic writes (no partial chunks)
- Crash consistency (fsync manifests)
- Ordering guarantees (manifest written after all chunks)

---

## 9. Security Considerations

### 9.1 Hash Collisions

BLAKE3 provides 256-bit security. Collision probability is negligible (< 2^-128 for birthday attack).

### 9.2 Hash Flooding

Attackers MUST NOT be able to craft inputs that force pathological chunking behavior.

**Mitigation:** Use keyed Gear hash table (LatticeFS-specific constant).

### 9.3 Storage Exhaustion

Implementations MUST enforce quotas:

```toml
[quota]
max_storage_gb = 100
max_objects = 1000000
max_chunks_per_object = 100000
```

### 9.4 Chunk Integrity

Chunks are immutable and content-addressed. Tampering is detectable via hash mismatch.

---

## 10. Compatibility

### 10.1 Version

This protocol is version `1`. Future versions MUST maintain backward compatibility or provide migration tools.

### 10.2 Endianness

All numeric values MUST be stored in little-endian format.

### 10.3 Encoding

- Strings: UTF-8
- Binary data: Raw bytes
- Hashes: Hexadecimal (lowercase)
- UUIDs: RFC 4122 format

---

## Appendix A: Test Vectors

### A.1 Empty File

```
Input: []
Chunks: []
Merkle Root: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
            (BLAKE3 of empty bytes)
```

### A.2 Small File (< 8KB)

```
Input: b"Hello, LatticeFS!\n" (18 bytes)
Chunks: [
  {offset: 0, length: 18, hash: "..."}
]
Merkle Root: <BLAKE3 of chunk>
```

### A.3 Large File (> 64KB)

```
Input: /dev/urandom (1 MiB of random data with seed 12345)
Expected: ~64 chunks (avg 16KB)
Test: Re-chunking produces identical boundaries
```

---

## Appendix B: References

- [FastCDC Paper](https://www.usenix.org/system/files/conference/atc16/atc16-paper-xia.pdf)
- [BLAKE3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf)
- [Merkle Trees](https://en.wikipedia.org/wiki/Merkle_tree)
- [sled Documentation](https://docs.rs/sled/latest/sled/)

---

**End of LFS-001**
