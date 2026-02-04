# Benchmark-driven improvement plan (Rust)

This document proposes concrete performance improvements tied directly to the benchmark results
in `docs/benchmarking.md`. Each item describes the likely impact, the target benchmarks, and a
rough implementation approach.

## P0 — High priority

### 1) Parallel chunk hashing and Merkle construction
**Targets:** `chunking/chunk_data_16mb`, `hashing/compute_hash_16mb`, `repo/repo_store_object_8mb`  
**Why:** Chunking + hashing dominate CPU time. Parallelism should reduce wall-clock time on multi-core
systems (especially for large objects).

**Approach (rough):**
- When chunk boundaries are known, hash each chunk in parallel using rayon or tokio blocking tasks.
- Build Merkle roots in a parallel reduction (pairwise hashes per level).
- Ensure chunk ordering is preserved for deterministic manifests.

### 2) Batch metadata operations
**Targets:** `metadata/store_load_object`  
**Why:** Single-object round trips take milliseconds. Batching can reduce per-object overhead, especially
for imports or tag updates across many objects.

**Approach (rough):**
- Wrap multiple `store_*` calls in a single sled transaction.
- Add a metadata write queue with periodic flush.
- Introduce a small LRU cache for hot objects/versions to reduce load calls.

## P1 — Medium priority

### 3) Long-lived tokio runtime for CLI + benchmarks
**Targets:** `repo/repo_store_object_8mb`, `repo/repo_read_object_8mb`  
**Why:** Runtime creation is expensive and can inflate small/medium workloads.

**Approach (rough):**
- Construct a single `Runtime` in CLI main and pass it to command handlers or use `tokio::main` for
  full async execution.
- For benchmarks, keep a runtime per benchmark group rather than per iteration.

### 4) Chunk store write buffering and fsync control
**Targets:** `repo/repo_store_object_8mb`  
**Why:** I/O overhead can dominate storage operations.

**Approach (rough):**
- Add an opt-in buffered writer for chunk storage.
- Allow configurable fsync behavior (always / periodic / never in dev).
- Batch filesystem directory creation for chunk paths.

### 5) Adaptive FastCDC parameters for small files
**Targets:** `chunking/chunk_data_16mb` (and new small-file benchmarks)  
**Why:** Small files pay a similar chunking overhead despite smaller size.

**Approach (rough):**
- For files under a threshold, use larger minimum chunks or fall back to a single chunk path.
- Keep protocol compatibility by storing chosen chunking params in the manifest.

## P2 — Lower priority

### 6) Read-side parallelism and read-ahead
**Targets:** `repo/repo_read_object_8mb`  
**Why:** Read variance suggests I/O effects; parallelizing chunk reads could stabilize latency.

**Approach (rough):**
- Read chunks concurrently with a bounded task queue.
- Implement read-ahead for sequential access patterns.

### 7) Manifest + hash caching
**Targets:** `manifest/manifest_encode_decode`, `repo/repo_store_object_8mb`  
**Why:** Recomputing manifests and hashes across repeated operations is avoidable.

**Approach (rough):**
- Cache recent manifests keyed by object id + version id.
- Add a small in-memory hash cache when chunking in tight loops.

## Suggested follow-up benchmarks
1. CLI import/export benchmarks on a fixed dataset.
2. Metadata-heavy workloads (tagging, policy updates, queries).
3. Read amplification tests (many small chunks vs. few large chunks).
