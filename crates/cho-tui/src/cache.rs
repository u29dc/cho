//! Route payload cache for interactive navigation.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::api::RoutePayload;

/// Cache quality tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheTier {
    /// Fast preview payload (smaller list limit).
    Preview,
    /// Full payload (normal list limit).
    Full,
}

/// Cache lookup key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    stored_at: Instant,
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

    /// Inserts or updates a cache entry.
    pub fn insert(&mut self, key: CacheKey, payload: RoutePayload) {
        if self.entries.contains_key(&key) {
            self.touch(&key);
        } else {
            self.lru.push_back(key.clone());
        }

        self.entries.insert(
            key,
            CacheEntry {
                payload,
                stored_at: Instant::now(),
            },
        );
        self.evict_if_needed();
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

    fn lookup_with_ttl(&self, key: &CacheKey, ttl: Duration) -> Option<CacheSnapshot> {
        let entry = self.entries.get(key)?;
        let age = entry.stored_at.elapsed();
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
