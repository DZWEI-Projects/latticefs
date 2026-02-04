# Quotas and Rate Limiting

LatticeFS enforces simple quotas and rate limits to prevent abuse.

## Storage quota
Configured in `config.toml`:
```toml
[quota]
max_storage_gb = 100
```

Before new data is stored, the system estimates how many **new** bytes would be written (dedup-aware). If the projected size exceeds the quota, the operation fails with `QuotaExceeded`.

## Rate limiting
Rate limiting uses a token bucket per repository.

Config:
```toml
[quota]
max_operations_per_minute = 1000
burst_allowance = 100
```

When the limit is exceeded, the operation fails with `RateLimited` and includes a retry-after duration.
