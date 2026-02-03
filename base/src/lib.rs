pub mod error;
pub mod model;
pub mod storage;

// Re-export commonly used types
pub use error::{LatticeError, Result};
pub use model::{
    timestamp_now, ActorID, KeyID, Link, LinkID, LinkType, MetadataPartition, Object, ObjectID,
    ObjectType, PolicyID, State, Tag, Timestamp, Version, VersionDAG, VersionID,
};
pub use storage::{
    chunk_data, compute_hash, compute_merkle_root, hash_to_hex, hex_to_hash, ChunkBoundary,
    ChunkManifest, ChunkRef, ChunkStore, Hash, MetadataStore,
};
