use crate::model::{ActorID, ObjectID};
use crate::storage::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEntry {
    pub temp_path: PathBuf,
    pub object_id: ObjectID,
    pub actor_id: ActorID,
    pub original_hash: Hash,
    pub last_known_hash: Hash,
    pub display_name: String,
    pub registered_at: i64,
}

#[derive(Debug, Clone)]
pub struct WatchRegistry {
    entries: Arc<RwLock<HashMap<PathBuf, WatchEntry>>>,
}

impl WatchRegistry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn from_entries(entries: HashMap<PathBuf, WatchEntry>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(entries)),
        }
    }

    pub fn register(&self, entry: WatchEntry) {
        let path = entry.temp_path.clone();
        self.entries.write().unwrap().insert(path, entry);
    }

    pub fn unregister(&self, path: &Path) -> Option<WatchEntry> {
        self.entries.write().unwrap().remove(path)
    }

    pub fn unregister_by_object(&self, object_id: &ObjectID) -> Vec<WatchEntry> {
        let mut entries = self.entries.write().unwrap();
        let paths_to_remove: Vec<PathBuf> = entries
            .iter()
            .filter(|(_, e)| &e.object_id == object_id)
            .map(|(p, _)| p.clone())
            .collect();

        paths_to_remove
            .into_iter()
            .filter_map(|p| entries.remove(&p))
            .collect()
    }

    pub fn get(&self, path: &Path) -> Option<WatchEntry> {
        self.entries.read().unwrap().get(path).cloned()
    }

    pub fn update_hash(&self, path: &Path, new_hash: Hash) -> bool {
        let mut entries = self.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(path) {
            entry.last_known_hash = new_hash;
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<WatchEntry> {
        self.entries.read().unwrap().values().cloned().collect()
    }

    pub fn snapshot(&self) -> HashMap<PathBuf, WatchEntry> {
        self.entries.read().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.entries.read().unwrap().len()
    }
}

impl Default for WatchRegistry {
    fn default() -> Self {
        Self::new()
    }
}
