use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use latticefs_base::{
    LatticeRepo,
    config::default_home,
    import::{ImportOptions, import_file, scanner},
    model::Tag,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use cli::commands::common::{ensure_identity, identity_actor, open_repo};

#[derive(Clone, Default)]
struct AppState;

#[derive(Debug, Serialize)]
struct ApiError {
    message: String,
}

impl ApiError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<latticefs_base::error::LatticeError> for ApiError {
    fn from(err: latticefs_base::error::LatticeError) -> Self {
        Self::new(err.to_string())
    }
}

type ApiResult<T> = Result<T, (StatusCode, Json<ApiError>)>;

#[derive(Debug, Serialize)]
struct StatusResponse {
    repo_path: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct FolderOption {
    id: String,
    name: String,
    path: String,
    exists: bool,
    default_selected: bool,
    is_demo: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OnboardingSettings {
    quarantine_downloads: bool,
    versioning: bool,
    execute_warning: bool,
}

impl Default for OnboardingSettings {
    fn default() -> Self {
        Self {
            quarantine_downloads: true,
            versioning: true,
            execute_warning: true,
        }
    }
}

#[derive(Debug, Serialize)]
struct SettingsResponse {
    settings: OnboardingSettings,
}

#[derive(Debug, Serialize)]
struct ImportSummary {
    folder_id: String,
    files: usize,
    objects: usize,
    bytes: u64,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ImportResponse {
    summaries: Vec<ImportSummary>,
    total_files: usize,
    total_objects: usize,
    total_bytes: u64,
    errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ImportRequest {
    folder_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateSettingsRequest {
    quarantine_downloads: bool,
    versioning: bool,
    execute_warning: bool,
}

#[derive(Debug, Serialize)]
struct SeedResponse {
    demo_root: String,
    folders: Vec<FolderOption>,
}

#[derive(Debug, Serialize)]
struct OnboardingView {
    id: String,
    name: String,
    icon: String,
    description: String,
    color: String,
}

#[derive(Debug, Serialize)]
struct OnboardingFile {
    id: String,
    name: String,
    extension: Option<String>,
    size_bytes: u64,
    created_at: String,
    tags: Vec<String>,
    views: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OnboardingProject {
    id: String,
    name: String,
    color: String,
}

#[derive(Debug, Serialize)]
struct OnboardingDataResponse {
    views: Vec<OnboardingView>,
    files: Vec<OnboardingFile>,
    projects: Vec<OnboardingProject>,
    highlighted_file_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AssignProjectRequest {
    object_id: String,
    project_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/status", get(status))
        .route("/api/onboarding/init", post(init_repo))
        .route("/api/onboarding/folders", get(list_folders))
        .route("/api/onboarding/seed-files", post(seed_demo_files))
        .route("/api/onboarding/import", post(import_folders))
        .route(
            "/api/onboarding/settings",
            get(get_settings).post(update_settings),
        )
        .route("/api/onboarding/data", get(get_onboarding_data))
        .route("/api/onboarding/assign-project", post(assign_project))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        )
        .with_state(AppState::default());

    let port = std::env::var("LFS_GUI_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8788);
    let addr = format!("0.0.0.0:{port}");
    info!("LatticeFS GUI API listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn status(State(_state): State<AppState>) -> ApiResult<Json<StatusResponse>> {
    let repo = open_repo(None).map_err(api_error)?;
    Ok(Json(StatusResponse {
        repo_path: repo.root.to_string_lossy().to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

async fn init_repo(State(_state): State<AppState>) -> ApiResult<Json<StatusResponse>> {
    let repo = tokio::task::spawn_blocking(|| LatticeRepo::init())
        .await
        .map_err(|err| api_error(anyhow::Error::new(err)))?
        .map_err(|err| api_error(anyhow::Error::new(err)))?;
    Ok(Json(StatusResponse {
        repo_path: repo.root.to_string_lossy().to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

async fn list_folders(State(_state): State<AppState>) -> ApiResult<Json<Vec<FolderOption>>> {
    Ok(Json(build_folder_options()))
}

async fn seed_demo_files(State(_state): State<AppState>) -> ApiResult<Json<SeedResponse>> {
    let demo_root = demo_root();
    create_demo_files(&demo_root).map_err(api_error)?;
    Ok(Json(SeedResponse {
        demo_root: demo_root.to_string_lossy().to_string(),
        folders: build_folder_options(),
    }))
}

async fn get_settings(State(_state): State<AppState>) -> ApiResult<Json<SettingsResponse>> {
    Ok(Json(SettingsResponse {
        settings: load_settings().map_err(api_error)?,
    }))
}

async fn update_settings(
    State(_state): State<AppState>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> ApiResult<Json<SettingsResponse>> {
    let settings = OnboardingSettings {
        quarantine_downloads: payload.quarantine_downloads,
        versioning: payload.versioning,
        execute_warning: payload.execute_warning,
    };
    save_settings(&settings).map_err(api_error)?;
    Ok(Json(SettingsResponse { settings }))
}

async fn import_folders(
    State(_state): State<AppState>,
    Json(payload): Json<ImportRequest>,
) -> ApiResult<Json<ImportResponse>> {
    let repo = open_repo(None).map_err(api_error)?;
    let identity = ensure_identity("default", None).map_err(api_error)?;
    let actor = identity_actor(&identity);
    let settings = load_settings().map_err(api_error)?;
    let folder_map = folder_definitions();

    let mut summaries = Vec::new();
    let mut total_files = 0usize;
    let mut total_objects = 0usize;
    let mut total_bytes = 0u64;
    let mut errors = Vec::new();

    for folder_id in payload.folder_ids {
        let folder = folder_map
            .iter()
            .find(|entry| entry.id == folder_id)
            .ok_or_else(|| api_error(anyhow::anyhow!("Unknown folder: {}", folder_id)))?;

        let path = folder.path.clone();
        if !path.exists() {
            errors.push(format!("{} does not exist", path.display()));
            continue;
        }

        let summary = import_folder(&repo, folder, actor, &settings)
            .await
            .map_err(api_error)?;
        total_files += summary.files;
        total_objects += summary.objects;
        total_bytes += summary.bytes;
        errors.extend(summary.errors.clone());
        summaries.push(summary);
    }

    Ok(Json(ImportResponse {
        summaries,
        total_files,
        total_objects,
        total_bytes,
        errors,
    }))
}

async fn get_onboarding_data(
    State(_state): State<AppState>,
) -> ApiResult<Json<OnboardingDataResponse>> {
    let repo = open_repo(None).map_err(api_error)?;
    let data = build_onboarding_data(&repo).map_err(api_error)?;
    Ok(Json(data))
}

async fn assign_project(
    State(_state): State<AppState>,
    Json(payload): Json<AssignProjectRequest>,
) -> ApiResult<Json<OnboardingDataResponse>> {
    let repo = open_repo(None).map_err(api_error)?;
    let identity = ensure_identity("default", None).map_err(api_error)?;
    let actor = identity_actor(&identity);
    let object_id = uuid::Uuid::parse_str(&payload.object_id)
        .map_err(|_| api_error(anyhow::anyhow!("Invalid object id")))?;
    let object_id = latticefs_base::model::ObjectID::from_uuid(object_id);
    let tag = Tag::new("project".to_string(), payload.project_id.clone(), actor);
    apply_tags(&repo, &object_id, vec![tag]).map_err(api_error)?;
    let data = build_onboarding_data(&repo).map_err(api_error)?;
    Ok(Json(data))
}

fn api_error(err: anyhow::Error) -> (StatusCode, Json<ApiError>) {
    (StatusCode::BAD_REQUEST, Json(ApiError::from(err)))
}

fn settings_path() -> PathBuf {
    default_home().join("gui").join("settings.json")
}

fn load_settings() -> anyhow::Result<OnboardingSettings> {
    let path = settings_path();
    if !path.exists() {
        return Ok(OnboardingSettings::default());
    }
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

fn save_settings(settings: &OnboardingSettings) -> anyhow::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

struct FolderDefinition {
    id: String,
    name: String,
    path: PathBuf,
    default_selected: bool,
    is_demo: bool,
}

fn demo_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LatticeDemo")
}

fn folder_definitions() -> Vec<FolderDefinition> {
    let mut folders = Vec::new();
    let documents = dirs::document_dir();
    let downloads = dirs::download_dir();
    let pictures = dirs::picture_dir();
    let projects = dirs::home_dir().map(|home| home.join("Projects"));

    if let Some(path) = documents {
        folders.push(FolderDefinition {
            id: "documents".to_string(),
            name: "Dokumente".to_string(),
            path,
            default_selected: true,
            is_demo: false,
        });
    }

    if let Some(path) = downloads {
        folders.push(FolderDefinition {
            id: "downloads".to_string(),
            name: "Downloads".to_string(),
            path,
            default_selected: true,
            is_demo: false,
        });
    }

    if let Some(path) = pictures {
        folders.push(FolderDefinition {
            id: "bilder".to_string(),
            name: "Bilder".to_string(),
            path,
            default_selected: false,
            is_demo: false,
        });
    }

    if let Some(path) = projects {
        folders.push(FolderDefinition {
            id: "projekte".to_string(),
            name: "Projekte".to_string(),
            path,
            default_selected: false,
            is_demo: false,
        });
    }

    let demo = demo_root();
    if demo.exists() {
        folders.push(FolderDefinition {
            id: "demo".to_string(),
            name: "Demo-Dateien".to_string(),
            path: demo,
            default_selected: false,
            is_demo: true,
        });
    }

    folders
}

fn build_folder_options() -> Vec<FolderOption> {
    folder_definitions()
        .into_iter()
        .map(|folder| FolderOption {
            id: folder.id,
            name: folder.name,
            path: folder.path.to_string_lossy().to_string(),
            exists: folder.path.exists(),
            default_selected: folder.default_selected,
            is_demo: folder.is_demo,
        })
        .collect()
}

fn create_demo_files(root: &Path) -> anyhow::Result<()> {
    let documents = root.join("Documents");
    let downloads = root.join("Downloads");
    let pictures = root.join("Pictures");
    let projects = root.join("Projects");
    let phoenix = projects.join("Phoenix");
    let aurora = projects.join("Aurora");
    let nebula = projects.join("Nebula");

    for dir in [
        &documents, &downloads, &pictures, &phoenix, &aurora, &nebula,
    ] {
        std::fs::create_dir_all(dir)?;
    }

    std::fs::write(
        documents.join("Projektplan_Phoenix.md"),
        "Projektplan für Phoenix\n\n- Ziele\n- Timeline\n",
    )?;
    std::fs::write(
        documents.join("Budget_2024.txt"),
        "Budget 2024\nMarketing: 12000\nForschung: 35000\n",
    )?;
    std::fs::write(
        downloads.join("setup_installer.exe"),
        "Pretend binary content",
    )?;
    std::fs::write(
        downloads.join("invoice_2024.pdf"),
        "%PDF-1.4\n1 0 obj\n<<>>\nendobj\n",
    )?;
    std::fs::write(
        pictures.join("Urlaub_Mallorca_2023.jpg"),
        "Not a real JPEG but enough for demo",
    )?;
    std::fs::write(
        phoenix.join("Meeting_Notes.md"),
        "Meeting Notes\n\n- Entscheidungen\n- Aufgaben\n",
    )?;
    std::fs::write(
        aurora.join("Pitch.pptx"),
        "Placeholder content for Aurora pitch deck",
    )?;
    std::fs::write(
        nebula.join("Roadmap.xlsx"),
        "Placeholder content for Nebula roadmap",
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let exe_path = downloads.join("setup_installer.exe");
        if let Ok(meta) = std::fs::metadata(&exe_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(exe_path, perms);
        }
    }

    Ok(())
}

async fn import_folder(
    repo: &LatticeRepo,
    folder: &FolderDefinition,
    actor: [u8; 32],
    settings: &OnboardingSettings,
) -> anyhow::Result<ImportSummary> {
    let entries = scanner::scan_path(&folder.path)?;
    let mut files = 0usize;
    let mut objects = 0usize;
    let mut bytes = 0u64;
    let mut errors = Vec::new();

    for entry in entries {
        files += 1;
        bytes += entry.size;
        let mut options = ImportOptions {
            tags: Vec::new(),
            extract_exif: repo.config.import.extract_exif,
            extract_id3: repo.config.import.extract_id3,
            extract_text: repo.config.import.extract_text,
            actor,
            base_path: if folder.path.is_dir() {
                Some(folder.path.clone())
            } else {
                None
            },
        };

        if folder.id != "demo" {
            options.tags.push(format!("folder:{}", folder.id));
        }

        match import_file(repo, &entry.path, &options).await {
            Ok(object_id) => {
                objects += 1;
                let tag_updates =
                    build_post_import_tags(folder, &entry.path, &options, settings, actor);
                if let Err(err) = apply_tags(repo, &object_id, tag_updates) {
                    errors.push(format!("{}: {}", entry.path.display(), err));
                }
            }
            Err(err) => errors.push(format!("{}: {}", entry.path.display(), err)),
        }
    }

    Ok(ImportSummary {
        folder_id: folder.id.clone(),
        files,
        objects,
        bytes,
        errors,
    })
}

fn build_post_import_tags(
    folder: &FolderDefinition,
    path: &Path,
    options: &ImportOptions,
    settings: &OnboardingSettings,
    actor: [u8; 32],
) -> Vec<Tag> {
    let mut tags = Vec::new();
    let mut folder_id = folder.id.clone();

    if folder.id == "demo" {
        if let Some(base) = &options.base_path {
            if let Ok(rel) = path.strip_prefix(base) {
                if let Some(component) = rel.components().next() {
                    let name = component.as_os_str().to_string_lossy().to_lowercase();
                    folder_id = match name.as_str() {
                        "documents" => "documents".to_string(),
                        "downloads" => "downloads".to_string(),
                        "pictures" => "bilder".to_string(),
                        "projects" => "projekte".to_string(),
                        _ => "demo".to_string(),
                    };
                }
            }
        }
    }

    tags.push(Tag::new("folder".to_string(), folder_id.clone(), actor));

    if folder_id == "downloads" {
        tags.push(Tag::new(
            "inbox".to_string(),
            "downloads".to_string(),
            actor,
        ));
        if settings.quarantine_downloads {
            tags.push(Tag::new(
                "risk".to_string(),
                "quarantine".to_string(),
                actor,
            ));
        }
    }

    if folder_id == "projekte" {
        if let Some(base) = &options.base_path {
            if let Ok(rel) = path.strip_prefix(base) {
                let mut components = rel.components();
                let first = components
                    .next()
                    .map(|component| component.as_os_str().to_string_lossy().to_string());
                let project = if folder.id == "demo"
                    && first
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case("Projects"))
                {
                    components
                        .next()
                        .map(|component| component.as_os_str().to_string_lossy().to_string())
                } else {
                    first
                };
                if let Some(project) = project {
                    if !project.is_empty() {
                        tags.push(Tag::new("project".to_string(), project, actor));
                    }
                }
            }
        }
    }

    let extension = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase());
    let is_executable = extension
        .as_deref()
        .is_some_and(|ext| matches!(ext, "exe" | "bat" | "cmd" | "sh" | "app"));

    if is_executable {
        tags.push(Tag::new(
            "auto:executable".to_string(),
            "true".to_string(),
            actor,
        ));
        if settings.execute_warning {
            tags.push(Tag::new("sys:trust".to_string(), "50".to_string(), actor));
        }
    }

    tags
}

fn apply_tags(
    repo: &LatticeRepo,
    object_id: &latticefs_base::model::ObjectID,
    tags: Vec<Tag>,
) -> anyhow::Result<()> {
    if tags.is_empty() {
        return Ok(());
    }
    let mut object = repo.metadata.load_object(object_id)?;
    let mut new_tags = Vec::new();

    for tag in tags {
        if object
            .tags
            .iter()
            .any(|existing| existing.key == tag.key && existing.value == tag.value)
        {
            continue;
        }
        object.add_tag(tag.clone());
        new_tags.push(tag.full_path());
    }

    if new_tags.is_empty() {
        return Ok(());
    }

    repo.metadata.store_object(&object)?;
    for tag in new_tags {
        repo.metadata.add_to_tag_index(&tag, object_id.as_bytes())?;
    }
    Ok(())
}

struct FileSummary {
    id: String,
    name: String,
    extension: Option<String>,
    size_bytes: u64,
    created_at: i64,
    tags: Vec<Tag>,
}

struct ViewDefinition {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    description: &'static str,
    color: &'static str,
    predicate: fn(&FileSummary) -> bool,
}

fn build_onboarding_data(repo: &LatticeRepo) -> anyhow::Result<OnboardingDataResponse> {
    let mut files = Vec::new();
    for item in repo.metadata.iter_objects() {
        let object = match item {
            Ok(obj) => obj,
            Err(err) => {
                tracing::warn!("Failed to read object: {err}");
                continue;
            }
        };
        let version = repo.metadata.load_version(&object.current_version)?;
        let (name, extension) = extract_name(&object.tags, &object.id.to_string());
        files.push(FileSummary {
            id: object.id.to_string(),
            name,
            extension,
            size_bytes: version.size_bytes,
            created_at: version.created_at,
            tags: object.tags,
        });
    }

    let view_defs = view_definitions();
    let mut views = Vec::new();
    let mut api_files = Vec::new();
    let mut view_counts: HashMap<&str, usize> = HashMap::new();

    for file in &files {
        let mut file_views = Vec::new();
        for view in &view_defs {
            if (view.predicate)(file) {
                file_views.push(view.id.to_string());
                *view_counts.entry(view.id).or_insert(0) += 1;
            }
        }
        api_files.push(OnboardingFile {
            id: file.id.clone(),
            name: file.name.clone(),
            extension: file.extension.clone(),
            size_bytes: file.size_bytes,
            created_at: timestamp_to_iso(file.created_at),
            tags: file.tags.iter().map(Tag::full_path).collect(),
            views: file_views,
        });
    }

    for view in view_defs {
        if view_counts.get(view.id).copied().unwrap_or(0) > 0 {
            views.push(OnboardingView {
                id: view.id.to_string(),
                name: view.name.to_string(),
                icon: view.icon.to_string(),
                description: view.description.to_string(),
                color: view.color.to_string(),
            });
        }
    }

    let projects = build_projects(&files);
    let highlighted_file_id = select_highlighted_file(&files);

    Ok(OnboardingDataResponse {
        views,
        files: api_files,
        projects,
        highlighted_file_id,
    })
}

fn view_definitions() -> Vec<ViewDefinition> {
    vec![
        ViewDefinition {
            id: "recent",
            name: "Neueste",
            icon: "Clock",
            description: "Zuletzt importierte Objekte",
            color: "primary",
            predicate: |file| is_recent(file, Duration::from_secs(60 * 60 * 24 * 7)),
        },
        ViewDefinition {
            id: "downloads",
            name: "Downloads",
            icon: "Download",
            description: "Alles aus deinen Downloads",
            color: "warning",
            predicate: |file| {
                has_tag(file, "folder", "downloads") || has_tag(file, "inbox", "downloads")
            },
        },
        ViewDefinition {
            id: "quarantine",
            name: "Quarantäne",
            icon: "Shield",
            description: "Objekte mit Sicherheitsflag",
            color: "warning",
            predicate: |file| {
                has_tag(file, "risk", "quarantine") || has_tag(file, "sys:trust", "50")
            },
        },
        ViewDefinition {
            id: "documents",
            name: "Dokumente",
            icon: "Grid",
            description: "Text, Tabellen und PDFs",
            color: "muted",
            predicate: |file| has_tag(file, "folder", "documents"),
        },
        ViewDefinition {
            id: "bilder",
            name: "Bilder",
            icon: "Grid",
            description: "Fotos und Bilder",
            color: "muted",
            predicate: |file| has_tag(file, "folder", "bilder"),
        },
        ViewDefinition {
            id: "projekte",
            name: "Projekte",
            icon: "Folder",
            description: "Projektbezogene Dateien",
            color: "secondary",
            predicate: |file| {
                has_tag(file, "folder", "projekte") || has_tag_prefix(file, "project")
            },
        },
    ]
}

fn is_recent(file: &FileSummary, window: Duration) -> bool {
    let created_at = file.created_at as i64;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_micros() as i64;
    now.saturating_sub(created_at) <= window.as_micros() as i64
}

fn has_tag(file: &FileSummary, key: &str, value: &str) -> bool {
    file.tags
        .iter()
        .any(|tag| tag.key == key && tag.value == value)
}

fn has_tag_prefix(file: &FileSummary, prefix: &str) -> bool {
    file.tags.iter().any(|tag| tag.key.starts_with(prefix))
}

fn build_projects(files: &[FileSummary]) -> Vec<OnboardingProject> {
    let mut projects = HashSet::new();
    for file in files {
        for tag in &file.tags {
            if tag.key == "project" {
                projects.insert(tag.value.clone());
            }
        }
    }

    let palette = ["#8B5CF6", "#38BDF8", "#F59E0B", "#34D399", "#F472B6"];

    projects
        .into_iter()
        .enumerate()
        .map(|(idx, name)| OnboardingProject {
            id: name.clone(),
            name,
            color: palette[idx % palette.len()].to_string(),
        })
        .collect()
}

fn select_highlighted_file(files: &[FileSummary]) -> Option<String> {
    files
        .iter()
        .find(|file| has_tag(file, "risk", "quarantine"))
        .or_else(|| {
            files
                .iter()
                .find(|file| has_tag(file, "inbox", "downloads"))
        })
        .or_else(|| {
            files
                .iter()
                .find(|file| file.extension.as_deref() == Some("exe"))
        })
        .map(|file| file.id.clone())
}

fn extract_name(tags: &[Tag], fallback: &str) -> (String, Option<String>) {
    let file_name = tags
        .iter()
        .find(|tag| tag.key == "auto:filename_b64")
        .and_then(|tag| URL_SAFE_NO_PAD.decode(&tag.value).ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_else(|| fallback.to_string());

    let extension = Path::new(&file_name)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase());

    (file_name, extension)
}

fn timestamp_to_iso(micros: i64) -> String {
    let secs = micros.div_euclid(1_000_000) as u64;
    let nanos = (micros.rem_euclid(1_000_000) as u32) * 1_000;
    let timestamp = UNIX_EPOCH + Duration::new(secs, nanos);
    humantime::format_rfc3339(timestamp).to_string()
}
