use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::import::{export_object, ExportMode};
use latticefs_base::ipc::client;
use latticefs_base::storage::compute_hash;
use latticefs_base::LatticeRepo;

use super::common::{ensure_identity, identity_actor, resolve_object_id};

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Object reference (UUID or alias)
    pub reference: String,
    /// Don't register with watcher daemon
    #[arg(long)]
    pub no_watch: bool,
    /// Commit message template override
    #[arg(short, long)]
    pub message: Option<String>,
}

pub async fn run(repo: LatticeRepo, args: EditArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let object = repo.metadata.load_object(&object_id)?;

    // Determine display name from tags
    let name = display_name_from_tags(&object.tags, &object_id);
    let safe_name = name.replace(['/', '\\'], "_");

    // Export to watch directory
    let watch_dir = repo.config.watch_dir();
    std::fs::create_dir_all(&watch_dir)?;
    let output_path = watch_dir.join(format!("{}_{}", object_id, safe_name));

    export_object(&repo, &object_id, None, &output_path, ExportMode::Tree)
        .await
        .with_context(|| format!("Failed to export object {}", object_id))?;

    println!("Exported to {}", output_path.display());

    // Compute content hash for change detection
    let content = std::fs::read(&output_path)?;
    let content_hash = compute_hash(&content);

    // Register with watcher daemon (best-effort)
    if !args.no_watch {
        let socket_path = repo.config.socket_path();
        if client::is_daemon_running(&socket_path) {
            let actor = match ensure_identity("default", None) {
                Ok(id) => identity_actor(&id),
                Err(_) => [0u8; 32],
            };

            match client::send_watch_register(
                &socket_path,
                &output_path,
                &object_id.to_string(),
                actor,
                content_hash,
                &name,
            )
            .await
            {
                Ok(true) => {
                    println!("Registered with watcher daemon (auto-versioning enabled)");
                }
                Ok(false) => {
                    eprintln!("Warning: Failed to register with watcher daemon");
                }
                Err(e) => {
                    eprintln!("Warning: Could not register with watcher daemon: {}", e);
                }
            }
        } else {
            eprintln!(
                "Hint: Start the watcher daemon with `lfs watchd start` for auto-versioning"
            );
        }
    }

    // Open with system default app
    open_with_default_app(&output_path)?;

    Ok(())
}

fn display_name_from_tags(tags: &[latticefs_base::model::Tag], object_id: &latticefs_base::model::ObjectID) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    for tag in tags {
        if tag.key == "auto:filename_b64" {
            if let Ok(decoded) = URL_SAFE_NO_PAD.decode(&tag.value) {
                if let Ok(s) = String::from_utf8(decoded) {
                    return s;
                }
            }
        }
    }

    for tag in tags {
        if tag.key == "auto:relpath_b64" {
            if let Ok(decoded) = URL_SAFE_NO_PAD.decode(&tag.value) {
                if let Ok(s) = String::from_utf8(decoded) {
                    if let Some(base) = std::path::Path::new(&s)
                        .file_name()
                        .and_then(|n| n.to_str())
                    {
                        return base.to_string();
                    }
                    if !s.is_empty() {
                        return s;
                    }
                }
            }
        }
    }

    for tag in tags {
        if tag.key == "name" || tag.key == "auto:filename" {
            return tag.value.clone();
        }
    }

    object_id.to_string()
}

fn open_with_default_app(path: &std::path::Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .context("Failed to open file with default application")?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .context("Failed to open file with default application")?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .context("Failed to open file with default application")?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        println!("File exported to: {}", path.display());
        Ok(())
    }
}
