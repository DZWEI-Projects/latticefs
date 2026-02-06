use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use latticefs_base::LatticeRepo;
use latticefs_base::ipc::client;
use latticefs_base::ipc::{MessageType, send_message};
use latticefs_base::watcher::{FileWatcher, PersistentRegistry, WatchRegistry};
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Args, Debug)]
pub struct WatchdArgs {
    #[command(subcommand)]
    pub command: WatchdCommand,
}

#[derive(Subcommand, Debug)]
pub enum WatchdCommand {
    /// Start the watcher daemon
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(long)]
        foreground: bool,
    },
    /// Stop the watcher daemon
    Stop,
    /// Show watcher daemon status and watched files
    Status,
}

pub async fn run(repo: LatticeRepo, args: WatchdArgs) -> Result<()> {
    match args.command {
        WatchdCommand::Start { foreground } => start(repo, foreground).await,
        WatchdCommand::Stop => stop(repo).await,
        WatchdCommand::Status => status(repo).await,
    }
}

async fn start(repo: LatticeRepo, foreground: bool) -> Result<()> {
    // Extract config and drop the repo immediately so the Sled file lock
    // is released. The daemon will open the database on demand for each
    // operation, allowing CLI commands to access the repo in between.
    let config = repo.config.clone();
    let pid_path = config.watchd_pid_path();
    let socket_path = config.socket_path();
    drop(repo);

    // Check if already running
    if pid_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process is actually running
                if is_process_running(pid) {
                    anyhow::bail!("Watcher daemon is already running (pid {})", pid);
                }
                // Stale PID file, clean up
                let _ = std::fs::remove_file(&pid_path);
            }
        }
    }

    if !foreground {
        // Spawn a detached child process with --foreground
        let exe = std::env::current_exe().context("Failed to get current executable")?;
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("watchd").arg("start").arg("--foreground");

        // Pass repo path if LATTICE_HOME is set
        if let Ok(home) = std::env::var("LATTICE_HOME") {
            cmd.env("LATTICE_HOME", home);
        }

        // Detach the child
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let child = cmd.spawn().context("Failed to start watcher daemon")?;
        println!("Watcher daemon started (pid {})", child.id());
        return Ok(());
    }

    // Foreground mode: write PID file and run
    let pid = std::process::id();
    std::fs::write(&pid_path, pid.to_string())?;

    // Load or create persistent registry
    let registry_path = config.watch_registry_path();
    let persist = PersistentRegistry::new(registry_path);
    let registry = persist.load().unwrap_or_else(|_| WatchRegistry::new());
    let registry_arc = Arc::new(registry.clone());

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Start IPC server in a separate task (uses Config, opens DB on demand)
    let ipc_config = config.clone();
    let ipc_registry = registry_arc.clone();
    let ipc_shutdown_tx = shutdown_tx.clone();
    let ipc_handle = tokio::spawn(async move {
        if let Err(e) =
            latticefs_base::ipc::server::run_ipc_server_with_watcher(ipc_config, Some(ipc_registry))
                .await
        {
            eprintln!("IPC server error: {}", e);
        }
        // If IPC server exits (e.g., shutdown request), signal the watcher too
        let _ = ipc_shutdown_tx.send(true);
    });

    eprintln!("Watcher daemon started (pid {})", pid);
    eprintln!("Socket: {}", socket_path.display());
    eprintln!("Watch dir: {}", config.watcher.watch_dir);
    eprintln!("Press Ctrl+C to stop\n");

    // Start the file watcher (uses Config, opens DB on demand per file change)
    let mut watcher = FileWatcher::new(registry, persist, config.clone(), shutdown_rx);

    // Handle Ctrl+C
    let ctrl_c_shutdown = shutdown_tx.clone();
    let ctrl_c_pid_path = pid_path.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        eprintln!("\nShutting down watcher daemon...");
        let _ = ctrl_c_shutdown.send(true);
        let _ = std::fs::remove_file(&ctrl_c_pid_path);
    });

    watcher.run().await.ok();

    // Clean up PID file
    let _ = std::fs::remove_file(&pid_path);
    // Clean up socket
    let sock = config.socket_path();
    let _ = std::fs::remove_file(&sock);

    // Wait for IPC to finish
    ipc_handle.abort();

    eprintln!("Watcher daemon stopped");
    Ok(())
}

async fn stop(repo: LatticeRepo) -> Result<()> {
    let socket_path = repo.config.socket_path();

    if !client::is_daemon_running(&socket_path) {
        anyhow::bail!("Watcher daemon is not running");
    }

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .context("Failed to connect to daemon")?;

    let request = latticefs_base::ipc::proto::ShutdownRequest {
        force: false,
        timeout_seconds: 5,
    };
    send_message(&mut stream, MessageType::ShutdownRequest, &request).await?;

    println!("Shutdown request sent to watcher daemon");

    // Clean up PID file
    let pid_path = repo.config.watchd_pid_path();
    let _ = std::fs::remove_file(&pid_path);

    Ok(())
}

async fn status(repo: LatticeRepo) -> Result<()> {
    let socket_path = repo.config.socket_path();

    if !client::is_daemon_running(&socket_path) {
        println!("Watcher daemon: not running");
        return Ok(());
    }

    let status = client::send_watch_status(&socket_path)
        .await
        .context("Failed to get watcher status")?;

    println!("Watcher daemon: running (pid {})", status.pid);
    println!("Watch directory: {}", status.watch_dir);
    println!("Watched files: {}", status.watched_count);

    if status.watched_count > 0 {
        let files = client::send_watch_list(&socket_path)
            .await
            .context("Failed to list watched files")?;

        println!();
        println!("{:<38} {:<30} {}", "OBJECT ID", "NAME", "PATH");
        println!("{}", "-".repeat(100));
        for file in files {
            println!(
                "{:<38} {:<30} {}",
                file.object_id, file.display_name, file.temp_path,
            );
        }
    }

    Ok(())
}

fn is_process_running(pid: u32) -> bool {
    // On Unix, send signal 0 to check if process exists
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        args: WatchdArgs,
    }

    #[test]
    fn parses_start_foreground_flag() {
        let cli = Cli::parse_from(["watchd", "start", "--foreground"]);
        match cli.args.command {
            WatchdCommand::Start { foreground } => assert!(foreground),
            _ => panic!("expected start subcommand"),
        }
    }

    #[test]
    fn parses_status_subcommand() {
        let cli = Cli::parse_from(["watchd", "status"]);
        assert!(matches!(cli.args.command, WatchdCommand::Status));
    }

    #[cfg(unix)]
    #[test]
    fn current_process_is_reported_as_running() {
        let pid = std::process::id();
        assert!(is_process_running(pid));
    }
}
