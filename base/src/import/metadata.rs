//! Metadata extraction for import operations.

use crate::error::Result;
use crate::model::{ActorID, Tag};
use id3::TagLike;
use mime_guess::MimeGuess;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub tags: Vec<Tag>,
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MetadataOptions {
    pub extract_exif: bool,
    pub extract_id3: bool,
    pub extract_text: bool,
}

impl MetadataOptions {
    pub fn from_import(options: &super::ImportOptions) -> Self {
        Self {
            extract_exif: options.extract_exif,
            extract_id3: options.extract_id3,
            extract_text: options.extract_text,
        }
    }
}

pub fn extract_metadata(path: &Path, actor: ActorID, options: &MetadataOptions) -> Result<ExtractedMetadata> {
    let mut tags: Vec<Tag> = Vec::new();

    // MIME type tag
    let mime = MimeGuess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    tags.push(Tag::new("auto:mimetype".to_string(), mime.clone(), actor));

    // Executable flag (Unix only)
    if is_executable(path)? {
        tags.push(Tag::new("auto:executable".to_string(), "true".to_string(), actor));
    }

    // EXIF metadata
    if options.extract_exif && mime.starts_with("image/") {
        if let Ok(file) = std::fs::File::open(path) {
            let mut bufreader = std::io::BufReader::new(file);
            if let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) {
                for field in exif.fields() {
                    let tag_name = format!("auto:exif:{}", field.tag.to_string().to_lowercase());
                    let value = field.display_value().with_unit(&exif).to_string();
                    tags.push(Tag::new(tag_name, value, actor));
                }
            }
        }
    }

    // ID3 metadata
    if options.extract_id3 && mime.starts_with("audio/") {
        if let Ok(tag) = id3::Tag::read_from_path(path) {
            if let Some(title) = tag.title() {
                tags.push(Tag::new("auto:id3:title".to_string(), title.to_string(), actor));
            }
            if let Some(artist) = tag.artist() {
                tags.push(Tag::new("auto:id3:artist".to_string(), artist.to_string(), actor));
            }
            if let Some(album) = tag.album() {
                tags.push(Tag::new("auto:id3:album".to_string(), album.to_string(), actor));
            }
            if let Some(genre) = tag.genre() {
                tags.push(Tag::new("auto:id3:genre".to_string(), genre.to_string(), actor));
            }
            if let Some(year) = tag.year() {
                tags.push(Tag::new("auto:id3:year".to_string(), year.to_string(), actor));
            }
            if let Some(track) = tag.track() {
                tags.push(Tag::new("auto:id3:track".to_string(), track.to_string(), actor));
            }
        }
    }

    // Text extraction
    let mut extracted_text: Option<String> = None;
    if options.extract_text {
        if mime == "application/pdf" {
            if let Ok(text) = pdf_extract::extract_text(path) {
                if !text.is_empty() {
                    tags.push(Tag::new("auto:text".to_string(), "true".to_string(), actor));
                    extracted_text = Some(text);
                }
            }
        } else if mime.starts_with("text/") {
            if let Ok(text) = std::fs::read_to_string(path) {
                if !text.is_empty() {
                    tags.push(Tag::new("auto:text".to_string(), "true".to_string(), actor));
                    extracted_text = Some(text);
                }
            }
        }
    }

    Ok(ExtractedMetadata {
        tags,
        text: extracted_text,
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path)?;
    let mode = metadata.permissions().mode();
    Ok((mode & 0o111) != 0)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> Result<bool> {
    Ok(false)
}
