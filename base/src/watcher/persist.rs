use crate::error::Result;
use crate::watcher::registry::{WatchEntry, WatchRegistry};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct PersistentRegistry {
    path: PathBuf,
}

impl PersistentRegistry {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load registry from disk, pruning entries whose temp files no longer exist.
    pub fn load(&self) -> Result<WatchRegistry> {
        if !self.path.exists() {
            return Ok(WatchRegistry::new());
        }

        let contents = std::fs::read_to_string(&self.path)?;
        let entries: HashMap<PathBuf, WatchEntry> = serde_json::from_str(&contents)
            .map_err(|e| crate::error::LatticeError::Serialization(format!("Failed to parse watcher registry: {}", e)))?;

        // Prune entries whose temp files no longer exist
        let pruned: HashMap<PathBuf, WatchEntry> = entries
            .into_iter()
            .filter(|(path, _)| path.exists())
            .collect();

        Ok(WatchRegistry::from_entries(pruned))
    }

    /// Save registry snapshot to disk.
    pub fn save(&self, registry: &WatchRegistry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let snapshot = registry.snapshot();
        let contents = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| crate::error::LatticeError::Serialization(format!("Failed to serialize watcher registry: {}", e)))?;
        std::fs::write(&self.path, contents)?;
        Ok(())
    }

    /// Delete the registry file.
    pub fn clear(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
