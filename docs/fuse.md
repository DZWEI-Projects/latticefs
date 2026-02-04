# FUSE Support (Read-Only)

## What it is
FUSE (Filesystem in Userspace) lets LatticeFS expose a virtual filesystem tree without kernel-level code. The kernel forwards file operations (open/read/readdir) to the LatticeFS process.

## Read-only meaning
The mount is **read-only**. You can browse and open objects, but you **cannot write or save** through the mount. All writes must go through the CLI.

**Pros**
- Safe: no app can corrupt your data store via filesystem writes.
- Predictable: object immutability is preserved.
- Compatible: tools can browse objects as files.

**Cons**
- You can’t save edits into the mount.
- Apps expecting write access must use `lfs export`/`lfs import` instead.

## Runtime flag requirement
Mounting requires the `--fuse` flag:
```bash
lfs --fuse mount ~/Lattice
```
If `--fuse` is omitted, `lfs mount` exits with a clear error.

## Build-time feature
FUSE support is compiled behind a feature:
```bash
cargo build -p cli --features latticefs-base/fuse
```
If you build without this feature, mount operations fail and instruct you to rebuild with the feature.

## macOS (macFUSE)
1. Install:
   ```bash
   brew install --cask macfuse
   ```
2. Approve the system extension if prompted:
   **System Settings → Privacy & Security → Allow**
3. Reboot if required.

## Linux (libfuse)
Install libfuse (package name varies):
```bash
# Debian/Ubuntu
sudo apt-get install libfuse2 libfuse-dev

# Fedora
sudo dnf install fuse fuse-libs fuse-devel
```

## Troubleshooting
- `mount_macfuse: the file system is not available (1)`
  - macFUSE is installed but not approved/loaded. Approve in Privacy & Security and reboot if required.

- `FUSE disabled. Re-run with --fuse ...`
  - You tried to mount without the runtime flag.

- `FUSE support not enabled. Rebuild with --features fuse ...`
  - The binary was compiled without the fuse feature.
