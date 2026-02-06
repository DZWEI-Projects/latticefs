use anyhow::Result;
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::crypto::{Capability, Permission, PublicKey};
use latticefs_base::views::ViewSnapshot;

use super::common::{
    ResolvedView, ensure_identity, identity_actor, parse_duration, resolve_identity_password,
    resolve_object_id, resolve_view_reference,
};

#[derive(Args, Debug)]
pub struct ShareCommand {
    #[command(subcommand)]
    pub subcommand: Option<ShareSubcommand>,
    /// Object reference (omit when using snapshot subcommand)
    pub reference: Option<String>,
    /// Capability permission (read|write|comment|share|admin)
    #[arg(long, default_value = "read", global = true)]
    pub cap: String,
    /// Recipient public key (DID:key or hex)
    #[arg(long, global = true)]
    pub to: Option<String>,
    /// Expiration duration (e.g., 7d, 1h)
    #[arg(long, default_value = "7d", global = true)]
    pub expires: String,
    /// Optional password for key storage
    #[arg(long, global = true)]
    pub password: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ShareSubcommand {
    Snapshot(ShareSnapshotArgs),
}

#[derive(Args, Debug)]
pub struct ShareSnapshotArgs {
    /// View name or ID to snapshot
    pub view: String,
}

pub async fn run(repo: LatticeRepo, command: ShareCommand) -> Result<()> {
    let to = command
        .to
        .ok_or_else(|| anyhow::anyhow!("share requires --to argument"))?;
    if let Some(sub) = command.subcommand {
        match sub {
            ShareSubcommand::Snapshot(args) => {
                share_snapshot(
                    repo,
                    args,
                    command.cap,
                    to,
                    command.expires,
                    command.password,
                )
                .await
            }
        }
    } else {
        let reference = command
            .reference
            .ok_or_else(|| anyhow::anyhow!("share requires an object reference"))?;
        let args = ShareObjectArgs {
            reference,
            cap: command.cap,
            to,
            expires: command.expires,
            password: command.password,
        };
        share_object(repo, args).await
    }
}

#[derive(Debug)]
struct ShareObjectArgs {
    reference: String,
    cap: String,
    to: String,
    expires: String,
    password: Option<String>,
}

async fn share_object(repo: LatticeRepo, args: ShareObjectArgs) -> Result<()> {
    let permission: Permission = args.cap.parse()?;
    let expires = parse_duration(&args.expires)?;
    let password = resolve_identity_password(args.password);
    let identity = ensure_identity("default", password.as_deref())?;
    let audience = parse_public_key(&args.to)?;

    let object_id = resolve_object_id(&repo, &args.reference)?;
    let object = repo.metadata.load_object(&object_id)?;
    repo.authorize_object_permission(&object, Permission::Share, true)?;
    repo.authorize_object_permission(&object, permission, false)?;
    repo.enforce_rate_limit(1)?;
    let cap = Capability::create(&identity, &audience, &object_id, permission, expires)?;
    repo.metadata.store_capability(&cap)?;
    repo.events.emit_sync(latticefs_base::Event::share_issued(
        format!("latticefs:object:{}", object_id),
        cap.cid(),
        audience.did(),
        cap.expires_at(),
    ));

    println!("Shared object {}", object_id);
    println!("CID: {}", cap.cid());
    println!("UCAN: {}", cap.token);
    Ok(())
}

async fn share_snapshot(
    repo: LatticeRepo,
    args: ShareSnapshotArgs,
    cap: String,
    to: String,
    expires: String,
    password: Option<String>,
) -> Result<()> {
    let permission: Permission = cap.parse()?;
    let expires = parse_duration(&expires)?;
    let password = resolve_identity_password(password);
    let identity = ensure_identity("default", password.as_deref())?;
    let audience = parse_public_key(&to)?;

    let (snapshot, resource) = create_snapshot(&repo, &args.view, identity_actor(&identity))?;
    for object_id in snapshot.object_ids.iter() {
        let object = repo.metadata.load_object(object_id)?;
        repo.authorize_object_permission(&object, Permission::Share, true)?;
        repo.authorize_object_permission(&object, permission, false)?;
    }
    repo.enforce_rate_limit(1)?;
    repo.metadata.store_snapshot(&snapshot)?;

    let capability = Capability::create_for_resource(
        &identity,
        &audience,
        resource.clone(),
        permission,
        expires,
    )?;
    repo.metadata.store_capability(&capability)?;
    repo.events.emit_sync(latticefs_base::Event::share_issued(
        resource.clone(),
        capability.cid(),
        audience.did(),
        capability.expires_at(),
    ));
    println!("Snapshot shared. CID: {}", capability.cid());
    println!("UCAN: {}", capability.token);
    Ok(())
}

fn parse_public_key(input: &str) -> Result<PublicKey> {
    if input.starts_with("did:key:") {
        return Ok(PublicKey::from_did(input)?);
    }
    let bytes = hex::decode(input).map_err(|_| anyhow::anyhow!("Invalid public key format"))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;
    Ok(PublicKey::from_bytes(&arr)?)
}

fn create_snapshot(
    repo: &LatticeRepo,
    view_ref: &str,
    actor: [u8; 32],
) -> Result<(ViewSnapshot, String)> {
    match resolve_view_reference(repo, view_ref)? {
        ResolvedView::Builtin(builtin) => {
            let mut dynamic =
                latticefs_base::views::DynamicView::new(builtin.query(), &repo.metadata)?;
            let object_ids = dynamic.evaluate()?;
            let snapshot = ViewSnapshot::new(
                view_ref.to_string(),
                builtin.query().to_string(),
                object_ids,
                actor,
            );
            let resource = format!("latticefs:view:{}", view_ref);
            Ok((snapshot, resource))
        }
        ResolvedView::Dynamic(view) => {
            let mut dynamic = latticefs_base::views::DynamicView::new(&view.query, &repo.metadata)?;
            let object_ids = dynamic.evaluate()?;
            let snapshot = ViewSnapshot::from_view(&view, object_ids, actor);
            let resource = format!("latticefs:view:{}", view.name);
            Ok((snapshot, resource))
        }
    }
}
