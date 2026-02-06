use anyhow::Result;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use latticefs_base::LatticeRepo;
use latticefs_base::import::{ImportOptions, import_file, scanner};
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
        base_path: if args.path.is_dir() {
            Some(args.path.clone())
        } else {
            None
        },
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
        Err(anyhow::anyhow!(
            "Import completed with {} errors",
            errors.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        args: ImportArgs,
    }

    #[test]
    fn parses_import_path_only() {
        let cli = Cli::parse_from(["import", "/tmp/input.txt"]);
        assert_eq!(cli.args.path, PathBuf::from("/tmp/input.txt"));
        assert!(cli.args.tags.is_empty());
    }

    #[test]
    fn parses_multiple_tags() {
        let cli = Cli::parse_from([
            "import",
            "assets",
            "--tag",
            "kind:image",
            "-t",
            "project:lattice",
        ]);

        assert_eq!(cli.args.path, PathBuf::from("assets"));
        assert_eq!(
            cli.args.tags,
            vec!["kind:image".to_string(), "project:lattice".to_string()]
        );
    }
}
