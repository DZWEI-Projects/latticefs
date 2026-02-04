use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::import::{import_file, ImportOptions};
use latticefs_base::LatticeRepo;
use std::path::PathBuf;

use super::common::{ensure_identity, identity_actor};

#[derive(Args, Debug)]
pub struct AddArgs {
    /// File to add
    pub file: PathBuf,
    /// Tags to attach (key:value)
    #[arg(long = "tag", short = 't')]
    pub tags: Vec<String>,
}

pub async fn run(repo: LatticeRepo, args: AddArgs) -> Result<()> {
    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let options = ImportOptions {
        tags: args.tags,
        extract_exif: repo.config.import.extract_exif,
        extract_id3: repo.config.import.extract_id3,
        extract_text: repo.config.import.extract_text,
        actor,
    };

    let object_id = import_file(&repo, &args.file, &options)
        .await
        .with_context(|| format!("Failed to add {}", args.file.display()))?;

    println!("Added object {}", object_id);
    Ok(())
}
