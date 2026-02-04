//! Import and export module for LatticeFS.

pub mod chunker;
pub mod metadata;
pub mod scanner;

use crate::error::{LatticeError, Result};
use crate::model::{ActorID, Object, ObjectID, ObjectType, Tag, Version, VersionID};
use crate::repo::LatticeRepo;
use crate::views::{BuiltinView, BuiltinViews, DynamicView};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub tags: Vec<String>,
    pub extract_exif: bool,
    pub extract_id3: bool,
    pub extract_text: bool,
    pub actor: ActorID,
    pub base_path: Option<std::path::PathBuf>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            extract_exif: true,
            extract_id3: true,
            extract_text: true,
            actor: [0u8; 32],
            base_path: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub files: usize,
    pub objects: usize,
    pub bytes: u64,
    pub errors: Vec<String>,
}

/// Import a path (file or directory) into the repository.
pub async fn import_path(
    repo: &LatticeRepo,
    path: &Path,
    options: &ImportOptions,
) -> Result<ImportReport> {
    let files = scanner::scan_path(path)?;
    let mut report = ImportReport::default();

    for entry in files {
        report.files += 1;
        report.bytes += entry.size;
        match import_file(repo, &entry.path, options).await {
            Ok(_) => report.objects += 1,
            Err(err) => report
                .errors
                .push(format!("{}: {}", entry.path.display(), err)),
        }
    }

    Ok(report)
}

/// Import a single file.
pub async fn import_file(
    repo: &LatticeRepo,
    path: &Path,
    options: &ImportOptions,
) -> Result<ObjectID> {
    let data = tokio::fs::read(path).await?;
    let manifest = repo.store_object_data(&data).await?;
    let manifest_hash = repo.metadata.store_manifest(&manifest)?;

    let object_id = ObjectID::new();
    let version = Version::new(
        object_id,
        None,
        manifest.merkle_root,
        manifest_hash,
        options.actor,
        data.len() as u64,
        manifest.chunks.len() as u32,
        None,
    );

    let mut object = Object::new(ObjectType::Blob, version.id, options.actor);
    object.id = object_id;

    // Source filename + relative path (base64url encoded for safe querying)
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        add_encoded_tag(&mut object, "auto:filename_b64", &file_name, options.actor);
    }
    if let Some(base_path) = &options.base_path {
        if let Ok(rel_path) = path.strip_prefix(base_path) {
            if !rel_path.as_os_str().is_empty() {
                // Normalize the relative path to use `/` separators for portability
                let mut rel_path_normalized = String::new();
                for (i, component) in rel_path.components().enumerate() {
                    if i > 0 {
                        rel_path_normalized.push('/');
                    }
                    rel_path_normalized.push_str(&component.as_os_str().to_string_lossy());
                }
                add_encoded_tag(
                    &mut object,
                    "auto:relpath_b64",
                    &rel_path_normalized,
                    options.actor,
                );
            }
        }
    }

    // User-provided tags
    for tag_str in &options.tags {
        let tag = Tag::parse(tag_str, options.actor)?;
        object.add_tag(tag);
    }

    // Extract metadata
    let meta_opts = metadata::MetadataOptions::from_import(options);
    let extracted = metadata::extract_metadata(path, options.actor, &meta_opts)?;
    for tag in extracted.tags {
        object.add_tag(tag);
    }

    // Store object + version
    repo.metadata.store_object(&object)?;
    repo.metadata.store_version(&version)?;
    repo.events.emit_sync(crate::events::Event::object_created(
        &object.id,
        &version.id,
        options.actor,
    ));

    // Tag index updates
    for tag in &object.tags {
        repo.metadata
            .add_to_tag_index(&tag.full_path(), object.id.as_bytes())?;
    }

    // Store extracted text if any
    if let Some(text) = extracted.text {
        repo.metadata.store_text(&object_id, &text)?;
    }

    Ok(object_id)
}

fn add_encoded_tag(object: &mut Object, key: &str, value: &str, actor: ActorID) {
    if value.is_empty() {
        return;
    }
    let encoded = URL_SAFE_NO_PAD.encode(value.as_bytes());
    object.add_tag(Tag::new(key.to_string(), encoded, actor));
}

/// Export mode for object/view export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportMode {
    Tree,
    Archive,
}

impl std::str::FromStr for ExportMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tree" => Ok(ExportMode::Tree),
            "archive" => Ok(ExportMode::Archive),
            _ => Err(format!("Unknown export mode: {}", s)),
        }
    }
}

/// Export a single object (by ID) to a file or directory.
pub async fn export_object(
    repo: &LatticeRepo,
    object_id: &ObjectID,
    version_id: Option<VersionID>,
    output: &Path,
    mode: ExportMode,
) -> Result<()> {
    let (data, filename) = read_object_bytes(repo, object_id, version_id).await?;

    match mode {
        ExportMode::Tree => {
            let out_path = if output.is_dir() {
                output.join(filename)
            } else {
                output.to_path_buf()
            };
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(out_path, data)?;
        }
        ExportMode::Archive => {
            let file = std::fs::File::create(output)?;
            let mut builder = tar::Builder::new(file);
            append_tar_entry(&mut builder, &filename, &data)?;
            builder.finish()?;
        }
    }

    Ok(())
}

/// Export a view (built-in or dynamic) to tree or archive.
pub async fn export_view(
    repo: &LatticeRepo,
    view_name: &str,
    output: &Path,
    mode: ExportMode,
) -> Result<()> {
    let object_ids = resolve_view(repo, view_name)?;

    match mode {
        ExportMode::Tree => {
            std::fs::create_dir_all(output)?;
            for object_id in object_ids {
                let (data, filename) = read_object_bytes(repo, &object_id, None).await?;
                let out_path = output.join(filename);
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(out_path, data)?;
            }
        }
        ExportMode::Archive => {
            let file = std::fs::File::create(output)?;
            let mut builder = tar::Builder::new(file);
            for object_id in object_ids {
                let (data, filename) = read_object_bytes(repo, &object_id, None).await?;
                append_tar_entry(&mut builder, &filename, &data)?;
            }
            builder.finish()?;
        }
    }

    Ok(())
}

fn append_tar_entry(
    builder: &mut tar::Builder<std::fs::File>,
    name: &str,
    data: &[u8],
) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o444);
    header.set_cksum();
    builder.append_data(&mut header, name, data)?;
    Ok(())
}

fn resolve_view(repo: &LatticeRepo, view_name: &str) -> Result<Vec<ObjectID>> {
    if let Some(builtin) = BuiltinView::by_name(view_name) {
        let builtin_views = BuiltinViews::new(&repo.metadata);
        return builtin_views.evaluate(builtin);
    }

    let view = repo.metadata.load_view(view_name)?;
    let mut dynamic = DynamicView::new(&view.query, &repo.metadata)?;
    dynamic.evaluate()
}

async fn read_object_bytes(
    repo: &LatticeRepo,
    object_id: &ObjectID,
    version_id: Option<VersionID>,
) -> Result<(Vec<u8>, String)> {
    let object = repo.metadata.load_object(object_id)?;
    repo.authorize_object_permission(&object, crate::crypto::Permission::Read, false)?;
    if crate::security::is_quarantined_executable(&object.tags) {
        return Err(LatticeError::Unauthorized {
            permission: "read".to_string(),
            object: object_id.to_string(),
        });
    }
    let version = match version_id {
        Some(v) => repo.metadata.load_version(&v)?,
        None => repo.metadata.load_version(&object.current_version)?,
    };
    if version.object_id != *object_id {
        return Err(LatticeError::VersionNotFound {
            id: format!("{}", version.id),
        });
    }

    let manifest = repo.metadata.load_manifest(&version.manifest_ref)?;
    let data = repo.chunks.retrieve_object(&manifest).await?;
    Ok((data, object_id.to_string()))
}

/// Export an object or view, auto-detecting by reference name.
pub async fn export_ref(
    repo: &LatticeRepo,
    reference: &str,
    output: &Path,
    mode: ExportMode,
) -> Result<()> {
    if let Ok(uuid) = uuid::Uuid::parse_str(reference) {
        let object_id = ObjectID::from_uuid(uuid);
        return export_object(repo, &object_id, None, output, mode).await;
    }

    export_view(repo, reference, output, mode).await
}

/// Resolve a ref string to an ObjectID (UUID) if possible.
pub fn resolve_object_id(reference: &str) -> Result<ObjectID> {
    let uuid = uuid::Uuid::parse_str(reference).map_err(|e| {
        LatticeError::Serialization(format!("Invalid object reference '{}': {}", reference, e))
    })?;
    Ok(ObjectID::from_uuid(uuid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_actor() -> ActorID {
        [0u8; 32]
    }

    fn find_tag<'a>(object: &'a Object, key: &str) -> Option<&'a Tag> {
        object.tags.iter().find(|t| t.key == key)
    }

    #[tokio::test]
    async fn test_import_adds_filename_tag_b64() {
        let temp = tempdir().unwrap();
        let repo_path = temp.path().join("repo");
        let repo = LatticeRepo::open_at(&repo_path).unwrap();

        let file_path = temp.path().join("Report Final (v1).txt");
        tokio::fs::write(&file_path, b"hello").await.unwrap();

        let options = ImportOptions {
            tags: Vec::new(),
            extract_exif: false,
            extract_id3: false,
            extract_text: false,
            actor: test_actor(),
            base_path: None,
        };

        let object_id = import_file(&repo, &file_path, &options).await.unwrap();
        let object = repo.metadata.load_object(&object_id).unwrap();

        let tag = find_tag(&object, "auto:filename_b64").expect("filename tag");
        let expected = URL_SAFE_NO_PAD.encode("Report Final (v1).txt".as_bytes());
        assert_eq!(tag.value, expected);
        assert!(find_tag(&object, "auto:relpath_b64").is_none());
    }

    #[tokio::test]
    async fn test_import_adds_relpath_tag_b64() {
        let temp = tempdir().unwrap();
        let repo_path = temp.path().join("repo");
        let repo = LatticeRepo::open_at(&repo_path).unwrap();

        let import_root = temp.path().join("import-root");
        let nested_dir = import_root.join("docs");
        std::fs::create_dir_all(&nested_dir).unwrap();
        let file_path = nested_dir.join("Report Final (v1).txt");
        tokio::fs::write(&file_path, b"hello").await.unwrap();

        let options = ImportOptions {
            tags: Vec::new(),
            extract_exif: false,
            extract_id3: false,
            extract_text: false,
            actor: test_actor(),
            base_path: Some(import_root.clone()),
        };

        let object_id = import_file(&repo, &file_path, &options).await.unwrap();
        let object = repo.metadata.load_object(&object_id).unwrap();

        let filename_tag = find_tag(&object, "auto:filename_b64").expect("filename tag");
        let expected_filename = URL_SAFE_NO_PAD.encode("Report Final (v1).txt".as_bytes());
        assert_eq!(filename_tag.value, expected_filename);

        let rel_path = file_path.strip_prefix(&import_root).unwrap();
        let expected_rel = URL_SAFE_NO_PAD.encode(rel_path.to_string_lossy().as_bytes());
        let rel_tag = find_tag(&object, "auto:relpath_b64").expect("relpath tag");
        assert_eq!(rel_tag.value, expected_rel);
    }
}
