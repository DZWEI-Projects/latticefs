use anyhow::Result;
use clap::{Args, Subcommand};
use latticefs_base::query::{parse, Explainer};
use latticefs_base::views::{BuiltinView, View};
use latticefs_base::LatticeRepo;

use super::common::{ensure_identity, identity_actor, resolve_object_id};

#[derive(Subcommand, Debug)]
pub enum ViewCommand {
    Create(CreateArgs),
    List(ListArgs),
    Delete(DeleteArgs),
    Explain(ExplainArgs),
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    #[command(subcommand)]
    pub command: ViewCommand,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    pub name: String,
    #[arg(long)]
    pub query: String,
}

#[derive(Args, Debug)]
pub struct ListArgs {}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    pub name: String,
}

#[derive(Args, Debug)]
pub struct ExplainArgs {
    pub reference: String,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub view: Option<String>,
}

pub async fn run(repo: LatticeRepo, command: ViewCommand) -> Result<()> {
    match command {
        ViewCommand::Create(args) => create(repo, args).await,
        ViewCommand::List(_args) => list(repo).await,
        ViewCommand::Delete(args) => delete(repo, args).await,
        ViewCommand::Explain(args) => explain(repo, args).await,
    }
}

async fn create(repo: LatticeRepo, args: CreateArgs) -> Result<()> {
    validate_view_name(&args.name)?;
    parse(&args.query)?;

    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let view = View::new(args.name.clone(), args.query, actor);
    repo.metadata.store_view(&view)?;
    println!("Created view {}", view.name);
    Ok(())
}

async fn list(repo: LatticeRepo) -> Result<()> {
    println!("Built-in views:");
    for view in BuiltinView::all() {
        println!("- {}: {}", view.name(), view.description());
    }

    println!("\nDynamic views:");
    for view in repo.metadata.list_views()? {
        println!("- {}: {}", view.name, view.query);
    }
    Ok(())
}

async fn delete(repo: LatticeRepo, args: DeleteArgs) -> Result<()> {
    repo.metadata.delete_view(&args.name)?;
    println!("Deleted view {}", args.name);
    Ok(())
}

async fn explain(repo: LatticeRepo, args: ExplainArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;

    let query = if let Some(q) = args.query {
        q
    } else if let Some(view_name) = args.view {
        if let Some(builtin) = BuiltinView::by_name(&view_name) {
            builtin.query().to_string()
        } else {
            repo.metadata.load_view(&view_name)?.query
        }
    } else {
        // Default to built-in "Recent" view for convenience
        BuiltinView::Recent.query().to_string()
    };

    let parsed = parse(&query)?;
    let explainer = Explainer::new(&repo.metadata);
    let explanation = explainer.explain(&object_id, &parsed)?;
    println!("{}", explanation);
    Ok(())
}

fn validate_view_name(name: &str) -> Result<()> {
    if name.contains('/') || name.contains('\0') {
        return Err(anyhow::anyhow!("Invalid view name"));
    }
    Ok(())
}
