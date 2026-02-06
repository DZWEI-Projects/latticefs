pub mod config;
pub mod crypto;
pub mod error;
pub mod events;
pub mod fuse;
pub mod import;
pub mod ipc;
pub mod model;
pub mod policy;
pub mod query;
pub mod repo;
pub mod security;
pub mod storage;
pub mod views;
pub mod watcher;

// Re-export commonly used types
pub use error::{LatticeError, Result};

// Config + repo
pub use config::{
    Config, ExperimentalConfig, FuseConfig, ImportConfig, NestedViewsExperimentalConfig,
    QuotaConfig, ShareConfig, StorageConfig, WatcherConfig,
};
pub use events::{Event, EventBus, EventKind};
pub use policy::{
    PolicyContext, PolicyDecision, PolicyEngine, QuotaEnforcer, QuotaReport, RateLimiter,
};
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

// Security helpers
pub use security::{has_executable_tag, is_quarantined_executable, trust_level};

// FUSE helpers
pub use fuse::mount_fs;
#[cfg(feature = "fuse")]
pub use fuse::LatticeFS;

// Query types
pub use query::{
    parse, CompareOp, Explainer, Explanation, Expr, Lexer, MimePattern, ObjectRef, OrderBy, Parser,
    Predicate, Query, QueryEvaluator, Reason, SortDirection, SortField, TimeField, TimeOp,
    TimeValue, Token, TrustLevel,
};

// View types
pub use views::{
    BuiltinView, BuiltinViews, DynamicView, EffectiveQueryOptions, View, ViewConfig, ViewID,
    ViewJoinOperator, ViewSnapshot, ViewStore, DEFAULT_MAX_PARENT_DEPTH,
};
