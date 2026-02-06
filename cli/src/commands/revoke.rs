use anyhow::Result;
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::crypto::{Capability, Revocation};

use super::common::{ensure_identity, resolve_identity_password};

#[derive(Args, Debug)]
pub struct RevokeArgs {
    /// Capability token or CID
    pub capability: String,
    /// Optional reason
    #[arg(long)]
    pub reason: Option<String>,
    /// Optional password for key storage
    #[arg(long)]
    pub password: Option<String>,
}

pub async fn run(repo: LatticeRepo, args: RevokeArgs) -> Result<()> {
    repo.enforce_rate_limit(1)?;
    let cap = if args.capability.contains('.') {
        Capability::parse(&args.capability)?
    } else {
        repo.metadata.load_capability(&args.capability)?
    };

    let password = resolve_identity_password(args.password);
    let identity = ensure_identity("default", password.as_deref())?;

    let revocation = Revocation::new(cap.cid(), &identity, args.reason)?;
    repo.metadata.store_revocation(&revocation)?;

    println!("Revoked capability {}", revocation.ucan_cid);
    Ok(())
}
