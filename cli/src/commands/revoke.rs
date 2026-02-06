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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        args: RevokeArgs,
    }

    #[test]
    fn parses_revoke_with_optional_fields() {
        let cli = Cli::parse_from([
            "revoke",
            "cid123",
            "--reason",
            "rotated-keys",
            "--password",
            "secret",
        ]);

        assert_eq!(cli.args.capability, "cid123");
        assert_eq!(cli.args.reason.as_deref(), Some("rotated-keys"));
        assert_eq!(cli.args.password.as_deref(), Some("secret"));
    }

    #[test]
    fn revoke_requires_capability() {
        let err = Cli::try_parse_from(["revoke"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }
}
