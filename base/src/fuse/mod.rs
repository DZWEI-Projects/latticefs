//! FUSE module for NeuralFS.

pub mod inode;

#[cfg(feature = "fuse")]
pub mod mount;
#[cfg(feature = "fuse")]
pub mod readonly;

pub use inode::{
    inode_for_view_name, InodeMapper, PROJECTS_INODE, RECENT_INODE, ROOT_INODE, VIEWS_INODE,
};

#[cfg(feature = "fuse")]
pub use mount::mount_fs;
#[cfg(feature = "fuse")]
pub use readonly::NeuralFS;

#[cfg(not(feature = "fuse"))]
pub fn mount_fs(
    _repo: crate::repo::LatticeRepo,
    _mount_point: &std::path::Path,
    _config: &crate::config::FuseConfig,
) -> crate::error::Result<()> {
    Err(crate::error::LatticeError::Io(std::io::Error::other(
        "FUSE support not enabled. Rebuild with --features fuse and install libfuse.",
    )))
}
