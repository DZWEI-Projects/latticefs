//! Configuration handling for LatticeFS.
//!
//! Loads config from $LATTICE_HOME/config.toml (or ~/.latticefs/config.toml).
//! Environment variable LATTICE_HOME overrides the default storage path.

use crate::error::{LatticeError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub storage: StorageConfig,
    pub quota: QuotaConfig,
    pub fuse: FuseConfig,
    pub crypto: CryptoConfig,
    pub share: ShareConfig,
    pub import: ImportConfig,
    pub logging: LoggingConfig,
    pub ipc: IpcConfig,
    pub watcher: WatcherConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            quota: QuotaConfig::default(),
            fuse: FuseConfig::default(),
            crypto: CryptoConfig::default(),
            share: ShareConfig::default(),
            import: ImportConfig::default(),
            logging: LoggingConfig::default(),
            ipc: IpcConfig::default(),
            watcher: WatcherConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    pub path: String,
    pub cache_size_mb: u64,
    pub max_chunk_size_kb: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            path: "~/.latticefs".to_string(),
            cache_size_mb: 512,
            max_chunk_size_kb: 64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QuotaConfig {
    pub max_storage_gb: u64,
    pub max_operations_per_minute: u64,
    pub burst_allowance: u64,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            max_storage_gb: 100,
            max_operations_per_minute: 1000,
            burst_allowance: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FuseConfig {
    pub mount_point: String,
    pub readonly: bool,
    pub allow_other: bool,
}

impl Default for FuseConfig {
    fn default() -> Self {
        Self {
            mount_point: "~/Lattice".to_string(),
            readonly: true,
            allow_other: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CryptoConfig {
    pub algorithm: String,
    pub key_derivation: String,
    pub keyring_service: String,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            algorithm: "aes-256-gcm".to_string(),
            key_derivation: "argon2id".to_string(),
            keyring_service: "latticefs".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShareConfig {
    pub http_port: u16,
    pub max_concurrent_shares: u32,
    pub default_ttl_days: u32,
}

impl Default for ShareConfig {
    fn default() -> Self {
        Self {
            http_port: 8771,
            max_concurrent_shares: 10,
            default_ttl_days: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImportConfig {
    pub extract_exif: bool,
    pub extract_id3: bool,
    pub extract_text: bool,
    pub create_embeddings: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            extract_exif: true,
            extract_id3: true,
            extract_text: true,
            create_embeddings: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub audit_log: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            audit_log: "~/.latticefs/logs/events.jsonl".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IpcConfig {
    pub verbose: bool,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatcherConfig {
    pub enabled: bool,
    pub debounce_ms: u64,
    pub commit_message_template: String,
    pub watch_dir: String,
    pub ignored_patterns: Vec<String>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 1000,
            commit_message_template: "Auto-saved from external editor at {timestamp}".to_string(),
            watch_dir: "/tmp/latticefs-open".to_string(),
            ignored_patterns: vec![
                "*.swp".to_string(),
                "*.swo".to_string(),
                "*~".to_string(),
                ".DS_Store".to_string(),
                "*.tmp".to_string(),
                "*.bak".to_string(),
                ".~lock.*".to_string(),
            ],
        }
    }
}

impl Config {
    /// Load config from disk, or return defaults if missing.
    pub fn load() -> Result<Self> {
        let config_path = config_path();
        if !config_path.exists() {
            return Ok(Self::default_with_home());
        }

        let contents = std::fs::read_to_string(&config_path)?;
        let mut cfg: Config = toml::from_str(&contents)
            .map_err(|e| LatticeError::Serialization(format!("Failed to parse config: {}", e)))?;

        // Apply env override if present.
        if let Some(home) = env_home_override() {
            cfg.storage.path = home.to_string_lossy().to_string();
        }

        Ok(cfg)
    }

    /// Load config or create defaults, and ensure paths are expanded.
    pub fn load_or_default() -> Result<Self> {
        let cfg = Self::load()?;
        Ok(cfg)
    }

    /// Default config honoring LATTICE_HOME.
    pub fn default_with_home() -> Self {
        let mut cfg = Self::default();
        if let Some(home) = env_home_override() {
            cfg.storage.path = home.to_string_lossy().to_string();
        }
        cfg
    }

    /// Resolve the storage root path with ~ expansion.
    pub fn storage_path(&self) -> PathBuf {
        expand_tilde(&self.storage.path)
    }

    /// Resolve the FUSE mount point with ~ expansion.
    pub fn mount_point(&self) -> PathBuf {
        expand_tilde(&self.fuse.mount_point)
    }

    /// Resolve the audit log path with ~ expansion.
    pub fn audit_log_path(&self) -> PathBuf {
        expand_tilde(&self.logging.audit_log)
    }

    /// Resolve the IPC socket path.
    pub fn socket_path(&self) -> PathBuf {
        self.storage_path().join("latticefs.sock")
    }

    /// Resolve the revocation log path.
    pub fn revocation_log_path(&self) -> PathBuf {
        self.storage_path().join("logs").join("revocations.jsonl")
    }

    /// Resolve the watcher directory path.
    pub fn watch_dir(&self) -> PathBuf {
        expand_tilde(&self.watcher.watch_dir)
    }

    /// Resolve the watcher daemon PID file path.
    pub fn watchd_pid_path(&self) -> PathBuf {
        self.storage_path().join("watchd.pid")
    }

    /// Resolve the watcher registry file path.
    pub fn watch_registry_path(&self) -> PathBuf {
        self.storage_path().join("watcher_registry.json")
    }

    /// Write config to disk at the default location.
    pub fn write_default(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize config: {}", e))
        })?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

/// Resolve config path from LATTICE_HOME or default ~/.latticefs.
pub fn config_path() -> PathBuf {
    let home = env_home_override().unwrap_or_else(default_home);
    home.join("config.toml")
}

/// Default LatticeFS home (~/.latticefs).
pub fn default_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".latticefs")
}

fn env_home_override() -> Option<PathBuf> {
    std::env::var_os("LATTICE_HOME")
        .or_else(|| std::env::var_os("LATTICEFS_HOME"))
        .map(PathBuf::from)
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    Path::new(path).to_path_buf()
}
