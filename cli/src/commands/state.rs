use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::Permission;
use latticefs_base::model::State;

use super::common::parse_ref_with_version;

#[derive(Subcommand, Debug)]
pub enum StateCommand {
    Set(StateSetArgs),
}

#[derive(Args, Debug)]
pub struct StateArgs {
    #[command(subcommand)]
    pub command: StateCommand,
}

#[derive(Args, Debug)]
pub struct StateSetArgs {
    /// Object reference (optionally @version)
    pub reference: String,
    /// New state (draft|review|approved|discarded|sealed|archived)
    pub state: String,
}

pub async fn run(repo: LatticeRepo, cmd: StateCommand) -> Result<()> {
    match cmd {
        StateCommand::Set(args) => set_state(repo, args).await,
    }
}

async fn set_state(repo: LatticeRepo, args: StateSetArgs) -> Result<()> {
    let (object_id, version_id) = parse_ref_with_version(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Write, false)?;
    repo.enforce_rate_limit(1)?;

    let target_state: State = args.state.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    let version_id = version_id.unwrap_or(object.current_version);
    let mut version = repo.metadata.load_version(&version_id)?;

    if version.object_id != object_id {
        return Err(anyhow::anyhow!("Version does not belong to object"));
    }

    let from = version.state;
    version.transition_state(target_state).map_err(|_| {
        latticefs_base::LatticeError::InvalidStateTransition {
            from: from.to_string(),
            to: target_state.to_string(),
        }
    })?;

    repo.metadata.store_version(&version)?;
    println!("Set state {} -> {} for {}", from, version.state, version.id);
    Ok(())
}
