use anyhow::Result;
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::crypto::Capability;

#[derive(Subcommand, Debug)]
pub enum SharesCommand {
    List,
}

#[derive(Args, Debug)]
pub struct SharesArgs {
    #[command(subcommand)]
    pub command: SharesCommand,
}

pub async fn run(repo: LatticeRepo, cmd: SharesCommand) -> Result<()> {
    match cmd {
        SharesCommand::List => list(repo).await,
    }
}

async fn list(repo: LatticeRepo) -> Result<()> {
    let caps = repo.metadata.list_capabilities()?;
    if caps.is_empty() {
        println!("No shares");
        return Ok(());
    }

    let mut first = true;
    for (cid, token) in caps {
        if !first {
            println!();
        }
        first = false;

        let parsed = Capability::parse(&token);
        match parsed {
            Ok(cap) => {
                let subject = cap
                    .payload
                    .sub
                    .clone()
                    .unwrap_or_else(|| "(none)".to_string());
                let perms: Vec<String> = cap
                    .payload
                    .att
                    .iter()
                    .map(|a| format!("{}:{}", a.with, a.can))
                    .collect();

                // Format expiration time
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let exp_timestamp = cap.payload.exp;
                let is_expired = cap.is_expired();
                let exp_status = if is_expired {
                    "expired".to_string()
                } else {
                    let remaining = exp_timestamp.saturating_sub(now);
                    let days = remaining / 86400;
                    let hours = (remaining % 86400) / 3600;
                    if days > 0 {
                        format!("expires in {}d {}h", days, hours)
                    } else if hours > 0 {
                        format!("expires in {}h", hours)
                    } else {
                        format!("expires in {}s", remaining)
                    }
                };

                println!("Share ID: {}", cid);
                println!("  issuer:   {}", cap.payload.iss);
                println!("  audience: {}", cap.payload.aud);
                println!("  subject:  {}", subject);
                println!("  perms:    {}", perms.join(", "));
                println!("  status:   {}", exp_status);
            }
            Err(_) => {
                println!("Share ID: {}", cid);
                println!("  token: {}", token);
                println!("  status: invalid (failed to parse capability)");
            }
        }
    }

    Ok(())
}
