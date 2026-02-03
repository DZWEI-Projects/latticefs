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
}

pub type Result<T> = std::result::Result<T, LatticeError>;
