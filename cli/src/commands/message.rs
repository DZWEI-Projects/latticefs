use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::Permission;

use super::common::parse_ref_with_version;

#[derive(Subcommand, Debug)]
pub enum MessageCommand {
    Set(MessageSetArgs),
}

#[derive(Args, Debug)]
pub struct MessageArgs {
    #[command(subcommand)]
    pub command: MessageCommand,
}

#[derive(Args, Debug)]
pub struct MessageSetArgs {
    /// Object reference (optionally @version)
    pub reference: String,
    /// New commit message (can be empty)
    #[arg(short, long)]
    pub message: Option<String>,
    /// Clear the commit message
    #[arg(long)]
    pub clear: bool,
}

pub async fn run(repo: LatticeRepo, cmd: MessageCommand) -> Result<()> {
    match cmd {
        MessageCommand::Set(args) => set_message(repo, args).await,
    }
}

async fn set_message(repo: LatticeRepo, args: MessageSetArgs) -> Result<()> {
    if args.clear && args.message.is_some() {
        return Err(anyhow::anyhow!("Use either --message or --clear, not both"));
    }

    let (object_id, version_id) = parse_ref_with_version(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Write, false)?;

    let message = if args.clear {
        None
    } else {
        Some(
            args.message
                .ok_or_else(|| anyhow::anyhow!("Missing --message (or use --clear)"))?,
        )
    };

    let updated = repo.update_version_message(&object_id, version_id, message)?;
    println!("Set message for {}", updated.id);
    Ok(())
}
