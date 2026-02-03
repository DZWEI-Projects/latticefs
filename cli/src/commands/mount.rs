use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::mount_fs;
use latticefs_base::LatticeRepo;
use std::path::PathBuf;

use super::common::expand_path;

#[derive(Args, Debug)]
pub struct MountArgs {
    /// Mount point (default from config)
    pub mount_point: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct UnmountArgs {
    /// Mount point (default from config)
    pub mount_point: Option<PathBuf>,
}

pub async fn run_mount(repo: LatticeRepo, args: MountArgs) -> Result<()> {
    let mount_point = args
        .mount_point
        .as_ref()
        .map(|p| expand_path(p))
        .unwrap_or_else(|| repo.config.mount_point());
    let fuse_config = repo.config.fuse.clone();

    mount_fs(repo, &mount_point, &fuse_config)
        .with_context(|| format!("Failed to mount at {}", mount_point.display()))?;
    Ok(())
}

pub async fn run_unmount(repo: LatticeRepo, args: UnmountArgs) -> Result<()> {
    let mount_point = args
        .mount_point
        .as_ref()
        .map(|p| expand_path(p))
        .unwrap_or_else(|| repo.config.mount_point());

    unmount_path(&mount_point)
        .with_context(|| format!("Failed to unmount {}", mount_point.display()))?;
    Ok(())
}

fn unmount_path(path: &PathBuf) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("umount")
            .arg(path)
            .status()
            .context("umount failed")?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("fusermount")
            .arg("-u")
            .arg(path)
            .status()
            .context("fusermount failed")?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        return Err(anyhow::anyhow!("Unmount not supported on this platform"));
    }

    Ok(())
}
