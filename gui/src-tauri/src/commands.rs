use latticefs_base::config::Config;
use latticefs_base::crypto::{Identity, KeyManager};
use latticefs_base::error::LatticeError;
use latticefs_base::import::{import_file, scanner, ImportOptions};
use latticefs_base::LatticeRepo;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Emitter;

#[derive(Debug, Serialize)]
pub struct RepoInfo {
    pub root: String,
    pub config_path: String,
}

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub imported: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportTarget {
    pub path: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SampleFilesResult {
    pub root: String,
    pub files: Vec<String>,
}

fn repo_info() -> Result<RepoInfo, String> {
    let config = Config::load_or_default().map_err(|err| err.to_string())?;
    let root = config.storage_path();
    let config_path = latticefs_base::config::config_path();
    Ok(RepoInfo {
        root: root.to_string_lossy().to_string(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

fn load_or_create_identity() -> Result<Identity, String> {
    let manager = KeyManager::auto();
    let password = std::env::var("LFS_KEY_PASSWORD").ok();
    match manager.load("default", password.as_deref()) {
        Ok(identity) => Ok(identity),
        Err(LatticeError::IdentityNotFound { .. }) => {
            let identity = Identity::generate("default");
            manager
                .store(&identity, password.as_deref())
                .map_err(|err| err.to_string())?;
            Ok(identity)
        }
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub fn get_repo_info() -> Result<RepoInfo, String> {
    repo_info()
}

#[tauri::command]
pub fn init_repo() -> Result<RepoInfo, String> {
    LatticeRepo::init().map_err(|err| err.to_string())?;
    repo_info()
}

#[tauri::command]
pub async fn import_paths(
    app: tauri::AppHandle,
    targets: Vec<ImportTarget>,
) -> Result<ImportSummary, String> {
    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let identity = load_or_create_identity()?;
    let actor = identity.public_bytes();

    let mut all_entries: Vec<(PathBuf, Vec<String>, Option<PathBuf>)> = Vec::new();
    for target in &targets {
        let path = PathBuf::from(&target.path);
        if !path.exists() {
            return Err(format!("Path does not exist: {}", target.path));
        }
        let entries = scanner::scan_path(&path).map_err(|err| err.to_string())?;
        let base_path = if path.is_dir() {
            Some(path.clone())
        } else {
            None
        };
        for entry in entries {
            all_entries.push((entry.path, target.tags.clone(), base_path.clone()));
        }
    }

    let total = all_entries.len();
    let mut imported = 0usize;
    let mut failed = 0usize;
    let mut errors = Vec::new();

    for (index, (path, tags, base_path)) in all_entries.into_iter().enumerate() {
        let options = ImportOptions {
            tags,
            extract_exif: repo.config.import.extract_exif,
            extract_id3: repo.config.import.extract_id3,
            extract_text: repo.config.import.extract_text,
            actor,
            base_path,
        };

        match import_file(&repo, &path, &options).await {
            Ok(_) => imported += 1,
            Err(err) => {
                failed += 1;
                errors.push(format!("{}: {}", path.display(), err));
            }
        }

        let progress = ImportProgress {
            current: index + 1,
            total,
            path: path.to_string_lossy().to_string(),
        };
        let _ = app.emit("import_progress", &progress);
    }

    Ok(ImportSummary {
        imported,
        failed,
        errors,
    })
}

fn write_sample_file(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    std::fs::write(path, contents).map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_sample_files() -> Result<SampleFilesResult, String> {
    let root = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LatticeFS Samples");

    let sample_files = vec![
        (
            root.join("Documents").join("Willkommen_bei_LatticeFS.md"),
            "# Willkommen bei LatticeFS\n\nDies ist eine Beispielnotiz für den Import.\n",
        ),
        (
            root.join("Downloads").join("setup_installer.exe"),
            "binary-placeholder",
        ),
        (
            root.join("Projects")
                .join("Phoenix")
                .join("Projektplan.txt"),
            "Projekt Phoenix\n- Kickoff\n- Planung\n- Umsetzung\n",
        ),
        (
            root.join("Pictures").join("urlaub_mallorca_2023.txt"),
            "Foto-Metadaten: Mallorca 2023\n",
        ),
    ];

    let mut created = Vec::new();
    for (path, contents) in sample_files {
        write_sample_file(&path, contents)?;
        created.push(path.to_string_lossy().to_string());
    }

    Ok(SampleFilesResult {
        root: root.to_string_lossy().to_string(),
        files: created,
    })
}
