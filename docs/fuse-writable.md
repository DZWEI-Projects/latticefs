# Writable FUSE Support (Future Plan)

This document outlines the design for future writable FUSE support in LatticeFS. Currently, the FUSE mount is read-only. Writable FUSE would allow editing files directly through the mounted filesystem, with changes automatically creating new versions.

## Overview

Writable FUSE would intercept `write()`, `release()`, and `setattr()` operations on the mounted filesystem and create new LatticeFS versions from the modified content.

## Architecture

### Write-Through Model

The FUSE layer would use a write-through approach:

1. **`open()` for writing** -- Export the object to a temporary staging area (same `watch_dir` used by the file watcher)
2. **`write()` calls** -- Buffer writes to the staging file
3. **`release()` (close)** -- Compute BLAKE3 hash, compare with last version, create new version if changed via `add_version_from_bytes()`

### Integration with FileWatcher

Two approaches for writable FUSE:

#### Option A: In-Process Handling (Preferred)

The FUSE layer handles versioning directly without the watcher daemon:
- `release()` calls `repo.add_version_from_bytes()` synchronously
- No IPC overhead, lower latency
- FUSE process needs direct repo access (already the case)

#### Option B: Delegate to Watcher

The FUSE layer writes to `watch_dir` and lets the watcher daemon detect changes:
- Reuses existing watcher infrastructure
- Adds IPC latency to every save
- Watcher daemon must be running

**Recommendation**: Option A for FUSE, with Option B as fallback if FUSE is not available.

### Inode Management

Challenges:
- LatticeFS objects don't have stable inode numbers
- FUSE requires consistent inode -> content mapping during a session
- Solution: Maintain an in-memory inode table mapping `ino -> (ObjectID, VersionID)`, updated on version creation

### Atomic Writes

Many editors use atomic save patterns:
1. Write to `filename.tmp`
2. Rename `filename.tmp` -> `filename`

The FUSE layer must:
- Track temporary files created in the same directory as watched objects
- Detect the rename pattern and treat it as a modification of the target object
- Apply the same debouncing/dedup logic as the file watcher

### Conflict Resolution

If an object is modified both through FUSE and through the CLI/GUI simultaneously:
- LatticeFS versioning is append-only, so both modifications create separate versions
- The VersionDAG records the parent of each version
- If both start from the same parent, this creates a fork in the DAG
- Resolution is deferred to the user (similar to git merge conflicts)

### Cache Coherency

- Read cache must be invalidated when a new version is created through FUSE
- Other readers (CLI, GUI) should see the new version via the EventBus
- The LRU cache in the current FUSE implementation needs write-through invalidation

## Performance Considerations

- **Debouncing**: Buffer writes and only create versions on `release()`, not on every `write()` call
- **Large files**: For files larger than `max_chunk_size_kb`, use streaming writes to avoid holding entire content in memory
- **FastCDC**: The existing content-defined chunking means only changed chunks need to be stored

## Security

- Writable FUSE should respect the same policy engine and permission checks
- `ObjectSealed` state must prevent writes (return `EACCES`)
- Rate limiting should apply to FUSE writes

## Implementation Steps

1. Add `writable: bool` option to `FuseConfig` (default: `false`)
2. Extend `LatticeFS` (FUSE handler) with a staging directory and write buffer
3. Implement `write()`, `setattr()`, and `release()` FUSE operations
4. Add inode table with version tracking
5. Handle atomic-save rename patterns
6. Add write-through cache invalidation
7. Integration tests with real FUSE mount

## Prerequisites

- File watcher daemon (completed)
- Stable repo access from FUSE process (existing)
- Event system for cross-component notifications (existing)
