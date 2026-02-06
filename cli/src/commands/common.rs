use anyhow::{Result, anyhow};
use latticefs_base::views::{BuiltinView, View};
use latticefs_base::{Config, KeyManager, LatticeRepo};
use latticefs_base::{Identity, ObjectID, VersionID};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const LOCAL_REPO_CONFIG_FILE: &str = ".latticefs.toml";
const LOCAL_REPO_DIR: &str = ".latticefs";

#[derive(Debug, Deserialize)]
struct LocalRepoConfig {
    repo: Option<LocalRepoSection>,
}

#[derive(Debug, Deserialize)]
struct LocalRepoSection {
    auto_load: Option<bool>,
}

pub fn open_repo(repo_path: Option<PathBuf>) -> Result<LatticeRepo> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("Failed to read current directory: {}", err))?;
    if let Some(path) = resolve_repo_override(repo_path, &cwd)? {
        return Ok(LatticeRepo::open_at(&path)?);
    }

    let config = Config::load_or_default()?;
    Ok(LatticeRepo::open(config)?)
}

fn resolve_repo_override(repo_path: Option<PathBuf>, cwd: &Path) -> Result<Option<PathBuf>> {
    if let Some(path) = repo_path {
        return Ok(Some(path));
    }

    if should_auto_load_repo(cwd)? {
        return Ok(Some(cwd.join(LOCAL_REPO_DIR)));
    }

    Ok(None)
}

fn should_auto_load_repo(cwd: &Path) -> Result<bool> {
    let marker_path = cwd.join(LOCAL_REPO_CONFIG_FILE);
    if !marker_path.exists() {
        return Ok(false);
    }

    let contents = fs::read_to_string(&marker_path)?;
    let config: LocalRepoConfig = toml::from_str(&contents)
        .map_err(|err| anyhow!("Failed to parse {}: {}", marker_path.display(), err))?;
    Ok(config
        .repo
        .and_then(|repo| repo.auto_load)
        .unwrap_or(false))
}

pub fn ensure_identity(name: &str, password: Option<&str>) -> Result<Identity> {
    let manager = KeyManager::auto();
    let env_password = std::env::var("LFS_KEY_PASSWORD").ok();
    let password = password.map(str::to_string).or(env_password);
    if manager.exists(name) {
        return Ok(manager.load(name, password.as_deref())?);
    }

    let identity = Identity::generate(name);
    manager.store(&identity, password.as_deref())?;
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

pub enum ResolvedView {
    Builtin(BuiltinView),
    Dynamic(View),
}

pub fn resolve_view_reference(repo: &LatticeRepo, reference: &str) -> Result<ResolvedView> {
    if let Some(builtin) = BuiltinView::by_name(reference) {
        return Ok(ResolvedView::Builtin(builtin));
    }

    if let Ok(uuid) = uuid::Uuid::parse_str(reference) {
        if let Some(view) = find_view_by_id(repo, &uuid)? {
            return Ok(ResolvedView::Dynamic(view));
        }
    }

    let view = repo.metadata.load_view(reference)?;
    Ok(ResolvedView::Dynamic(view))
}

pub fn resolve_dynamic_view(repo: &LatticeRepo, reference: &str) -> Result<View> {
    if let Ok(uuid) = uuid::Uuid::parse_str(reference) {
        if let Some(view) = find_view_by_id(repo, &uuid)? {
            return Ok(view);
        }
    }

    match repo.metadata.load_view(reference) {
        Ok(view) => Ok(view),
        Err(err) => {
            if BuiltinView::by_name(reference).is_some() {
                return Err(anyhow!("Built-in views cannot be modified"));
            }
            Err(anyhow!("{}", err.to_string()))
        }
    }
}

pub fn find_view_by_id(repo: &LatticeRepo, id: &uuid::Uuid) -> Result<Option<View>> {
    for view in repo.metadata.list_views()? {
        if view.id.as_uuid() == id {
            return Ok(Some(view));
        }
    }
    Ok(None)
}

pub fn parse_ref_with_version(
    repo: &LatticeRepo,
    reference: &str,
) -> Result<(ObjectID, Option<VersionID>)> {
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

fn resolve_version_alias(
    repo: &LatticeRepo,
    object_id: &ObjectID,
    spec: &str,
) -> Result<Option<VersionID>> {
    if !spec.starts_with('v') {
        return Ok(None);
    }
    let index: usize = spec[1..]
        .parse()
        .map_err(|_| anyhow!("Invalid version index: {}", spec))?;
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
    let value: u64 = num_str
        .parse()
        .map_err(|_| anyhow!("Invalid duration: {}", spec))?;
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
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    if let Some(rest) = path_str.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    path.to_path_buf()
}

pub fn resolve_identity_password(explicit: Option<String>) -> Option<String> {
    explicit.or_else(|| std::env::var("LFS_KEY_PASSWORD").ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn marker_missing_does_not_auto_load() {
        let temp = TempDir::new().unwrap();
        let auto = should_auto_load_repo(temp.path()).unwrap();
        assert!(!auto);
    }

    #[test]
    fn marker_requires_auto_load_true() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join(LOCAL_REPO_CONFIG_FILE),
            "[repo]\nauto_load = false\n",
        )
        .unwrap();

        let auto = should_auto_load_repo(temp.path()).unwrap();
        assert!(!auto);
    }

    #[test]
    fn marker_auto_load_true_enables_current_dir_repo() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join(LOCAL_REPO_CONFIG_FILE),
            "[repo]\nauto_load = true\n",
        )
        .unwrap();

        let auto = should_auto_load_repo(temp.path()).unwrap();
        assert!(auto);
    }

    #[test]
    fn marker_parse_errors_are_reported() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join(LOCAL_REPO_CONFIG_FILE),
            "[repo]\nauto_load = tru\n",
        )
        .unwrap();

        let err = should_auto_load_repo(temp.path()).unwrap_err();
        assert!(err
            .to_string()
            .contains(&format!("Failed to parse {}", temp.path().join(LOCAL_REPO_CONFIG_FILE).display())));
    }

    #[test]
    fn explicit_repo_override_has_highest_precedence() {
        let cwd = TempDir::new().unwrap();
        fs::write(
            cwd.path().join(LOCAL_REPO_CONFIG_FILE),
            "[repo]\nauto_load = true\n",
        )
        .unwrap();
        let explicit = PathBuf::from("/tmp/lattice-explicit");

        let resolved = resolve_repo_override(Some(explicit.clone()), cwd.path()).unwrap();
        assert_eq!(resolved, Some(explicit));
    }

    #[test]
    fn marker_auto_load_sets_repo_to_local_repo_dir() {
        let cwd = TempDir::new().unwrap();
        fs::write(
            cwd.path().join(LOCAL_REPO_CONFIG_FILE),
            "[repo]\nauto_load = true\n",
        )
        .unwrap();

        let resolved = resolve_repo_override(None, cwd.path()).unwrap();
        assert_eq!(resolved, Some(cwd.path().join(LOCAL_REPO_DIR)));
    }

    #[test]
    fn without_explicit_or_marker_falls_back_to_global_config() {
        let cwd = TempDir::new().unwrap();

        let resolved = resolve_repo_override(None, cwd.path()).unwrap();
        assert_eq!(resolved, None);
    }
}
