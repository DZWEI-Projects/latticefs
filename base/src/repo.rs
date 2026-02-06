//! Repository access helpers for LatticeFS.

use crate::config::{default_home, Config};
use crate::error::{LatticeError, Result};
use crate::events::{spawn_logger, Event, EventBus};
use crate::model::{ActorID, Object, ObjectID, State, Version};
use crate::policy::{PolicyContext, PolicyEngine, QuotaEnforcer, RateLimiter};
use crate::storage::{ChunkManifest, ChunkStore, Hash, MetadataStore};
use std::path::{Path, PathBuf};

/// Opened LatticeFS repository.
pub struct LatticeRepo {
    pub root: PathBuf,
    pub config: Config,
    pub metadata: MetadataStore,
    pub chunks: ChunkStore,
    pub events: EventBus,
    quota: QuotaEnforcer,
    rate_limiter: RateLimiter,
}

impl LatticeRepo {
    /// Open an existing repo (or create directories if missing).
    pub fn open(config: Config) -> Result<Self> {
        let root = config.storage_path();
        ensure_layout(&root)?;
        let metadata = MetadataStore::open(&root)?;
        let chunks = ChunkStore::new(root.clone());
        let (events, receiver) = EventBus::new(1024);
        spawn_logger(receiver, config.audit_log_path())?;
        let quota = QuotaEnforcer::new(config.quota.clone());
        let rate_limiter = RateLimiter::new(&config.quota);
        Ok(Self {
            root,
            config,
            metadata,
            chunks,
            events,
            quota,
            rate_limiter,
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
        self.enforce_rate_limit(1)?;
        self.quota.check_storage_quota(&self.chunks, data)?;
        self.chunks.store_object(data).await
    }

    /// Add a new version for an existing object using raw bytes.
    pub async fn add_version_from_bytes(
        &self,
        object_id: &ObjectID,
        data: &[u8],
        actor: ActorID,
        message: Option<String>,
    ) -> Result<Version> {
        self.quota.check_storage_quota(&self.chunks, data)?;
        let manifest = self.chunks.store_object(data).await?;
        let manifest_ref = self.metadata.store_manifest(&manifest)?;
        self.add_version_from_manifest(
            object_id,
            &manifest,
            manifest_ref,
            data.len() as u64,
            actor,
            message,
        )
    }

    /// Add a new version for an existing object using an existing manifest.
    pub fn add_version_from_manifest(
        &self,
        object_id: &ObjectID,
        manifest: &ChunkManifest,
        manifest_ref: Hash,
        size_bytes: u64,
        actor: ActorID,
        message: Option<String>,
    ) -> Result<Version> {
        self.enforce_rate_limit(1)?;
        let mut object = self.metadata.load_object(object_id)?;
        self.authorize_object_permission(&object, crate::crypto::Permission::Write, false)?;
        let mut current = self.metadata.load_version(&object.current_version)?;

        if current.state == State::Sealed {
            return Err(LatticeError::ObjectSealed {
                id: object_id.to_string(),
            });
        }

        let mut updated_current = false;
        match current.state {
            State::Review => {
                current.transition_state(State::Approved).map_err(|_| {
                    LatticeError::InvalidStateTransition {
                        from: "review".to_string(),
                        to: "approved".to_string(),
                    }
                })?;
                updated_current = true;
            }
            State::Draft => {
                current.transition_state(State::Discarded).map_err(|_| {
                    LatticeError::InvalidStateTransition {
                        from: "draft".to_string(),
                        to: "discarded".to_string(),
                    }
                })?;
                updated_current = true;
            }
            _ => {}
        }

        if updated_current {
            self.metadata.store_version(&current)?;
        }

        let version = Version::new(
            *object_id,
            Some(object.current_version),
            manifest.merkle_root,
            manifest_ref,
            actor,
            size_bytes,
            manifest.chunks.len() as u32,
            message,
        );

        object.add_version(version.id);
        self.metadata.store_version(&version)?;
        self.metadata.store_object(&object)?;
        self.events.emit_sync(Event::version_added(
            object_id,
            &version.id,
            version.parent_version.as_ref(),
            actor,
        ));
        Ok(version)
    }

    pub fn authorize_object_permission(
        &self,
        object: &Object,
        permission: crate::crypto::Permission,
        external_share: bool,
    ) -> Result<()> {
        let policies = self.load_policies_for_object(object)?;
        let context = PolicyContext::for_object(object).with_external_share(external_share);
        let engine = PolicyEngine::new();

        match engine.authorize(&policies, &context, permission) {
            Ok(()) => Ok(()),
            Err(LatticeError::PolicyViolation { reason }) => {
                self.events.emit_sync(Event::policy_violation(
                    Some(object.id.to_string()),
                    permission.to_string(),
                    reason.clone(),
                ));
                Err(LatticeError::PolicyViolation { reason })
            }
            Err(err) => Err(err),
        }
    }

    pub fn load_policies_for_object(&self, object: &Object) -> Result<Vec<crate::model::Policy>> {
        let mut policies = Vec::new();
        for policy_id in &object.policy_refs {
            let policy = self.metadata.load_policy_by_id(policy_id)?;
            policies.push(policy);
        }
        Ok(policies)
    }

    /// Enforce rate limiting with atomic compare-and-swap to prevent race conditions.
    ///
    /// This method uses optimistic locking to ensure that concurrent requests
    /// cannot bypass rate limits by reading stale state.
    pub fn enforce_rate_limit(&self, ops: u64) -> Result<()> {
        let rate_limiter = self.rate_limiter.clone();
        self.metadata
            .atomic_rate_limit_consume("default", move |state| {
                rate_limiter.check_and_consume(state, ops)
            })
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
