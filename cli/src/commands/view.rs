use anyhow::Result;
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::query::{Explainer, parse};
use latticefs_base::views::{BuiltinView, Locale, View, ViewID};

use super::common::{
    ResolvedView, ensure_identity, identity_actor, resolve_dynamic_view, resolve_object_id,
    resolve_view_reference,
};

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
    #[arg(long)]
    /// Parent view name or ID
    pub parent: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// View name or ID
    pub name: String,
}

#[derive(Args, Debug)]
pub struct ExplainArgs {
    pub reference: String,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    /// View name or ID
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

    let mut view = View::new(args.name.clone(), args.query, actor);

    if let Some(parent_ref) = args.parent {
        let parent = resolve_dynamic_view(&repo, &parent_ref)?;
        view = view.with_parent(parent.id);
    }

    repo.metadata.store_view_with_max_depth(
        &view,
        repo.config.experimental.nested_views.max_parent_depth,
    )?;
    println!("Created view {} ({})", view.name, view.id);
    Ok(())
}

async fn list(repo: LatticeRepo) -> Result<()> {
    let locale = Locale::from_system();
    println!("Built-in views:");
    for view in BuiltinView::all() {
        println!(
            "- {}: {}",
            view.name_localized(locale),
            view.description_localized(locale)
        );
    }

    println!("\nDynamic views:");
    let views = repo.metadata.list_views()?;

    // Build tree structure
    let mut view_map: std::collections::HashMap<ViewID, Vec<&View>> =
        std::collections::HashMap::new();
    let mut root_views = Vec::new();

    for view in &views {
        if let Some(parent_id) = view.parent_id {
            view_map.entry(parent_id).or_default().push(view);
        } else {
            root_views.push(view);
        }
    }

    // Print tree recursively
    fn print_view_tree(
        view: &View,
        view_map: &std::collections::HashMap<ViewID, Vec<&View>>,
        indent: usize,
    ) {
        let prefix = "  ".repeat(indent);
        println!(
            "{}- {} (id: {}): {}",
            prefix, view.name, view.id, view.query
        );

        if let Some(children) = view_map.get(&view.id) {
            for child in children {
                print_view_tree(child, view_map, indent + 1);
            }
        }
    }

    for view in root_views {
        print_view_tree(view, &view_map, 0);
    }

    Ok(())
}

async fn delete(repo: LatticeRepo, args: DeleteArgs) -> Result<()> {
    let view = resolve_dynamic_view(&repo, &args.name)?;
    repo.metadata.delete_view(&view.name)?;
    println!("Deleted view {} ({})", view.name, view.id);
    Ok(())
}

async fn explain(repo: LatticeRepo, args: ExplainArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;

    let query = if let Some(q) = args.query {
        q
    } else if let Some(view_name) = args.view {
        match resolve_view_reference(&repo, &view_name)? {
            ResolvedView::Builtin(builtin) => builtin.query().to_string(),
            ResolvedView::Dynamic(view) => view.query,
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
