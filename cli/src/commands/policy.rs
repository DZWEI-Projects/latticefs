use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::Permission;
use latticefs_base::model::{Policy, PolicyTemplate};

use super::common::resolve_object_id;

#[derive(Subcommand, Debug)]
pub enum PolicyCommand {
    Create(CreateArgs),
    Apply(ApplyArgs),
    Remove(RemoveArgs),
}

#[derive(Args, Debug)]
pub struct PolicyArgs {
    #[command(subcommand)]
    pub command: PolicyCommand,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    pub name: String,
    #[arg(long)]
    pub template: String,
}

#[derive(Args, Debug)]
pub struct ApplyArgs {
    pub reference: String,
    pub policy: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    pub reference: String,
    pub policy: String,
}

pub async fn run(repo: LatticeRepo, cmd: PolicyCommand) -> Result<()> {
    match cmd {
        PolicyCommand::Create(args) => create(repo, args).await,
        PolicyCommand::Apply(args) => apply(repo, args).await,
        PolicyCommand::Remove(args) => remove(repo, args).await,
    }
}

async fn create(repo: LatticeRepo, args: CreateArgs) -> Result<()> {
    let template: PolicyTemplate = args
        .template
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    repo.enforce_rate_limit(1)?;
    let policy = Policy::from_template(args.name.clone(), template);
    repo.metadata.store_policy(&policy)?;
    println!("Created policy {}", policy.name);
    Ok(())
}

async fn apply(repo: LatticeRepo, args: ApplyArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let policy = repo.metadata.load_policy(&args.policy)?;

    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Admin, false)?;
    repo.enforce_rate_limit(1)?;
    object.add_policy(policy.id);
    repo.metadata.store_object(&object)?;

    println!("Applied policy {} to {}", policy.name, object_id);
    Ok(())
}

async fn remove(repo: LatticeRepo, args: RemoveArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let policy = repo.metadata.load_policy(&args.policy)?;

    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Admin, false)?;
    repo.enforce_rate_limit(1)?;
    object.policy_refs.retain(|id| *id != policy.id);
    repo.metadata.store_object(&object)?;

    println!("Removed policy {} from {}", policy.name, object_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        args: PolicyArgs,
    }

    #[test]
    fn parses_create_subcommand() {
        let cli = Cli::parse_from(["policy", "create", "strict", "--template", "archive"]);
        match cli.args.command {
            PolicyCommand::Create(args) => {
                assert_eq!(args.name, "strict");
                assert_eq!(args.template, "archive");
            }
            _ => panic!("expected create subcommand"),
        }
    }

    #[test]
    fn parses_apply_subcommand() {
        let cli = Cli::parse_from(["policy", "apply", "obj-ref", "strict"]);
        match cli.args.command {
            PolicyCommand::Apply(args) => {
                assert_eq!(args.reference, "obj-ref");
                assert_eq!(args.policy, "strict");
            }
            _ => panic!("expected apply subcommand"),
        }
    }

    #[test]
    fn policy_requires_subcommand() {
        let err = Cli::try_parse_from(["policy"]).unwrap_err();
        assert!(matches!(
            err.kind(),
            clap::error::ErrorKind::MissingSubcommand
                | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        ));
    }
}
