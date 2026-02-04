//! Filesystem scanner for import operations.

use crate::error::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

/// Scan a path for files. If path is a file, returns just that file.
pub fn scan_path(path: &Path) -> Result<Vec<FileEntry>> {
    let mut files = Vec::new();

    if path.is_file() {
        let size = path.metadata()?.len();
        files.push(FileEntry {
            path: path.to_path_buf(),
            size,
        });
        return Ok(files);
    }

    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry.map_err(|e| crate::error::LatticeError::Io(std::io::Error::other(e)))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let size = entry
            .metadata()
            .map_err(|e| crate::error::LatticeError::Io(std::io::Error::other(e)))?
            .len();
        files.push(FileEntry {
            path: entry.path().to_path_buf(),
            size,
        });
    }

    Ok(files)
}
