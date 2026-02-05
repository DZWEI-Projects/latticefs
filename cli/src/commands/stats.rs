use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use clap::{Args, Subcommand};
use latticefs_base::crypto::Capability;
use latticefs_base::model::Tag;
use latticefs_base::storage::hash_to_hex;
use latticefs_base::views::{BuiltinView, BuiltinViews, DynamicView};
use latticefs_base::{LatticeRepo, Permission};

use super::common::{
    parse_ref_with_version,
    resolve_object_id,
    resolve_view_reference,
    ResolvedView,
};

#[derive(Subcommand, Debug)]
pub enum StatsCommand {
    /// Show content checksum for an object (alias: hash)
    #[command(alias = "hash")]
    Checksum(StatsChecksumArgs),
    /// Show object statistics
    Object(StatsObjectArgs),
    /// Show statistics for a single view
    View(StatsViewArgs),
    /// List objects for a view with minimal tag output
    ViewObjects(StatsViewObjectsArgs),
    /// Summarize all views
    Views,
    /// Show statistics for a single policy
    Policy(StatsPolicyArgs),
    /// Summarize all policies
    Policies,
    /// Summarize shared capabilities
    Shares,
}

#[derive(Args, Debug)]
pub struct StatsArgs {
    #[command(subcommand)]
    pub command: StatsCommand,
}

#[derive(Args, Debug)]
pub struct StatsChecksumArgs {
    /// Object reference (optionally @version)
    pub reference: String,
}

#[derive(Args, Debug)]
pub struct StatsObjectArgs {
    /// Object reference
    pub reference: String,
    /// Include per-version stats
    #[arg(long)]
    pub all_versions: bool,
}

#[derive(Args, Debug)]
pub struct StatsViewArgs {
    /// View name or ID (builtin or dynamic)
    pub name: String,
}

#[derive(Args, Debug)]
pub struct StatsViewObjectsArgs {
    /// View name or ID (builtin or dynamic)
    pub name: String,
    /// Include auto/system tags
    #[arg(long)]
    pub all_tags: bool,
    /// Show encoded tag values alongside decoded values
    #[arg(long)]
    pub raw_tags: bool,
}

#[derive(Args, Debug)]
pub struct StatsPolicyArgs {
    /// Policy name
    pub name: String,
}

pub async fn run(repo: LatticeRepo, cmd: StatsCommand) -> Result<()> {
    match cmd {
        StatsCommand::Checksum(args) => checksum(repo, args).await,
        StatsCommand::Object(args) => object_stats(repo, args).await,
        StatsCommand::View(args) => view_stats(repo, args).await,
        StatsCommand::ViewObjects(args) => view_objects(repo, args).await,
        StatsCommand::Views => views_summary(repo).await,
        StatsCommand::Policy(args) => policy_stats(repo, args).await,
        StatsCommand::Policies => policies_summary(repo).await,
        StatsCommand::Shares => shares_summary(repo).await,
    }
}

async fn checksum(repo: LatticeRepo, args: StatsChecksumArgs) -> Result<()> {
    let (object_id, version_id) = parse_ref_with_version(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Read, false)?;

    let version_id = version_id.unwrap_or(object.current_version);
    let version = repo.metadata.load_version(&version_id)?;
    if version.object_id != object_id {
        return Err(anyhow::anyhow!("Version does not belong to object"));
    }

    println!("Object: {}", object_id);
    println!("Version: {}", version_id);
    println!("Algorithm: BLAKE3 (chunk merkle root)");
    println!("Chunk root: {}", hash_to_hex(&version.chunk_root));
    println!("Manifest: {}", hash_to_hex(&version.manifest_ref));
    println!("Size bytes: {}", version.size_bytes);
    println!("Chunks: {}", version.chunk_count);
    Ok(())
}

async fn object_stats(repo: LatticeRepo, args: StatsObjectArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Read, false)?;

    let current = repo.metadata.load_version(&object.current_version)?;

    println!("Object: {}", object.id);
    println!("Type: {}", object.object_type);
    println!("Created at: {}", object.created_at);
    println!("Created by: {}", hex::encode(object.created_by));
    println!("Metadata partition: {:?}", object.metadata_partition);
    println!("Tags: {}", object.tags.len());
    println!("Links: {}", object.links.len());
    println!("Policies: {}", object.policy_refs.len());
    println!("Versions: {}", object.versions.len());
    println!("Current version: {}", object.current_version);
    println!("Current size bytes: {}", current.size_bytes);
    println!("Current chunks: {}", current.chunk_count);
    println!("Current state: {}", current.state);

    if args.all_versions {
        let mut versions = Vec::new();
        for vid in &object.versions {
            versions.push(repo.metadata.load_version(vid)?);
        }
        versions.sort_by_key(|v| v.created_at);

        let total_bytes: u64 = versions.iter().map(|v| v.size_bytes).sum();
        let total_chunks: u64 = versions.iter().map(|v| v.chunk_count as u64).sum();
        println!("Total version bytes: {}", total_bytes);
        println!("Total version chunks: {}", total_chunks);
        println!("All versions:");
        for (idx, v) in versions.iter().enumerate() {
            println!(
                "- v{} {} size={} chunks={} state={} created_at={}",
                idx + 1,
                v.id,
                v.size_bytes,
                v.chunk_count,
                v.state,
                v.created_at
            );
        }
    }

    Ok(())
}

async fn view_stats(repo: LatticeRepo, args: StatsViewArgs) -> Result<()> {
    match resolve_view_reference(&repo, &args.name)? {
        ResolvedView::Builtin(builtin) => {
            let builtins = BuiltinViews::new(&repo.metadata);
            let object_ids = builtins.evaluate(builtin)?;
            let readable_count = count_readable(&repo, &object_ids);
            println!("View: {}", builtin.name());
            println!("Type: builtin");
            println!("Query: {}", builtin.query());
            println!("Description: {}", builtin.description());
            println!("Objects: {}", readable_count);
        }
        ResolvedView::Dynamic(view) => {
            let mut dynamic =
                DynamicView::new(&view.query, &repo.metadata)?.with_config(view.config.clone());
            let object_ids = dynamic.evaluate()?;
            let readable_count = count_readable(&repo, &object_ids);

            println!("View: {}", view.name);
            println!("Id: {}", view.id);
            println!("Type: dynamic");
            println!("Query: {}", view.query);
            if let Some(description) = &view.description {
                println!("Description: {}", description);
            }
            println!("Created at: {}", view.created_at);
            println!("Modified at: {}", view.modified_at);
            println!("Created by: {}", hex::encode(view.created_by));
            println!("Config max results: {:?}", view.config.max_results);
            println!("Config cache ttl secs: {}", view.config.cache_ttl_secs);
            println!("Config include archived: {}", view.config.include_archived);
            println!("Config min trust: {:?}", view.config.min_trust_level);
            println!("Objects: {}", readable_count);
        }
    }
    Ok(())
}

async fn view_objects(repo: LatticeRepo, args: StatsViewObjectsArgs) -> Result<()> {
    let object_ids = match resolve_view_reference(&repo, &args.name)? {
        ResolvedView::Builtin(builtin) => {
            let builtins = BuiltinViews::new(&repo.metadata);
            builtins.evaluate(builtin)?
        }
        ResolvedView::Dynamic(view) => {
            let mut dynamic =
                DynamicView::new(&view.query, &repo.metadata)?.with_config(view.config.clone());
            dynamic.evaluate()?
        }
    };

    if object_ids.is_empty() {
        println!("No objects");
        return Ok(());
    }

    for object_id in object_ids {
        let object = repo
            .metadata
            .load_object(&object_id)
            .with_context(|| format!("Object not found: {}", object_id))?;
        repo.authorize_object_permission(&object, Permission::Read, false)?;
        let mut tags: Vec<_> = if args.all_tags {
            object.tags
        } else {
            object
                .tags
                .into_iter()
                .filter(|t| !t.is_auto_generated() && !t.is_system())
                .collect()
        };
        tags.sort_by_key(|t| t.full_path());

        println!("{}", object.id);
        if tags.is_empty() {
            println!("  (no tags)");
        } else {
            for tag in tags {
                println!("  {}", format_tag(&tag, args.raw_tags));
            }
        }
    }

    Ok(())
}

async fn views_summary(repo: LatticeRepo) -> Result<()> {
    let builtins = BuiltinViews::new(&repo.metadata);
    println!("Built-in views:");
    for builtin in BuiltinView::all() {
        let object_ids = builtins.evaluate(*builtin)?;
        let readable_count = count_readable(&repo, &object_ids);
        println!("- {}: {} objects", builtin.name(), readable_count);
    }

    let views = repo.metadata.list_views()?;
    println!("\nDynamic views: {}", views.len());
    for view in views {
        let mut dynamic =
            DynamicView::new(&view.query, &repo.metadata)?.with_config(view.config.clone());
        let object_ids = dynamic.evaluate()?;
        let readable_count = count_readable(&repo, &object_ids);
        println!("- {} (id: {}): {} objects", view.name, view.id, readable_count);
    }

    let snapshots = repo.metadata.list_snapshots()?;
    println!("\nSnapshots: {}", snapshots.len());
    Ok(())
}

async fn policy_stats(repo: LatticeRepo, args: StatsPolicyArgs) -> Result<()> {
    let policy = repo.metadata.load_policy(&args.name)?;
    let mut object_count = 0usize;
    for object in repo.metadata.iter_objects() {
        let object = object?;
        if object.policy_refs.contains(&policy.id) {
            if repo
                .authorize_object_permission(&object, Permission::Read, false)
                .is_ok()
            {
                object_count += 1;
            }
        }
    }

    println!("Policy: {}", policy.name);
    println!("Id: {}", policy.id);
    println!("Version: {}", policy.version);
    println!(
        "Allow: {}",
        policy
            .allow
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "Deny: {}",
        policy
            .deny
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if policy.require.is_empty() {
        println!("Requirements: (none)");
    } else {
        println!("Requirements:");
        for req in &policy.require {
            println!("- {:?}", req);
        }
    }
    println!("Retain days: {:?}", policy.retain_days);
    println!("External share: {}", policy.external_share);
    println!("Objects using policy: {}", object_count);
    Ok(())
}

async fn policies_summary(repo: LatticeRepo) -> Result<()> {
    let policies = repo.metadata.list_policies()?;
    println!("Policies: {}", policies.len());

    let mut external_share = 0usize;
    let mut with_retention = 0usize;
    for policy in &policies {
        if policy.external_share {
            external_share += 1;
        }
        if policy.retain_days.is_some() {
            with_retention += 1;
        }
    }

    println!("External share enabled: {}", external_share);
    println!("Retention policies: {}", with_retention);
    for policy in policies {
        println!("- {}", policy.name);
    }
    Ok(())
}

async fn shares_summary(repo: LatticeRepo) -> Result<()> {
    let caps = repo.metadata.list_capabilities()?;
    if caps.is_empty() {
        println!("No shares");
        return Ok(());
    }

    let mut total = 0usize;
    let mut expired = 0usize;
    let mut object_caps = 0usize;
    let mut view_caps = 0usize;
    let mut other_caps = 0usize;
    let mut perm_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (_cid, token) in caps {
        total += 1;
        if let Ok(cap) = Capability::parse(&token) {
            if cap.is_expired() {
                expired += 1;
            }
            if let Some(sub) = &cap.payload.sub {
                if sub.starts_with("latticefs:object:") {
                    object_caps += 1;
                } else if sub.starts_with("latticefs:view:") {
                    view_caps += 1;
                } else {
                    other_caps += 1;
                }
            } else {
                other_caps += 1;
            }
            for att in &cap.payload.att {
                let key = format!("{}", att.can);
                *perm_counts.entry(key).or_insert(0) += 1;
            }
        }
    }

    println!("Shares: {}", total);
    println!("Expired: {}", expired);
    println!("Active: {}", total - expired);
    println!("Object shares: {}", object_caps);
    println!("View shares: {}", view_caps);
    println!("Other shares: {}", other_caps);
    if !perm_counts.is_empty() {
        println!("Permissions:");
        let mut perms: Vec<_> = perm_counts.into_iter().collect();
        perms.sort_by_key(|(k, _)| k.clone());
        for (perm, count) in perms {
            println!("- {}: {}", perm, count);
        }
    }
    Ok(())
}

fn format_tag(tag: &Tag, raw_tags: bool) -> String {
    if let Some(decoded) = decode_b64_tag(tag) {
        if raw_tags {
            format!("{}:{} (decoded: {})", tag.key, tag.value, decoded)
        } else {
            format!("{}:{}", tag.key, decoded)
        }
    } else {
        tag.full_path()
    }
}

fn decode_b64_tag(tag: &Tag) -> Option<String> {
    if !tag.key.ends_with("_b64") {
        return None;
    }
    let decoded = URL_SAFE_NO_PAD.decode(tag.value.as_bytes()).ok()?;
    String::from_utf8(decoded).ok()
}

/// Count objects that the current actor has read permission for.
fn count_readable(repo: &LatticeRepo, object_ids: &[latticefs_base::model::ObjectID]) -> usize {
    object_ids
        .iter()
        .filter(|id| {
            repo.metadata
                .load_object(id)
                .ok()
                .map(|obj| {
                    repo.authorize_object_permission(&obj, Permission::Read, false)
                        .is_ok()
                })
                .unwrap_or(false)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    #[test]
    fn test_format_tag_decodes_b64() {
        let encoded = URL_SAFE_NO_PAD.encode("Report Final (v1).txt".as_bytes());
        let tag = Tag::new("auto:filename_b64".to_string(), encoded, test_actor());
        assert_eq!(
            format_tag(&tag, false),
            "auto:filename_b64:Report Final (v1).txt"
        );
    }

    #[test]
    fn test_format_tag_raw_includes_encoded() {
        let encoded = URL_SAFE_NO_PAD.encode("Report Final (v1).txt".as_bytes());
        let tag = Tag::new(
            "auto:filename_b64".to_string(),
            encoded.clone(),
            test_actor(),
        );
        assert_eq!(
            format_tag(&tag, true),
            format!(
                "auto:filename_b64:{} (decoded: Report Final (v1).txt)",
                encoded
            )
        );
    }

    #[test]
    fn test_format_tag_non_b64() {
        let tag = Tag::new("project".to_string(), "phoenix".to_string(), test_actor());
        assert_eq!(format_tag(&tag, false), "project:phoenix");
    }
}
