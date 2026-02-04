//! Chunking helpers for import.

use crate::error::Result;
use crate::storage::ChunkManifest;
use crate::storage::ChunkStore;
use std::path::Path;

/// Chunk a file and return its manifest.
pub async fn chunk_file(store: &ChunkStore, path: &Path) -> Result<ChunkManifest> {
    let data = tokio::fs::read(path).await?;
    store.store_object(&data).await
}
