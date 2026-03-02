//! Route payload cache for interactive navigation.

use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::RoutePayload;

const CACHE_FILE_VERSION: u32 = 1;
const MAX_CACHE_FILE_SIZE: u64 = 8 * 1024 * 1024;

/// Cache quality tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheTier {
    /// Fast preview payload (smaller list limit).
    Preview,
    /// Full payload (normal list limit).
    Full,
}

/// Cache lookup key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Stable route id.
    pub route_id: String,
    /// Route-specific context fingerprint.
    pub context: String,
    /// Payload quality tier.
    pub tier: CacheTier,
}

impl CacheKey {
    /// Creates a cache key.
    pub fn new(route_id: String, context: String, tier: CacheTier) -> Self {
        Self {
            route_id,
            context,
            tier,
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    payload: RoutePayload,
    stored_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedRouteCache {
    version: u32,
    entries: Vec<PersistedCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedCacheEntry {
    key: CacheKey,
    payload: RoutePayload,
    stored_at: DateTime<Utc>,
}

/// Snapshot returned by cache lookups.
#[derive(Debug, Clone)]
pub struct CacheSnapshot {
    /// Cached payload.
    pub payload: RoutePayload,
    /// Payload freshness flag.
    pub fresh: bool,
    /// Payload age.
    pub age: Duration,
    /// Source cache tier.
    pub tier: CacheTier,
}

/// In-memory bounded route cache.
#[derive(Debug)]
pub struct RouteCache {
    entries: HashMap<CacheKey, CacheEntry>,
    lru: VecDeque<CacheKey>,
    capacity: usize,
}

impl RouteCache {
    /// Creates a new cache with bounded capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            lru: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    /// Loads cache from disk when available.
    pub fn load_from_disk(path: &Path, capacity: usize, max_age: Duration) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new(capacity));
        }

        let metadata = std::fs::metadata(path).map_err(|e| {
            format!(
                "failed reading route cache metadata {}: {e}",
                path.display()
            )
        })?;
        if metadata.len() > MAX_CACHE_FILE_SIZE {
            return Err(format!(
                "route cache {} exceeds max size ({} > {})",
                path.display(),
                metadata.len(),
                MAX_CACHE_FILE_SIZE
            ));
        }

        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("failed reading route cache {}: {e}", path.display()))?;
        let persisted = serde_json::from_str::<PersistedRouteCache>(&raw)
            .map_err(|e| format!("failed parsing route cache {}: {e}", path.display()))?;

        if persisted.version != CACHE_FILE_VERSION {
            return Err(format!(
                "unsupported route cache version {} (expected {}) in {}",
                persisted.version,
                CACHE_FILE_VERSION,
                path.display()
            ));
        }

        let mut cache = Self::new(capacity);
        for entry in persisted.entries {
            let age = age_since(entry.stored_at);
            if age <= max_age {
                cache.insert_with_timestamp(entry.key, entry.payload, entry.stored_at);
            }
        }
        Ok(cache)
    }

    /// Persists cache to disk.
    pub fn save_to_disk(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("failed creating cache directory {}: {e}", parent.display())
            })?;
        }

        let persisted = PersistedRouteCache {
            version: CACHE_FILE_VERSION,
            entries: self.persisted_entries(),
        };

        let raw = serde_json::to_string(&persisted)
            .map_err(|e| format!("failed serializing route cache for disk: {e}"))?;

        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, raw).map_err(|e| {
            format!(
                "failed writing route cache temp file {}: {e}",
                tmp.display()
            )
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600)).map_err(
                |e| {
                    format!(
                        "failed setting route cache permissions on {}: {e}",
                        tmp.display()
                    )
                },
            )?;
        }

        std::fs::rename(&tmp, path).map_err(|e| {
            format!(
                "failed replacing route cache file {} from {}: {e}",
                path.display(),
                tmp.display()
            )
        })
    }

    /// Inserts or updates a cache entry.
    pub fn insert(&mut self, key: CacheKey, payload: RoutePayload) {
        self.insert_with_timestamp(key, payload, Utc::now());
    }

    /// Returns best available payload for route/context.
    /// Preference order:
    /// full fresh -> preview fresh -> full stale -> preview stale.
    pub fn best_for_route(
        &self,
        route_id: &str,
        context: &str,
        preview_ttl: Duration,
        full_ttl: Duration,
    ) -> Option<CacheSnapshot> {
        let full_key = CacheKey::new(route_id.to_string(), context.to_string(), CacheTier::Full);
        let preview_key = CacheKey::new(
            route_id.to_string(),
            context.to_string(),
            CacheTier::Preview,
        );

        let full = self.lookup_with_ttl(&full_key, full_ttl);
        let preview = self.lookup_with_ttl(&preview_key, preview_ttl);

        if let Some(snapshot) = full.as_ref()
            && snapshot.fresh
        {
            return Some(snapshot.clone());
        }

        if let Some(snapshot) = preview.as_ref()
            && snapshot.fresh
        {
            return Some(snapshot.clone());
        }

        match (full, preview) {
            (Some(full), Some(preview)) => {
                if full.age <= preview.age {
                    Some(full)
                } else {
                    Some(preview)
                }
            }
            (Some(full), None) => Some(full),
            (None, Some(preview)) => Some(preview),
            (None, None) => None,
        }
    }

    fn persisted_entries(&self) -> Vec<PersistedCacheEntry> {
        self.lru
            .iter()
            .filter_map(|key| {
                self.entries.get(key).map(|entry| PersistedCacheEntry {
                    key: key.clone(),
                    payload: entry.payload.clone(),
                    stored_at: entry.stored_at,
                })
            })
            .collect()
    }

    fn insert_with_timestamp(
        &mut self,
        key: CacheKey,
        payload: RoutePayload,
        stored_at: DateTime<Utc>,
    ) {
        if self.entries.contains_key(&key) {
            self.touch(&key);
        } else {
            self.lru.push_back(key.clone());
        }

        self.entries.insert(key, CacheEntry { payload, stored_at });
        self.evict_if_needed();
    }

    fn lookup_with_ttl(&self, key: &CacheKey, ttl: Duration) -> Option<CacheSnapshot> {
        let entry = self.entries.get(key)?;
        let age = age_since(entry.stored_at);
        Some(CacheSnapshot {
            payload: entry.payload.clone(),
            fresh: age <= ttl,
            age,
            tier: key.tier,
        })
    }

    fn touch(&mut self, key: &CacheKey) {
        if let Some(index) = self.lru.iter().position(|existing| existing == key) {
            self.lru.remove(index);
        }
        self.lru.push_back(key.clone());
    }

    fn evict_if_needed(&mut self) {
        while self.entries.len() > self.capacity {
            let Some(oldest) = self.lru.pop_front() else {
                break;
            };
            self.entries.remove(&oldest);
        }
    }
}

fn age_since(stored_at: DateTime<Utc>) -> Duration {
    let elapsed = Utc::now().signed_duration_since(stored_at);
    elapsed.to_std().unwrap_or(Duration::ZERO)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::Duration as ChronoDuration;

    use super::*;

    #[test]
    fn route_cache_persists_and_loads_from_disk() {
        let path = unique_cache_path("roundtrip");
        let key = CacheKey::new(
            "invoices".to_string(),
            "resource=static".to_string(),
            CacheTier::Full,
        );
        let payload = RoutePayload::Message("cached".to_string());

        let mut cache = RouteCache::new(8);
        cache.insert(key.clone(), payload);
        cache
            .save_to_disk(&path)
            .expect("route cache should persist");

        let loaded = RouteCache::load_from_disk(&path, 8, Duration::from_secs(60))
            .expect("route cache should load");
        let snapshot = loaded
            .best_for_route(
                "invoices",
                "resource=static",
                Duration::from_secs(10),
                Duration::from_secs(10),
            )
            .expect("cached snapshot should exist");

        match snapshot.payload {
            RoutePayload::Message(text) => assert_eq!(text, "cached"),
            other => panic!("expected message payload, got {other:?}"),
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn route_cache_load_skips_entries_older_than_max_age() {
        let path = unique_cache_path("max-age");
        let key = CacheKey::new(
            "invoices".to_string(),
            "resource=static".to_string(),
            CacheTier::Full,
        );
        let payload = RoutePayload::Message("old".to_string());

        let mut cache = RouteCache::new(8);
        cache.insert_with_timestamp(key, payload, Utc::now() - ChronoDuration::hours(6));
        cache
            .save_to_disk(&path)
            .expect("route cache should persist");

        let loaded = RouteCache::load_from_disk(&path, 8, Duration::from_secs(30))
            .expect("route cache should load");
        assert!(
            loaded
                .best_for_route(
                    "invoices",
                    "resource=static",
                    Duration::from_secs(10),
                    Duration::from_secs(10)
                )
                .is_none(),
            "old entries should be discarded when loading from disk"
        );

        let _ = std::fs::remove_file(path);
    }

    fn unique_cache_path(suffix: &str) -> PathBuf {
        let nanos = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .unsigned_abs();
        std::env::temp_dir().join(format!(
            "cho-tui-cache-{suffix}-{}-{}.json",
            std::process::id(),
            nanos
        ))
    }
}
