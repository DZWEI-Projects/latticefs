# Benchmarking (Rust)

## Scope

This document covers Rust-side performance for the NeuralFS core (`base`) library, with a focus on
chunking, hashing, metadata, and repository read/write paths. Go services are out of scope for now.

## Benchmark suite overview

The suite is implemented as Criterion benchmarks under `base/benches/performance.rs`. It targets:

- **Chunking**: FastCDC boundary detection on 16 MiB buffers.
- **Hashing**: BLAKE3 hash on 16 MiB buffers.
- **Merkle root**: Merkle root calculation over chunk hashes from a 16 MiB buffer.
- **Repo store**: `LatticeRepo::store_object_data` on 8 MiB buffers (includes chunking + write).
- **Repo read**: `LatticeRepo::read_object_data` on 8 MiB buffers (chunk reassembly + read).
- **Metadata**: Store + load a single object in sled (metadata tree).
- **Manifest**: Serialize + deserialize `ChunkManifest` using `bincode`.

### How to run

```bash
cargo bench -p base --bench performance
```

Criterion reports are written to `target/criterion/` (including HTML charts).

## Environment used for the numbers below

- CPU: Intel(R) Xeon(R) Platinum 8370C @ 2.80GHz (3 vCPUs) (KVM) — `lscpu`.
- Rust: `rustc 1.89.0 (29483883e 2025-08-04)`.

## Results (Criterion, 100 samples)

Times are reported as `[low, median, high]` from Criterion’s analysis.

| Benchmark | Median time | Notes |
| --- | --- | --- |
| `chunking/chunk_data_16mb` | **14.700 ms** | 2% outliers (mild). |
| `hashing/compute_hash_16mb` | **6.8127 ms** | 10% outliers (mild/severe). |
| `merkle/compute_merkle_root_16mb` | **18.969 µs** | 6% outliers. |
| `repo/repo_store_object_8mb` | **18.574 ms** | 12% outliers (mild/severe). |
| `repo/repo_read_object_8mb` | **12.255 ms** | 14% outliers (mild/severe). |
| `metadata/store_load_object` | **2.7328 ms** | 14% outliers (mild/severe). |
| `manifest/manifest_encode_decode` | **26.812 µs** | 5% outliers. |

## Evaluation & opinions

### Observations

1. **Chunking + hashing are the dominant per-object CPU costs**, as expected. The 16 MiB chunking pass
   is ~2.2x the hash pass; both are large relative to metadata or Merkle costs.
2. **Repo store/read are primarily I/O + chunking**. The 8 MiB store time is in the high teens of ms,
   which roughly scales with chunking + write overhead. Reads show wider variance (higher outliers),
   suggesting filesystem/cache effects dominate.
3. **Merkle root calculation is negligible** compared to chunking/hash (tens of microseconds).
4. **Metadata store/load is slower than expected for a single object** (median ~2.7 ms), which likely
   reflects sled’s transaction overhead and filesystem sync behavior in short-lived DBs.
5. **Manifest serialization is small** and not a significant bottleneck for the tested size.

### What this means for end-user commands

Most CLI commands that ingest or export data will be dominated by:

- **FastCDC chunking** (CPU-bound).
- **BLAKE3 hashing** (CPU-bound).
- **Chunk I/O** to the content store (I/O-bound).

Metadata-heavy commands (tagging, small-object churn, or queries) may experience noticeable cost if
they perform frequent sled writes/reads.

## Improvement opportunities (priority × complexity)
>
> Priority: **P0** (must), **P1** (should), **P2** (nice-to-have).  
> Complexity: **S** (small), **M** (medium), **L** (large).

| Area | Why it matters (benchmark reference) | Priority | Complexity |
| --- | --- | --- | --- |
| **Parallelize chunking + hashing** | `chunk_data_16mb` + `compute_hash_16mb` dominate CPU time. Parallelizing per-chunk hashing (and possibly chunk boundary detection) would reduce wall time on multi-core systems. | P0 | M |
| **Batch metadata writes** | `metadata/store_load_object` shows ms-level overhead per object. Grouping metadata ops in batched transactions or caching hot objects can reduce repeated writes. | P0 | M |
| **Reuse a persistent tokio runtime in benchmarks & hot paths** | Repo benchmarks currently create a runtime per iteration; real code also spawns runtimes in CLI. Pooling/long-lived runtime reduces overhead. | P1 | M |
| **Improve chunk store I/O (fsync strategy, write buffering)** | `repo_store_object_8mb` cost implies I/O sensitivity. Consider write buffering or optional fsync in non-critical paths. | P1 | L |
| **Tune FastCDC parameters per file size** | Chunking cost is high; dynamic min/avg/max sizing for small files may reduce CPU for small/medium objects. | P1 | M |
| **Evaluate `sled` configuration** | Metadata costs could be improved by adjusting `cache_capacity`, `flush_every_ms`, or migrating hot paths to in-memory caches. | P1 | M |
| **Checksum/manifest caching** | Reusing computed hashes/manifests across operations would reduce recomputation. | P2 | M |
| **I/O parallelism on read** | `repo_read_object_8mb` variance suggests single-threaded reads; parallel reassembly and read-ahead could help. | P2 | L |

## Next steps

1. Expand the suite with CLI-level benchmarks (import/export, query, tag operations).
2. Add profile-guided optimization (PPROF or `perf`) alongside Criterion for CPU flame graphs.
3. Track benchmarks in CI (with a baseline) once stable workloads are defined.
