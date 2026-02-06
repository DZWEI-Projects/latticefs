use crate::config::Config;
use crate::error::{LatticeError, Result};
use crate::events::Event;
use crate::repo::LatticeRepo;
use crate::storage::compute_hash;
use crate::watcher::persist::PersistentRegistry;
use crate::watcher::registry::WatchRegistry;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, warn};

pub struct FileWatcher {
    registry: WatchRegistry,
    persist: PersistentRegistry,
    config: Config,
    shutdown_rx: watch::Receiver<bool>,
}

impl FileWatcher {
    pub fn new(
        registry: WatchRegistry,
        persist: PersistentRegistry,
        config: Config,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            registry,
            persist,
            config,
            shutdown_rx,
        }
    }

    pub fn registry(&self) -> &WatchRegistry {
        &self.registry
    }

    pub async fn run(&mut self) -> Result<()> {
        let watch_dir = PathBuf::from(&self.config.watcher.watch_dir);
        if !watch_dir.exists() {
            std::fs::create_dir_all(&watch_dir)?;
        }

        let (tx, mut rx) = mpsc::channel::<Vec<PathBuf>>(256);

        let debounce_duration = Duration::from_millis(self.config.watcher.debounce_ms);
        let mut debouncer = new_debouncer(
            debounce_duration,
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    let paths: Vec<PathBuf> =
                        events.into_iter().flat_map(|e| e.event.paths).collect();
                    if !paths.is_empty() {
                        let _ = tx.blocking_send(paths);
                    }
                }
                Err(errors) => {
                    for err in errors {
                        error!("File watcher error: {:?}", err);
                    }
                }
            },
        )
        .map_err(|e| LatticeError::WatcherError(format!("Failed to create debouncer: {}", e)))?;

        debouncer
            .watch(&watch_dir, notify::RecursiveMode::NonRecursive)
            .map_err(|e| LatticeError::WatcherError(format!("Failed to watch directory: {}", e)))?;

        info!("File watcher started, watching: {}", watch_dir.display());
        info!("Watching {} registered files", self.registry.count());

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        info!("File watcher shutting down");
                        break;
                    }
                }
                Some(paths) = rx.recv() => {
                    for path in paths {
                        if let Err(e) = self.handle_file_change(&path).await {
                            error!("Error handling change for {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        // Save registry on shutdown
        if let Err(e) = self.persist.save(&self.registry) {
            error!("Failed to save registry on shutdown: {}", e);
        }

        Ok(())
    }

    async fn handle_file_change(&self, path: &Path) -> Result<()> {
        // Skip ignored patterns
        if self.is_ignored(path) {
            debug!("Ignoring file change: {}", path.display());
            return Ok(());
        }

        // Canonicalize path to handle symlinks (e.g., /tmp -> /private/tmp on macOS).
        // The OS reports file changes using the canonical path, so we must canonicalize
        // here to match the canonicalized path stored during registration.
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                debug!("Failed to canonicalize path {}: {}", path.display(), e);
                return Ok(());
            }
        };

        // Look up in registry using canonical path
        let entry = match self.registry.get(&canonical_path) {
            Some(entry) => entry,
            None => {
                debug!(
                    "File not in registry, skipping: {}",
                    canonical_path.display()
                );
                return Ok(());
            }
        };

        // Read file content — if deleted, unregister
        let data = match std::fs::read(&canonical_path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!(
                    "Watched file deleted, unregistering: {}",
                    canonical_path.display()
                );
                self.registry.unregister(&canonical_path);
                self.persist.save(&self.registry)?;
                // Open repo briefly to emit event
                if let Ok(repo) = LatticeRepo::open(self.config.clone()) {
                    repo.events.emit_sync(Event::watch_file_removed(
                        &entry.object_id,
                        canonical_path.display().to_string(),
                        "file_deleted".to_string(),
                    ));
                }
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        // Compute hash — skip if identical to last known
        let new_hash = compute_hash(&data);
        if new_hash == entry.last_known_hash {
            debug!(
                "No content change (hash match), skipping: {}",
                canonical_path.display()
            );
            return Ok(());
        }

        // Format commit message
        let message = self.format_commit_message(&canonical_path, &entry.object_id);

        // Open repo on demand for the database operation, then drop it to
        // release the Sled file lock so CLI commands can access the repo.
        let repo = LatticeRepo::open(self.config.clone())?;

        // Create new version
        match repo
            .add_version_from_bytes(&entry.object_id, &data, entry.actor_id, Some(message))
            .await
        {
            Ok(version) => {
                info!(
                    "Auto-created version {} for object {} from {}",
                    version.id,
                    entry.object_id,
                    canonical_path.display()
                );
                self.registry.update_hash(&canonical_path, new_hash);
                self.persist.save(&self.registry)?;
                repo.events.emit_sync(Event::auto_version_created(
                    &entry.object_id,
                    &version.id,
                    canonical_path.display().to_string(),
                    entry.actor_id,
                ));
            }
            Err(LatticeError::ObjectSealed { id }) => {
                warn!(
                    "Object {} is sealed, unregistering watcher for {}",
                    id,
                    canonical_path.display()
                );
                self.registry.unregister(&canonical_path);
                self.persist.save(&self.registry)?;
                repo.events.emit_sync(Event::watch_file_removed(
                    &entry.object_id,
                    canonical_path.display().to_string(),
                    "object_sealed".to_string(),
                ));
            }
            Err(e) => {
                error!(
                    "Failed to create version for {}: {}",
                    canonical_path.display(),
                    e
                );
            }
        }

        Ok(())
    }

    fn is_ignored(&self, path: &Path) -> bool {
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        for pattern in &self.config.watcher.ignored_patterns {
            if let Ok(pat) = glob::Pattern::new(pattern) {
                if pat.matches(filename) {
                    return true;
                }
            }
        }

        false
    }

    fn format_commit_message(&self, path: &Path, object_id: &crate::model::ObjectID) -> String {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let timestamp = chrono_like_timestamp();

        self.config
            .watcher
            .commit_message_template
            .replace("{timestamp}", &timestamp)
            .replace("{filename}", filename)
            .replace("{object_id}", &object_id.to_string())
    }
}

fn chrono_like_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Simple UTC timestamp: YYYY-MM-DD HH:MM:SS
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Approximate date calculation (good enough for commit messages)
    let (year, month, day) = days_to_date(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
