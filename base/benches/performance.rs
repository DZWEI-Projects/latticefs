use base::{
    chunk_data, compute_hash, compute_merkle_root, ChunkManifest, ChunkRef, LatticeRepo, Object,
    ObjectType,
};
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::path::PathBuf;
use tempfile::TempDir;

fn temp_repo_root() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create tempdir");
    let root = dir.path().join("repo");
    (dir, root)
}

fn benchmark_chunking(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunking");
    let data = vec![0u8; 16 * 1024 * 1024];

    group.bench_function("chunk_data_16mb", |b| {
        b.iter(|| {
            let _ = chunk_data(&data);
        });
    });

    group.finish();
}

fn benchmark_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashing");
    let data = vec![0u8; 16 * 1024 * 1024];

    group.bench_function("compute_hash_16mb", |b| {
        b.iter(|| {
            let _ = compute_hash(&data);
        });
    });

    group.finish();
}

fn benchmark_merkle(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle");
    let data = vec![0u8; 16 * 1024 * 1024];
    let boundaries = chunk_data(&data);
    let hashes: Vec<_> = boundaries
        .iter()
        .map(|chunk| compute_hash(&data[chunk.offset..chunk.offset + chunk.length]))
        .collect();

    group.bench_function("compute_merkle_root_16mb", |b| {
        b.iter(|| {
            let _ = compute_merkle_root(&hashes);
        });
    });

    group.finish();
}

fn benchmark_repo_store_and_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("repo");
    let data = vec![1u8; 8 * 1024 * 1024];

    group.bench_function("repo_store_object_8mb", |b| {
        b.iter_batched(
            || {
                let (_dir, root) = temp_repo_root();
                let repo = LatticeRepo::open_at(&root).expect("open repo");
                (repo, _dir)
            },
            |(repo, _dir)| {
                let runtime = tokio::runtime::Runtime::new().expect("runtime");
                runtime.block_on(async {
                    let _ = repo.store_object_data(&data).await.expect("store object");
                });
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("repo_read_object_8mb", |b| {
        b.iter_batched(
            || {
                let (_dir, root) = temp_repo_root();
                let repo = LatticeRepo::open_at(&root).expect("open repo");
                let runtime = tokio::runtime::Runtime::new().expect("runtime");
                let manifest = runtime
                    .block_on(async { repo.store_object_data(&data).await })
                    .expect("manifest");
                (repo, manifest, _dir)
            },
            |(repo, manifest, _dir)| {
                let runtime = tokio::runtime::Runtime::new().expect("runtime");
                runtime.block_on(async {
                    let _ = repo.read_object_data(&manifest).await.expect("read object");
                });
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn benchmark_metadata_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata");

    group.bench_function("store_load_object", |b| {
        b.iter_batched(
            || {
                let (_dir, root) = temp_repo_root();
                let repo = LatticeRepo::open_at(&root).expect("open repo");
                let object = Object::new(ObjectType::Blob, Default::default(), [0u8; 32]);
                (repo, object, _dir)
            },
            |(repo, object, _dir)| {
                repo.metadata.store_object(&object).expect("store object");
                let _ = repo.metadata.load_object(&object.id).expect("load object");
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn benchmark_manifest_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest");
    let data = vec![0u8; 8 * 1024 * 1024];
    let boundaries = chunk_data(&data);
    let chunks: Vec<ChunkRef> = boundaries
        .iter()
        .map(|chunk| ChunkRef {
            hash: compute_hash(&data[chunk.offset..chunk.offset + chunk.length]),
            offset: chunk.offset as u64,
            length: chunk.length as u32,
        })
        .collect();
    let manifest = ChunkManifest {
        version: 1,
        total_size: data.len() as u64,
        chunk_size_avg: 16 * 1024,
        chunks,
        merkle_root: compute_hash(b"root"),
    };

    group.bench_function("manifest_encode_decode", |b| {
        b.iter(|| {
            let encoded = bincode::serialize(&manifest).expect("serialize");
            let _: ChunkManifest = bincode::deserialize(&encoded).expect("deserialize");
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_chunking,
    benchmark_hashing,
    benchmark_merkle,
    benchmark_repo_store_and_read,
    benchmark_metadata_store,
    benchmark_manifest_serialization
);
criterion_main!(benches);
