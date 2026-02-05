use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use latticefs_base::config::Config;
use latticefs_base::crypto::{Identity, KeyManager};
use latticefs_base::error::LatticeError;
use latticefs_base::import::{export_object, import_file, scanner, ExportMode, ImportOptions};
use latticefs_base::model::{ObjectID, Tag};
use latticefs_base::query::{parse, QueryEvaluator};
use latticefs_base::views::{BuiltinView, BuiltinViews, Locale};
use latticefs_base::LatticeRepo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tauri::Emitter;
use uuid::Uuid;

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

#[derive(Debug, Serialize)]
pub struct OnboardingFile {
    pub id: String,
    pub name: String,
    pub extension: Option<String>,
    pub views: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OnboardingGraphData {
    pub files: Vec<OnboardingFile>,
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

fn decode_tag_value(tags: &[Tag], key: &str) -> Option<String> {
    let tag = tags.iter().find(|t| t.key == key)?;
    let decoded = URL_SAFE_NO_PAD.decode(tag.value.as_bytes()).ok()?;
    String::from_utf8(decoded).ok()
}

fn display_name(tags: &[Tag], object_id: &ObjectID) -> String {
    if let Some(name) = decode_tag_value(tags, "auto:filename_b64") {
        return name;
    }
    if let Some(relpath) = decode_tag_value(tags, "auto:relpath_b64") {
        if let Some(base) = Path::new(&relpath).file_name().and_then(|s| s.to_str()) {
            return base.to_string();
        }
        if !relpath.is_empty() {
            return relpath;
        }
    }
    object_id.to_string()
}

fn file_extension(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
}

fn parse_object_id(object_id: &str) -> Result<ObjectID, String> {
    let uuid = Uuid::parse_str(object_id).map_err(|err| err.to_string())?;
    Ok(ObjectID::from_uuid(uuid))
}

fn open_with_default_app(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .map_err(|err| err.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|err| err.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|err| err.to_string())?;
        return Ok(());
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Opening files is not supported on this platform".to_string())
    }
}

fn eval_query(repo: &LatticeRepo, query: &str) -> Result<Vec<ObjectID>, String> {
    let parsed = parse(query).map_err(|err| err.to_string())?;
    let evaluator = QueryEvaluator::new(&repo.metadata);
    evaluator.execute(&parsed).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_onboarding_graph() -> Result<OnboardingGraphData, String> {
    const MAX_PER_VIEW: usize = 80;
    const MAX_FILES: usize = 120;

    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let builtin = BuiltinViews::new(&repo.metadata);

    let mut recent = builtin
        .evaluate(BuiltinView::Recent)
        .map_err(|err| err.to_string())?;
    recent.truncate(MAX_PER_VIEW);

    let mut projects = builtin
        .evaluate(BuiltinView::Projects)
        .map_err(|err| err.to_string())?;
    if projects.is_empty() {
        projects = eval_query(&repo, "tag:source:projects SORT updated DESC LIMIT 120")?;
    }
    projects.truncate(MAX_PER_VIEW);

    let mut by_type = eval_query(&repo, "type:* SORT updated DESC LIMIT 120")?;
    by_type.truncate(MAX_PER_VIEW);

    let mut downloads = eval_query(&repo, "tag:source:downloads SORT updated DESC LIMIT 120")?;
    downloads.truncate(MAX_PER_VIEW);

    let mut quarantine = eval_query(
        &repo,
        "tag:auto:executable AND trust < 90 SORT updated DESC LIMIT 120",
    )?;
    quarantine.truncate(MAX_PER_VIEW);

    let view_sets = [
        ("neueste".to_string(), recent),
        ("projekte".to_string(), projects),
        ("nach-typ".to_string(), by_type),
        ("downloads".to_string(), downloads),
        ("quarant\u{00e4}ne".to_string(), quarantine),
    ];

    let mut files: HashMap<ObjectID, OnboardingFile> = HashMap::new();

    for (view_id, ids) in view_sets {
        for object_id in ids {
            if !files.contains_key(&object_id) {
                let object = match repo.metadata.load_object(&object_id) {
                    Ok(obj) => obj,
                    Err(_) => continue,
                };
                let name = display_name(&object.tags, &object_id);
                let extension = file_extension(&name);
                files.insert(
                    object_id,
                    OnboardingFile {
                        id: object_id.to_string(),
                        name,
                        extension,
                        views: Vec::new(),
                    },
                );
            }
            if let Some(entry) = files.get_mut(&object_id) {
                if !entry.views.contains(&view_id) {
                    entry.views.push(view_id.clone());
                }
            }
        }
    }

    let mut files: Vec<OnboardingFile> = files.into_values().collect();
    files.truncate(MAX_FILES);

    Ok(OnboardingGraphData { files })
}

// ============================================================================
// Nexus / Hub View Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub query: String,
    pub view_type: String, // "builtin" | "dynamic"
    pub icon: Option<String>,
    pub object_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectInfo {
    pub id: String,
    pub name: String,
    pub extension: Option<String>,
    pub object_type: String,
    pub size_bytes: u64,
    pub created_at: i64,
    pub modified_at: i64,
    pub tags: Vec<TagInfo>,
    pub views: Vec<String>,
    pub trust_level: Option<u8>,
}

fn builtin_view_icon(view: BuiltinView) -> Option<String> {
    match view {
        BuiltinView::Recent => Some("Clock".to_string()),
        BuiltinView::Projects => Some("Folder".to_string()),
        BuiltinView::Drafts => Some("FileEdit".to_string()),
        BuiltinView::Review => Some("Eye".to_string()),
        BuiltinView::Approved => Some("CheckCircle".to_string()),
        BuiltinView::All => Some("Grid".to_string()),
    }
}

fn object_to_info(
    repo: &LatticeRepo,
    object_id: &ObjectID,
    view_memberships: Option<&Vec<String>>,
) -> Option<ObjectInfo> {
    let object = repo.metadata.load_object(object_id).ok()?;
    let name = display_name(&object.tags, object_id);
    let extension = file_extension(&name);

    // Load current version for size info
    let version = repo.metadata.load_version(&object.current_version).ok();
    let size_bytes = version.as_ref().map(|v| v.size_bytes).unwrap_or(0);
    let modified_at = version.as_ref().map(|v| v.created_at).unwrap_or(object.created_at);

    // Get trust level from tags
    let trust_level = object
        .tags
        .iter()
        .find(|t| t.key == "trust")
        .and_then(|t| t.value.parse::<u8>().ok());

    // Convert tags
    let tags: Vec<TagInfo> = object
        .tags
        .iter()
        .filter(|t| !t.key.starts_with("auto:") && !t.key.starts_with("system:"))
        .map(|t| TagInfo {
            key: t.key.clone(),
            value: t.value.clone(),
        })
        .collect();

    let object_type = match object.object_type {
        latticefs_base::model::ObjectType::Blob => "blob",
        latticefs_base::model::ObjectType::Tree => "tree",
        latticefs_base::model::ObjectType::Commit => "commit",
    };

    Some(ObjectInfo {
        id: object_id.to_string(),
        name,
        extension,
        object_type: object_type.to_string(),
        size_bytes,
        created_at: object.created_at,
        modified_at,
        tags,
        views: view_memberships.cloned().unwrap_or_default(),
        trust_level,
    })
}

#[tauri::command]
pub fn list_views() -> Result<Vec<ViewInfo>, String> {
    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let builtin = BuiltinViews::new(&repo.metadata);
    let locale = Locale::from_system();

    let mut views = Vec::new();

    // Add built-in views with localized names and descriptions
    for bv in BuiltinView::all() {
        let count = builtin.count(*bv).unwrap_or(0);
        views.push(ViewInfo {
            id: bv.name().to_lowercase().replace(' ', "-"),
            name: bv.name_localized(locale).to_string(),
            description: bv.description_localized(locale).to_string(),
            query: bv.query().to_string(),
            view_type: "builtin".to_string(),
            icon: builtin_view_icon(*bv),
            object_count: count,
        });
    }

    // Add dynamic views from metadata store
    if let Ok(dynamic_views) = repo.metadata.list_views() {
        for view in dynamic_views {
            // Count objects for this view
            let count = eval_query(&repo, &view.query)
                .map(|ids| ids.len())
                .unwrap_or(0);
            views.push(ViewInfo {
                id: view.id.to_string(),
                name: view.name.clone(),
                description: view.description.clone().unwrap_or_default(),
                query: view.query.clone(),
                view_type: "dynamic".to_string(),
                icon: None,
                object_count: count,
            });
        }
    }

    Ok(views)
}

#[tauri::command]
pub fn get_view_objects(view_id: String) -> Result<Vec<ObjectInfo>, String> {
    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;

    // Try to find the view (builtin or dynamic)
    let query = if let Some(bv) = BuiltinView::by_name(&view_id)
        .or_else(|| BuiltinView::by_name(&view_id.replace('-', " ")))
    {
        bv.query().to_string()
    } else if let Ok(view) = repo.metadata.load_view(&view_id) {
        view.query.clone()
    } else if let Ok(views) = repo.metadata.list_views() {
        views
            .into_iter()
            .find(|view| view.id.to_string() == view_id)
            .map(|view| view.query)
            .ok_or_else(|| format!("View not found: {}", view_id))?
    } else {
        return Err(format!("View not found: {}", view_id));
    };

    let object_ids = eval_query(&repo, &query)?;

    // Build view memberships for each object
    let builtin = BuiltinViews::new(&repo.metadata);
    let mut view_memberships: HashMap<ObjectID, Vec<String>> = HashMap::new();

    // Check which builtin views each object belongs to
    for bv in BuiltinView::all() {
        if let Ok(bv_ids) = builtin.evaluate(*bv) {
            let bv_set: HashSet<ObjectID> = bv_ids.into_iter().collect();
            for oid in &object_ids {
                if bv_set.contains(oid) {
                    view_memberships
                        .entry(*oid)
                        .or_default()
                        .push(bv.name().to_lowercase().replace(' ', "-"));
                }
            }
        }
    }

    let objects: Vec<ObjectInfo> = object_ids
        .iter()
        .filter_map(|oid| object_to_info(&repo, oid, view_memberships.get(oid)))
        .collect();

    Ok(objects)
}

#[tauri::command]
pub fn evaluate_query(query: String) -> Result<Vec<ObjectInfo>, String> {
    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let object_ids = eval_query(&repo, &query)?;

    let objects: Vec<ObjectInfo> = object_ids
        .iter()
        .filter_map(|oid| object_to_info(&repo, oid, None))
        .collect();

    Ok(objects)
}

#[tauri::command]
pub fn add_object_tag(object_id: String, tag: TagInfo) -> Result<ObjectInfo, String> {
    if tag.key.trim().is_empty() || tag.value.trim().is_empty() {
        return Err("Tag key and value cannot be empty".to_string());
    }

    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let identity = load_or_create_identity()?;
    let actor = identity.public_bytes();
    let object_id = parse_object_id(&object_id)?;
    let mut object = repo
        .metadata
        .load_object(&object_id)
        .map_err(|err| err.to_string())?;

    let tag = Tag::new(tag.key, tag.value, actor);
    let tag_path = tag.full_path();
    object.add_tag(tag);
    repo.metadata
        .store_object(&object)
        .map_err(|err| err.to_string())?;
    repo.metadata
        .add_to_tag_index(&tag_path, object_id.as_bytes())
        .map_err(|err| err.to_string())?;

    object_to_info(&repo, &object_id, None).ok_or_else(|| "Object not found".to_string())
}

#[tauri::command]
pub fn remove_object_tag(object_id: String, tag: TagInfo) -> Result<ObjectInfo, String> {
    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let object_id = parse_object_id(&object_id)?;
    let mut object = repo
        .metadata
        .load_object(&object_id)
        .map_err(|err| err.to_string())?;

    let removed_tags: Vec<String> = object
        .tags
        .iter()
        .filter(|existing| existing.key == tag.key && existing.value == tag.value)
        .map(|existing| existing.full_path())
        .collect();

    if !removed_tags.is_empty() {
        object
            .tags
            .retain(|existing| !(existing.key == tag.key && existing.value == tag.value));
        repo.metadata
            .store_object(&object)
            .map_err(|err| err.to_string())?;
        for tag_path in removed_tags {
            repo.metadata
                .remove_from_tag_index(&tag_path, object_id.as_bytes())
                .map_err(|err| err.to_string())?;
        }
    }

    object_to_info(&repo, &object_id, None).ok_or_else(|| "Object not found".to_string())
}

#[tauri::command]
pub fn set_object_trust_level(
    object_id: String,
    trust_level: Option<u8>,
) -> Result<ObjectInfo, String> {
    if let Some(level) = trust_level {
        if level > 100 {
            return Err("Trust level must be between 0 and 100".to_string());
        }
    }

    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let identity = load_or_create_identity()?;
    let actor = identity.public_bytes();
    let object_id = parse_object_id(&object_id)?;
    let mut object = repo
        .metadata
        .load_object(&object_id)
        .map_err(|err| err.to_string())?;

    let removed_trust_tags: Vec<String> = object
        .tags
        .iter()
        .filter(|existing| existing.key == "trust")
        .map(|existing| existing.full_path())
        .collect();

    if !removed_trust_tags.is_empty() {
        object.tags.retain(|existing| existing.key != "trust");
        for tag_path in &removed_trust_tags {
            repo.metadata
                .remove_from_tag_index(tag_path, object_id.as_bytes())
                .map_err(|err| err.to_string())?;
        }
    }

    if let Some(level) = trust_level {
        let trust_tag = Tag::new("trust".to_string(), level.to_string(), actor);
        let tag_path = trust_tag.full_path();
        object.add_tag(trust_tag);
        repo.metadata
            .add_to_tag_index(&tag_path, object_id.as_bytes())
            .map_err(|err| err.to_string())?;
    }

    repo.metadata
        .store_object(&object)
        .map_err(|err| err.to_string())?;

    object_to_info(&repo, &object_id, None).ok_or_else(|| "Object not found".to_string())
}

#[tauri::command]
pub async fn open_object(object_id: String) -> Result<(), String> {
    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let object_id = parse_object_id(&object_id)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .map_err(|err| err.to_string())?;
    let name = display_name(&object.tags, &object_id);
    let safe_name = name.replace(['/', '\\'], "_");

    let output_dir = std::env::temp_dir().join("latticefs-open");
    std::fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
    let output_path = output_dir.join(format!("{}_{}", object_id, safe_name));

    export_object(&repo, &object_id, None, &output_path, ExportMode::Tree)
        .await
        .map_err(|err| err.to_string())?;
    open_with_default_app(&output_path)
}

#[derive(Debug, Deserialize)]
pub struct CreateViewArgs {
    pub name: String,
    pub query: String,
    pub description: Option<String>,
}

fn validate_view_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("View name cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\0') {
        return Err("View name contains invalid characters".to_string());
    }
    // Check if it conflicts with a builtin view
    if BuiltinView::by_name(name).is_some() {
        return Err(format!("Cannot use reserved name: {}", name));
    }
    Ok(())
}

#[tauri::command]
pub fn create_view(args: CreateViewArgs) -> Result<ViewInfo, String> {
    validate_view_name(&args.name)?;

    // Validate the query syntax
    parse(&args.query).map_err(|err| format!("Invalid query: {}", err))?;

    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    let identity = load_or_create_identity()?;
    let actor = identity.public_bytes();

    let mut view = latticefs_base::views::View::new(args.name.clone(), args.query.clone(), actor);
    if let Some(desc) = args.description {
        view = view.with_description(desc);
    }

    repo.metadata
        .store_view(&view)
        .map_err(|err| err.to_string())?;

    // Return the created view info
    let count = eval_query(&repo, &args.query)
        .map(|ids| ids.len())
        .unwrap_or(0);

    Ok(ViewInfo {
        id: view.id.to_string(),
        name: view.name,
        description: view.description.unwrap_or_default(),
        query: view.query,
        view_type: "dynamic".to_string(),
        icon: None,
        object_count: count,
    })
}

#[tauri::command]
pub fn delete_view(name: String) -> Result<(), String> {
    // Cannot delete builtin views
    if BuiltinView::by_name(&name).is_some() {
        return Err("Cannot delete built-in views".to_string());
    }

    let repo = LatticeRepo::init().map_err(|err| err.to_string())?;
    repo.metadata
        .delete_view(&name)
        .map_err(|err| err.to_string())?;

    Ok(())
}
