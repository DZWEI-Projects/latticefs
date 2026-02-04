use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod commands;

#[derive(Parser, Debug)]
#[command(name = "lfs", version, about = "LatticeFS CLI")]
struct Cli {
    /// Increase verbosity (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
    /// Enable FUSE operations (required for mount)
    #[arg(long, global = true)]
    fuse: bool,
    /// Override repository path
    #[arg(long, global = true)]
    repo: Option<PathBuf>,
    #[command(subcommand)]
    command: commands::Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        commands::Command::Init(args) => commands::system::init(args).await?,
        commands::Command::Status(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::system::status(repo, args).await?;
        }
        commands::Command::Gc(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::system::gc(repo, args).await?;
        }
        commands::Command::Verify(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::system::verify(repo, args).await?;
        }
        commands::Command::Add(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::add::run(repo, args).await?;
        }
        commands::Command::Import(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::import::run(repo, args).await?;
        }
        commands::Command::Export(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::export::run(repo, args).await?;
        }
        commands::Command::Tag(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::tag::run(repo, args).await?;
        }
        commands::Command::Tags(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::tags::run(repo, args).await?;
        }
        commands::Command::Untag(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::untag::run(repo, args).await?;
        }
        commands::Command::Link(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::link::run(repo, args).await?;
        }
        commands::Command::Meta(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::meta::run(repo, args).await?;
        }
        commands::Command::Get(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::get::run(repo, args).await?;
        }
        commands::Command::Cat(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::cat::run(repo, args).await?;
        }
        commands::Command::Versions(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::versions::run(repo, args).await?;
        }
        commands::Command::Restore(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::versions::restore(repo, args).await?;
        }
        commands::Command::Checkout(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::checkout::run(repo, args).await?;
        }
        commands::Command::Revise(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::revise::run(repo, args).await?;
        }
        commands::Command::Diff(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::diff::run(repo, args).await?;
        }
        commands::Command::State(cmd) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::state::run(repo, cmd.command).await?;
        }
        commands::Command::View(cmd) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::view::run(repo, cmd.command).await?;
        }
        commands::Command::Share(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::share::run(repo, args).await?;
        }
        commands::Command::Revoke(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::revoke::run(repo, args).await?;
        }
        commands::Command::Shares(cmd) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::shares::run(repo, cmd.command).await?;
        }
        commands::Command::Policy(cmd) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::policy::run(repo, cmd.command).await?;
        }
        commands::Command::Trust(cmd) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::trust::run(repo, cmd.command).await?;
        }
        commands::Command::Quarantine(cmd) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::trust::quarantine(repo, cmd.command).await?;
        }
        commands::Command::Mount(args) => {
            if !cli.fuse {
                return Err(anyhow::anyhow!(
                    "FUSE disabled. Re-run with --fuse to mount, and ensure macFUSE/libfuse is installed."
                ));
            }
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::mount::run_mount(repo, args).await?;
        }
        commands::Command::Unmount(args) => {
            let repo = commands::common::open_repo(cli.repo.clone())?;
            commands::mount::run_unmount(repo, args).await?;
        }
    }

    Ok(())
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt().with_max_level(level).init();
}
