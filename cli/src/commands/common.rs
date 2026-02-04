use anyhow::{anyhow, Result};
use latticefs_base::{Config, KeyManager, LatticeRepo};
use latticefs_base::{Identity, ObjectID, VersionID};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn open_repo(repo_path: Option<PathBuf>) -> Result<LatticeRepo> {
    if let Some(path) = repo_path {
        return Ok(LatticeRepo::open_at(&path)?);
    }

    let config = Config::load_or_default()?;
    Ok(LatticeRepo::open(config)?)
}

pub fn ensure_identity(name: &str, password: Option<&str>) -> Result<Identity> {
    let manager = KeyManager::auto();
    let password = password.or_else(|| std::env::var("LFS_KEY_PASSWORD").ok().as_deref());
    if manager.exists(name) {
        return Ok(manager.load(name, password)?);
    }

    let identity = Identity::generate(name);
    manager.store(&identity, password)?;
    Ok(identity)
}

pub fn identity_actor(identity: &Identity) -> [u8; 32] {
    identity.public_bytes()
}

pub fn resolve_object_id(repo: &LatticeRepo, reference: &str) -> Result<ObjectID> {
    if let Ok(uuid) = uuid::Uuid::parse_str(reference) {
        return Ok(ObjectID::from_uuid(uuid));
    }

    if let Some(id_bytes) = repo.metadata.resolve_alias(reference)? {
        let uuid = uuid::Uuid::from_slice(&id_bytes)
            .map_err(|e| anyhow!("Invalid alias mapping '{}': {}", reference, e))?;
        return Ok(ObjectID::from_uuid(uuid));
    }

    Err(anyhow!("Unknown object reference: {}", reference))
}

pub fn parse_ref_with_version(repo: &LatticeRepo, reference: &str) -> Result<(ObjectID, Option<VersionID>)> {
    let parts: Vec<&str> = reference.splitn(2, '@').collect();
    let object_id = resolve_object_id(repo, parts[0])?;
    if parts.len() == 1 {
        return Ok((object_id, None));
    }

    let version_spec = parts[1];
    if let Ok(uuid) = uuid::Uuid::parse_str(version_spec) {
        return Ok((object_id, Some(VersionID::from_uuid(uuid))));
    }

    if let Some(version_id) = resolve_version_alias(repo, &object_id, version_spec)? {
        return Ok((object_id, Some(version_id)));
    }

    Err(anyhow!("Invalid version spec: {}", version_spec))
}

fn resolve_version_alias(repo: &LatticeRepo, object_id: &ObjectID, spec: &str) -> Result<Option<VersionID>> {
    if !spec.starts_with('v') {
        return Ok(None);
    }
    let index: usize = spec[1..].parse().map_err(|_| anyhow!("Invalid version index: {}", spec))?;
    if index == 0 {
        return Ok(None);
    }

    let object = repo.metadata.load_object(object_id)?;
    let mut versions = Vec::new();
    for vid in object.versions {
        let version = repo.metadata.load_version(&vid)?;
        versions.push(version);
    }

    versions.sort_by_key(|v| v.created_at);
    if index > versions.len() {
        return Ok(None);
    }
    Ok(Some(versions[index - 1].id))
}

pub fn parse_duration(spec: &str) -> Result<Duration> {
    if spec.is_empty() {
        return Err(anyhow!("Duration cannot be empty"));
    }
    let (num_str, unit) = spec.split_at(spec.len() - 1);
    let value: u64 = num_str.parse().map_err(|_| anyhow!("Invalid duration: {}", spec))?;
    let secs = match unit {
        "s" => value,
        "m" => value * 60,
        "h" => value * 3600,
        "d" => value * 86400,
        "w" => value * 7 * 86400,
        "y" => value * 365 * 86400,
        _ => return Err(anyhow!("Invalid duration unit: {}", unit)),
    };
    Ok(Duration::from_secs(secs))
}

pub fn expand_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(".") );
    }
    if let Some(rest) = path_str.strip_prefix("~/") {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")) .join(rest);
    }
    path.to_path_buf()
}

pub fn resolve_identity_password(explicit: Option<String>) -> Option<String> {
    explicit.or_else(|| std::env::var("LFS_KEY_PASSWORD").ok())
}
