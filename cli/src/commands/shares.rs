use anyhow::Result;
use clap::{Args, Subcommand};
use latticefs_base::crypto::Capability;
use latticefs_base::LatticeRepo;

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

    for (cid, token) in caps {
        let parsed = Capability::parse(&token);
        match parsed {
            Ok(cap) => {
                let subject = cap.payload.sub.clone().unwrap_or_else(|| "(none)".to_string());
                let perms: Vec<String> = cap.payload.att.iter().map(|a| format!("{}:{}", a.with, a.can)).collect();
                println!("{}", cid);
                println!("  subject: {}", subject);
                println!("  perms: {}", perms.join(", "));
            }
            Err(_) => {
                println!("{}", cid);
                println!("  token: {}", token);
            }
        }
    }

    Ok(())
}
