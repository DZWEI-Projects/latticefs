# Configuration

Default config path:
```
~/.latticefs/config.toml
```

You can override the repo root with:
```
LATTICE_HOME=/path/to/repo lfs <command>
```

You can also override by command:
```
lfs --repo /path/to/repo <command>
```

## Per-directory auto-load

If you run `lfs` from a project directory and want that directory to be used as the repo root (without typing `--repo`), create `.latticefs.toml` in that directory:

```toml
[repo]
auto_load = true
```

Behavior:
- Precedence is `--repo` (highest), then `.latticefs.toml` auto-load, then global config/default (`~/.latticefs` or `LATTICE_HOME`).
- Only the current working directory is checked for `.latticefs.toml`.
- Any value other than `repo.auto_load = true` disables auto-load.

## Schema
```toml
[storage]
path = "~/.latticefs"
cache_size_mb = 512
max_chunk_size_kb = 64

[quota]
max_storage_gb = 100
max_operations_per_minute = 1000
burst_allowance = 100

[fuse]
mount_point = "~/Lattice"
readonly = true
allow_other = false

[crypto]
algorithm = "aes-256-gcm"
key_derivation = "argon2id"
keyring_service = "latticefs"

[share]
http_port = 8771
max_concurrent_shares = 10
default_ttl_days = 7

[import]
extract_exif = true
extract_id3 = true
extract_text = true
create_embeddings = false

[logging]
level = "info"
format = "json"
audit_log = "~/.latticefs/logs/events.jsonl"
```

## Notes
- The config file is created by `lfs init` or when the repo is first opened.
- `storage.path` is the repository root. Most data is stored under that path.
- `fuse.readonly` is true for MVP. Writes are done via CLI.
