use crate::config::QuotaConfig;
use crate::error::{LatticeError, Result};
use crate::storage::{chunk_data, compute_hash, ChunkStore};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct QuotaEnforcer {
    config: QuotaConfig,
}

impl QuotaEnforcer {
    pub fn new(config: QuotaConfig) -> Self {
        Self { config }
    }

    /// Check storage quota for new data and return a report.
    pub fn check_storage_quota(&self, store: &ChunkStore, data: &[u8]) -> Result<QuotaReport> {
        let current_bytes = total_chunk_bytes(store.root_path())?;
        let additional_bytes = estimate_additional_bytes(store, data)?;
        let max_bytes = self.config.max_storage_gb * 1024 * 1024 * 1024;
        let projected = current_bytes.saturating_add(additional_bytes);

        if projected > max_bytes {
            return Err(LatticeError::QuotaExceeded {
                current_bytes: projected,
                max_bytes,
            });
        }

        Ok(QuotaReport {
            current_bytes,
            additional_bytes,
            max_bytes,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct QuotaReport {
    pub current_bytes: u64,
    pub additional_bytes: u64,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitState {
    pub tokens: f64,
    pub last_refill: u64,
}

impl RateLimitState {
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: now_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_per_minute: f64,
    burst_allowance: f64,
}

impl RateLimiter {
    pub fn new(config: &QuotaConfig) -> Self {
        Self {
            max_per_minute: config.max_operations_per_minute as f64,
            burst_allowance: config.burst_allowance as f64,
        }
    }

    pub fn check_and_consume(
        &self,
        state: Option<RateLimitState>,
        ops: u64,
    ) -> Result<RateLimitState> {
        let capacity = self.max_per_minute + self.burst_allowance;
        let mut state = state.unwrap_or_else(|| RateLimitState::new(capacity));

        let now = now_secs();
        let elapsed = now.saturating_sub(state.last_refill) as f64;
        let refill_per_sec = self.max_per_minute / 60.0;
        state.tokens = (state.tokens + elapsed * refill_per_sec).min(capacity);
        state.last_refill = now;

        let ops_f = ops as f64;
        if state.tokens < ops_f {
            let deficit = ops_f - state.tokens;
            let retry_after = if refill_per_sec > 0.0 {
                (deficit / refill_per_sec).ceil() as u64
            } else {
                60
            };
            return Err(LatticeError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        state.tokens -= ops_f;
        Ok(state)
    }
}

fn estimate_additional_bytes(store: &ChunkStore, data: &[u8]) -> Result<u64> {
    let boundaries = chunk_data(data);
    let mut total = 0u64;

    for boundary in &boundaries {
        let chunk = &data[boundary.offset..boundary.offset + boundary.length];
        let hash = compute_hash(chunk);
        if !store.chunk_exists(&hash) {
            total = total.saturating_add(boundary.length as u64);
        }
    }

    Ok(total)
}

fn total_chunk_bytes(root: &Path) -> Result<u64> {
    let mut total = 0u64;
    let chunks_root = root.join("chunks");
    if !chunks_root.exists() {
        return Ok(0);
    }

    for entry in WalkDir::new(chunks_root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total = total.saturating_add(meta.len());
            }
        }
    }

    Ok(total)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}
