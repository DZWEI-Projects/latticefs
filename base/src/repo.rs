//! Repository access helpers for LatticeFS.

use crate::config::{Config, default_home};
use crate::error::{LatticeError, Result};
use crate::storage::{ChunkManifest, ChunkStore, MetadataStore};
use std::path::{Path, PathBuf};

/// Opened LatticeFS repository.
pub struct LatticeRepo {
    pub root: PathBuf,
    pub config: Config,
    pub metadata: MetadataStore,
    pub chunks: ChunkStore,
}

impl LatticeRepo {
    /// Open an existing repo (or create directories if missing).
    pub fn open(config: Config) -> Result<Self> {
        let root = config.storage_path();
        ensure_layout(&root)?;
        let metadata = MetadataStore::open(&root)?;
        let chunks = ChunkStore::new(root.clone());
        Ok(Self {
            root,
            config,
            metadata,
            chunks,
        })
    }

    /// Initialize a repo using config from disk or defaults.
    pub fn init() -> Result<Self> {
        let config = Config::load_or_default()?;
        if !crate::config::config_path().exists() {
            config.write_default()?;
        }
        Self::open(config)
    }

    /// Open repo at an explicit root path (ignores config file).
    pub fn open_at(root: &Path) -> Result<Self> {
        let mut config = Config::default();
        config.storage.path = root.to_string_lossy().to_string();
        Self::open(config)
    }

    /// Read full object content from a manifest.
    pub async fn read_object_data(&self, manifest: &ChunkManifest) -> Result<Vec<u8>> {
        self.chunks.retrieve_object(manifest).await
    }

    /// Store bytes and return the manifest (chunks already written).
    pub async fn store_object_data(&self, data: &[u8]) -> Result<ChunkManifest> {
        self.chunks.store_object(data).await
    }
}

/// Ensure required directory layout exists under root.
fn ensure_layout(root: &Path) -> Result<()> {
    if root == Path::new("") || root == Path::new(".") {
        return Err(LatticeError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid repository root",
        )));
    }

    std::fs::create_dir_all(root)?;
    std::fs::create_dir_all(root.join("chunks"))?;
    std::fs::create_dir_all(root.join("logs"))?;
    std::fs::create_dir_all(root.join("meta"))?;

    Ok(())
}

/// Get the default LatticeFS root path.
pub fn default_repo_root() -> PathBuf {
    default_home()
}
