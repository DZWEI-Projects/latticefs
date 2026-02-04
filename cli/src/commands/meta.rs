use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::Permission;

use super::common::resolve_object_id;

#[derive(Args, Debug)]
pub struct MetaArgs {
    /// Object reference
    pub reference: String,
    /// Show extracted text
    #[arg(long)]
    pub text: bool,
    /// Show metadata tags
    #[arg(long)]
    pub tags: bool,
    /// Include all tags (not just auto:*)
    #[arg(long)]
    pub all_tags: bool,
}

pub async fn run(repo: LatticeRepo, args: MetaArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Read, false)?;

    let mut show_tags = args.tags;
    let mut show_text = args.text;
    if !show_tags && !show_text {
        show_tags = true;
        show_text = true;
    }

    if show_tags {
        let mut tags: Vec<_> = if args.all_tags {
            object.tags
        } else {
            object
                .tags
                .into_iter()
                .filter(|t| t.is_auto_generated())
                .collect()
        };
        tags.sort_by(|a, b| a.full_path().cmp(&b.full_path()));

        println!("Tags:");
        if tags.is_empty() {
            println!("(none)");
        } else {
            for tag in tags {
                println!("- {}", tag.full_path());
            }
        }
    }

    if show_text {
        let text = repo.metadata.load_text(&object_id)?;
        if show_tags {
            println!("\nText:");
        } else {
            println!("Text:");
        }
        match text {
            Some(t) if !t.is_empty() => print!("{}", t),
            _ => println!("(none)"),
        }
    }

    Ok(())
}
