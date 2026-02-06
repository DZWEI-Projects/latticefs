# File Watcher

LatticeFS includes a file watcher daemon that monitors exported files for changes and automatically creates new versions when files are saved in an external editor.

## Overview

When you open an object for editing (`lfs edit <ref>`), LatticeFS:

1. Exports the object content to a temporary file in the watch directory
2. Registers the file with the watcher daemon for monitoring
3. Opens the file in your system's default editor

The watcher daemon detects saves and automatically creates new versions using `add_version_from_bytes()`, which handles all state transitions (draft -> discarded, review -> approved).

## Quick Start

```bash
# Start the watcher daemon
lfs watchd start

# Open an object for editing
lfs edit <object-id-or-alias>

# Edit and save the file in your editor
# New versions are created automatically

# Check status
lfs watchd status

# Stop the daemon
lfs watchd stop
```

## Architecture

The watcher lives in the **base library** so both the CLI and GUI can use it:

- `base/src/watcher/registry.rs` -- Thread-safe map of watched files to object metadata
- `base/src/watcher/persist.rs` -- JSON-based crash recovery (at `~/.latticefs/watcher_registry.json`)
- `base/src/watcher/daemon.rs` -- Core daemon using the `notify` crate with debouncing

Communication uses the existing IPC protocol (Unix domain socket) with message types in the 400-range.

## Change Detection

The watcher uses a two-level deduplication strategy:

1. **OS-level debouncing** via `notify-debouncer-full` -- Coalesces multiple filesystem events from a single save operation (temp-file-rename patterns, multiple writes)
2. **Content hashing** via BLAKE3 -- Compares the hash of new content against the last committed hash, skipping no-op saves

## Configuration

Add a `[watcher]` section to `~/.latticefs/config.toml`:

```toml
[watcher]
enabled = true
debounce_ms = 1000
commit_message_template = "Auto-saved from external editor at {timestamp}"
watch_dir = "/tmp/latticefs-open"
ignored_patterns = ["*.swp", "*.swo", "*~", ".DS_Store", "*.tmp", "*.bak", ".~lock.*"]
```

### Options

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `true` | Enable/disable the watcher |
| `debounce_ms` | `1000` | Debounce interval in milliseconds |
| `commit_message_template` | `"Auto-saved from external editor at {timestamp}"` | Template for auto-commit messages. Supports `{timestamp}`, `{filename}`, `{object_id}` |
| `watch_dir` | `/tmp/latticefs-open` | Directory where exported files are placed |
| `ignored_patterns` | `["*.swp", "*.swo", "*~", ".DS_Store", "*.tmp", "*.bak", ".~lock.*"]` | Glob patterns to ignore |

## CLI Commands

### `lfs watchd start`

Start the watcher daemon. By default, it daemonizes (runs in the background).

```bash
lfs watchd start              # Daemonize
lfs watchd start --foreground # Run in foreground (for debugging)
```

The daemon writes a PID file at `~/.latticefs/watchd.pid` and creates an IPC socket at `~/.latticefs/latticefs.sock`.

### `lfs watchd stop`

Send a shutdown request to the running daemon.

```bash
lfs watchd stop
```

### `lfs watchd status`

Show daemon status and a table of currently watched files.

```bash
lfs watchd status
```

Example output:
```
Watcher daemon: running (pid 12345)
Watch directory: /tmp/latticefs-open
Watched files: 2

OBJECT ID                              NAME                           PATH
a1b2c3d4-...                           report.pdf                     /tmp/latticefs-open/a1b2c3d4-..._report.pdf
e5f6a7b8-...                           notes.md                       /tmp/latticefs-open/e5f6a7b8-..._notes.md
```

### `lfs edit <reference>`

Export an object and open it for editing with auto-versioning.

```bash
lfs edit <uuid>                    # By object ID
lfs edit <alias>                   # By alias
lfs edit <ref> --no-watch          # Export only, skip watcher registration
lfs edit <ref> -m "custom message" # Custom commit message
```

## GUI Integration

The GUI's "Open" action (`open_object`) automatically registers files with the watcher daemon when available. The GUI also provides:

- `get_watcher_status` -- Returns daemon status (running/not, watched count, watch dir)
- `list_watched_files` -- Returns list of watched files with object IDs and names

## Troubleshooting

### Daemon won't start

- Check if another instance is running: `lfs watchd status`
- Remove stale PID file: `rm ~/.latticefs/watchd.pid`
- Remove stale socket: `rm ~/.latticefs/latticefs.sock`

### Changes not detected

- Verify the daemon is running: `lfs watchd status`
- Check that the file is registered (shown in status output)
- Some editors use atomic saves (write to temp, rename) which may need a longer debounce
- Increase `debounce_ms` in config if changes are missed

### Object sealed error

If an object is sealed, the watcher automatically unregisters it and emits a `WatchFileRemoved` event with reason `"object_sealed"`. You'll need to create a new version through the normal workflow.

### Registry corruption

The registry is stored as JSON at `~/.latticefs/watcher_registry.json`. If it becomes corrupted, simply delete it -- the daemon will start with an empty registry.
