//! FUSE mount utilities.

use crate::config::FuseConfig;
use crate::error::{LatticeError, Result};
use crate::fuse::readonly::LatticeFS;
use crate::repo::LatticeRepo;
use fuser::MountOption;
use std::path::Path;

/// Mount the LatticeFS FUSE filesystem (read-only MVP).
pub fn mount_fs(repo: LatticeRepo, mount_point: &Path, config: &FuseConfig) -> Result<()> {
    std::fs::create_dir_all(mount_point)?;

    let fs = LatticeFS::new(repo);
    let mut options = vec![
        MountOption::RO,
        MountOption::FSName("latticefs".to_string()),
        MountOption::DefaultPermissions,
    ];

    if config.allow_other {
        options.push(MountOption::AllowOther);
    }

    fuser::mount2(fs, mount_point, &options)
        .map_err(|e| LatticeError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(())
}
