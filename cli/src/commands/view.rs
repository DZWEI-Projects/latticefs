use anyhow::Result;
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::query::{Explainer, Query, parse};
use latticefs_base::views::{
    BuiltinView, Locale, View, ViewID, collect_descendants, resolve_effective_query,
    validate_parent_assignment,
};
use std::collections::HashMap;

use super::common::{
    ResolvedView, dynamic_view_path, ensure_identity, identity_actor, resolve_dynamic_view,
    resolve_object_id, resolve_view_reference,
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
    /// Parent view reference (UUID, path, or unique name)
    #[arg(long)]
    pub parent: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// View reference (UUID, path, or unique name)
    pub reference: String,
    /// Delete this view and all descendants.
    #[arg(long, conflicts_with = "detach_children")]
    pub cascade: bool,
    /// Reparent direct children to root before deleting.
    #[arg(long = "detach-children", conflicts_with = "cascade")]
    pub detach_children: bool,
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

    let parent_id = match args.parent {
        Some(parent_ref) => Some(resolve_dynamic_view(&repo, &parent_ref)?.id),
        None => None,
    };

    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let mut view = View::new(args.name.clone(), args.query, actor);
    view.update_parent(parent_id);
    validate_parent_assignment(&repo.metadata, Some(view.id), view.parent_id)?;
    repo.metadata.store_view(&view)?;
    println!(
        "Created view {} ({})",
        dynamic_view_path(&repo, &view)?,
        view.id
    );
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
    if views.is_empty() {
        println!("(none)");
        return Ok(());
    }

    let mut views_by_parent: HashMap<Option<ViewID>, Vec<View>> = HashMap::new();
    for view in views {
        views_by_parent
            .entry(view.parent_id)
            .or_default()
            .push(view);
    }
    for children in views_by_parent.values_mut() {
        children.sort_by(|a, b| {
            a.name
                .cmp(&b.name)
                .then_with(|| a.id.to_string().cmp(&b.id.to_string()))
        });
    }
    print_dynamic_tree(&repo, &views_by_parent, None, 0)?;
    Ok(())
}

async fn delete(repo: LatticeRepo, args: DeleteArgs) -> Result<()> {
    let view = resolve_dynamic_view(&repo, &args.reference)?;
    let view_path = dynamic_view_path(&repo, &view)?;
    let children = repo.metadata.list_children(Some(view.id))?;

    if !children.is_empty() && !args.cascade && !args.detach_children {
        return Err(anyhow::anyhow!(
            "View '{}' has children. Use --cascade or --detach-children.",
            view_path.as_str()
        ));
    }

    if args.cascade {
        let descendants = collect_descendants(&repo.metadata, view.id)?;
        for descendant in descendants {
            repo.metadata.delete_view_by_id(&descendant.id)?;
        }
    } else if args.detach_children {
        for mut child in children {
            child.update_parent(None);
            repo.metadata.store_view(&child)?;
        }
    }

    repo.metadata.delete_view_by_id(&view.id)?;
    println!("Deleted view {} ({})", view_path, view.id);
    Ok(())
}

async fn explain(repo: LatticeRepo, args: ExplainArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;

    let parsed: Query = if let Some(q) = args.query {
        parse(&q)?
    } else if let Some(view_name) = args.view {
        match resolve_view_reference(&repo, &view_name)? {
            ResolvedView::Builtin(builtin) => parse(builtin.query())?,
            ResolvedView::Dynamic(view) => resolve_effective_query(&repo.metadata, &view)?,
        }
    } else {
        // Default to built-in "Recent" view for convenience
        parse(BuiltinView::Recent.query())?
    };

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

fn print_dynamic_tree(
    repo: &LatticeRepo,
    views_by_parent: &HashMap<Option<ViewID>, Vec<View>>,
    parent_id: Option<ViewID>,
    depth: usize,
) -> Result<()> {
    let Some(children) = views_by_parent.get(&parent_id) else {
        return Ok(());
    };

    for view in children {
        let indent = "  ".repeat(depth);
        println!(
            "{}- {} (id: {}, path: {}): {}",
            indent,
            view.name,
            view.id,
            dynamic_view_path(repo, view)?,
            view.query
        );
        print_dynamic_tree(repo, views_by_parent, Some(view.id), depth + 1)?;
    }

    Ok(())
}
