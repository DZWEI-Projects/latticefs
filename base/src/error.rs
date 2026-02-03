use thiserror::Error;

#[derive(Error, Debug)]
pub enum LatticeError {
    // Storage errors
    #[error("Chunk not found: {hash}")]
    ChunkNotFound { hash: String },

    #[error("Corrupted chunk: expected {expected}, got {computed}")]
    CorruptedChunk { expected: String, computed: String },

    #[error("Hash mismatch during write")]
    HashMismatch,

    #[error("Storage quota exceeded: {current_bytes} / {max_bytes}")]
    QuotaExceeded { current_bytes: u64, max_bytes: u64 },

    // Object/Version errors
    #[error("Object not found: {id}")]
    ObjectNotFound { id: String },

    #[error("Version not found: {id}")]
    VersionNotFound { id: String },

    #[error("Cyclic version detected")]
    CyclicVersion,

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Object is sealed and cannot be updated: {id}")]
    ObjectSealed { id: String },

    // Manifest errors
    #[error("Manifest not found: {hash}")]
    ManifestNotFound { hash: String },

    #[error("Merkle root mismatch")]
    MerkleRootMismatch,

    #[error("Length mismatch in chunk")]
    LengthMismatch,

    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] sled::Error),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    // Hex decoding errors
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    // Crypto errors (Phase 2)
    #[error("Identity not found: {name}")]
    IdentityNotFound { name: String },

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("Encryption failed: {0}")]
    Encryption(String),

    #[error("Decryption failed: {0}")]
    Decryption(String),

    #[error("Invalid signature")]
    InvalidSignature,

    // Capability errors (Phase 2)
    #[error("Capability expired")]
    CapabilityExpired,

    #[error("Capability revoked")]
    CapabilityRevoked,

    #[error("Capability not yet valid")]
    CapabilityNotYetValid,

    #[error("Capability not found: {cid}")]
    CapabilityNotFound { cid: String },

    #[error("Revocation not found: {cid}")]
    RevocationNotFound { cid: String },

    #[error("Invalid capability attenuation: cannot escalate from {from} to {to}")]
    InvalidAttenuation { from: String, to: String },

    #[error("Invalid proof chain: {0}")]
    InvalidProofChain(String),

    #[error("Invalid revocation signature")]
    InvalidRevocationSignature,

    #[error("Unauthorized: missing {permission} permission for {object}")]
    Unauthorized { permission: String, object: String },

    // Query errors (Phase 2)
    #[error("Parse error at position {position}: {message}")]
    ParseError { position: usize, message: String },

    #[error("Query timeout after {seconds}s")]
    QueryTimeout { seconds: u64 },

    #[error("Traversal depth exceeded: max {max} hops")]
    TraversalDepthExceeded { max: usize },

    #[error("Invalid predicate: {0}")]
    InvalidPredicate(String),

    // View errors (Phase 2)
    #[error("View not found: {name}")]
    ViewNotFound { name: String },

    #[error("Invalid view query: {0}")]
    InvalidViewQuery(String),
}

pub type Result<T> = std::result::Result<T, LatticeError>;
