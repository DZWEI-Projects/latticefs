use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::Permission;
use latticefs_base::model::Tag;

use super::common::{ensure_identity, identity_actor, resolve_object_id};

#[derive(Subcommand, Debug)]
pub enum TrustCommand {
    Get(GetArgs),
    Set(SetArgs),
}

#[derive(Subcommand, Debug)]
pub enum QuarantineCommand {
    List,
}

#[derive(Args, Debug)]
pub struct TrustArgs {
    #[command(subcommand)]
    pub command: TrustCommand,
}

#[derive(Args, Debug)]
pub struct QuarantineArgs {
    #[command(subcommand)]
    pub command: QuarantineCommand,
}

#[derive(Args, Debug)]
pub struct GetArgs {
    pub reference: String,
}

#[derive(Args, Debug)]
pub struct SetArgs {
    pub reference: String,
    pub state: String,
}

pub async fn run(repo: LatticeRepo, cmd: TrustCommand) -> Result<()> {
    match cmd {
        TrustCommand::Get(args) => get(repo, args).await,
        TrustCommand::Set(args) => set(repo, args).await,
    }
}

pub async fn quarantine(repo: LatticeRepo, cmd: QuarantineCommand) -> Result<()> {
    match cmd {
        QuarantineCommand::List => list_quarantine(repo).await,
    }
}

async fn get(repo: LatticeRepo, args: GetArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;

    let value = trust_level(&object.tags);
    let label = trust_label(value);
    println!("{}: {} ({})", object_id, value, label);
    Ok(())
}

async fn set(repo: LatticeRepo, args: SetArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let value = trust_value(&args.state)?;

    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Write, false)?;
    repo.enforce_rate_limit(1)?;

    let removed: Vec<_> = object
        .tags
        .iter()
        .filter(|t| t.key == "sys:trust")
        .cloned()
        .collect();
    object.tags.retain(|t| t.key != "sys:trust");
    for tag in removed {
        repo.metadata
            .remove_from_tag_index(&tag.full_path(), object_id.as_bytes())?;
    }
    let tag = Tag::new("sys:trust".to_string(), value.to_string(), actor);
    object.add_tag(tag.clone());
    repo.metadata
        .add_to_tag_index(&tag.full_path(), object_id.as_bytes())?;
    repo.metadata.store_object(&object)?;

    println!("Set trust {} -> {}", object_id, value);
    Ok(())
}

async fn list_quarantine(repo: LatticeRepo) -> Result<()> {
    let mut quarantined = Vec::new();
    for object in repo.metadata.iter_objects() {
        let object = object?;
        if trust_level(&object.tags) <= 25 {
            quarantined.push(object.id);
        }
    }

    if quarantined.is_empty() {
        println!("No quarantined objects");
    } else {
        for id in quarantined {
            println!("{}", id);
        }
    }

    Ok(())
}

fn trust_value(state: &str) -> Result<u8> {
    match state.to_lowercase().as_str() {
        "untrusted" => Ok(0),
        "quarantined" => Ok(25),
        "trusted" => Ok(75),
        "approved" => Ok(100),
        _ => state
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("Invalid trust state: {}", state)),
    }
}

fn trust_label(value: u8) -> &'static str {
    match value {
        0 => "untrusted",
        25 => "quarantined",
        75 => "trusted",
        100 => "approved",
        _ => "custom",
    }
}

fn trust_level(tags: &[Tag]) -> u8 {
    tags.iter()
        .find(|t| t.key == "sys:trust")
        .and_then(|t| t.value.parse::<u8>().ok())
        .unwrap_or(75)
}
