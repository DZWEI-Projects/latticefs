pub mod crypto;
pub mod config;
pub mod error;
pub mod fuse;
pub mod import;
pub mod model;
pub mod query;
pub mod repo;
pub mod storage;
pub mod views;

// Re-export commonly used types
pub use error::{LatticeError, Result};

// Config + repo
pub use config::{Config, FuseConfig, ImportConfig, QuotaConfig, ShareConfig, StorageConfig};
pub use repo::LatticeRepo;

// Crypto types
pub use crypto::{
    Capability, EncryptedData, Identity, KeyManager, KeyStorage, ObjectKey, Permission, PublicKey,
    Revocation, RevocationChecker, RevocationList,
};

// Model types
pub use model::{
    timestamp_now, ActorID, KeyID, Link, LinkID, LinkType, MetadataPartition, Object, ObjectID,
    ObjectType, Policy, PolicyID, PolicyTemplate, Requirement, State, Tag, Timestamp, Version,
    VersionDAG, VersionID,
};

// Storage types
pub use storage::{
    chunk_data, compute_hash, compute_merkle_root, hash_to_hex, hex_to_hash, ChunkBoundary,
    ChunkManifest, ChunkRef, ChunkStore, Hash, MetadataStore,
};

// FUSE helpers
pub use fuse::mount_fs;
#[cfg(feature = "fuse")]
pub use fuse::LatticeFS;

// Query types
pub use query::{
    parse, CompareOp, Explanation, Explainer, Expr, Lexer, MimePattern, ObjectRef, OrderBy,
    Parser, Predicate, Query, QueryEvaluator, Reason, SortDirection, SortField, TimeField, TimeOp,
    TimeValue, Token, TrustLevel,
};

// View types
pub use views::{
    BuiltinView, BuiltinViews, DynamicView, View, ViewConfig, ViewID, ViewSnapshot, ViewStore,
};
