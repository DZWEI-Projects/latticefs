use anyhow::Result;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use latticefs_base::import::{import_file, scanner, ImportOptions};
use latticefs_base::LatticeRepo;
use std::path::PathBuf;

use super::common::{ensure_identity, identity_actor};

#[derive(Args, Debug)]
pub struct ImportArgs {
    /// Path to import (file or directory)
    pub path: PathBuf,
    /// Tags to attach (key:value)
    #[arg(long = "tag", short = 't')]
    pub tags: Vec<String>,
}

pub async fn run(repo: LatticeRepo, args: ImportArgs) -> Result<()> {
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

    let entries = scanner::scan_path(&args.path)?;
    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {pos}/{len} {msg}")
            .unwrap(),
    );

    let mut errors = Vec::new();
    for entry in entries {
        pb.set_message(entry.path.display().to_string());
        if let Err(err) = import_file(&repo, &entry.path, &options).await {
            errors.push(format!("{}: {}", entry.path.display(), err));
        }
        pb.inc(1);
    }
    pb.finish_and_clear();

    if errors.is_empty() {
        println!("Import completed successfully");
        Ok(())
    } else {
        for err in &errors {
            eprintln!("Import error: {}", err);
        }
        Err(anyhow::anyhow!("Import completed with {} errors", errors.len()))
    }
}
