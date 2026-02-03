//! Rate limiter for Xero API requests.
//!
//! Implements a token bucket algorithm tracking:
//! - **Concurrent limit**: 5 in-flight requests per app+org connection.
//! - **Per-minute limit**: 60 calls per minute per app+org connection.
//! - **Daily limit**: 5,000 calls per day per app+org connection.
//!
//! Also tracks `X-MinLimit-Remaining`, `X-DayLimit-Remaining`, and
//! `X-AppMinLimit-Remaining` response headers to pre-emptively delay when
//! approaching limits.

use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, warn};

/// Default concurrent request limit.
const DEFAULT_CONCURRENT: usize = 5;

/// Default per-minute request limit.
const DEFAULT_PER_MINUTE: u32 = 60;

/// Rate limiter configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum concurrent requests (default: 5).
    pub max_concurrent: usize,

    /// Maximum requests per minute (default: 60).
    pub max_per_minute: u32,

    /// Whether rate limiting is enabled (set false for tests).
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_concurrent: DEFAULT_CONCURRENT,
            max_per_minute: DEFAULT_PER_MINUTE,
            enabled: true,
        }
    }
}

/// Tracks rate limit state from Xero response headers.
#[derive(Debug, Default)]
struct HeaderLimits {
    /// Remaining requests this minute (from `X-MinLimit-Remaining`).
    min_remaining: Option<u32>,

    /// Remaining requests today (from `X-DayLimit-Remaining`).
    day_remaining: Option<u32>,

    /// Remaining app-wide requests this minute (from `X-AppMinLimit-Remaining`).
    app_min_remaining: Option<u32>,
}

/// Per-minute request tracker using a sliding window.
#[derive(Debug)]
struct MinuteTracker {
    /// Timestamps of recent requests within the last minute.
    timestamps: Vec<Instant>,

    /// Maximum requests per minute.
    max: u32,
}

impl MinuteTracker {
    fn new(max: u32) -> Self {
        Self {
            timestamps: Vec::with_capacity(max as usize),
            max,
        }
    }

    /// Records a request and returns the delay needed (if any) before the
    /// request should proceed.
    fn record(&mut self) -> Option<Duration> {
        let now = Instant::now();
        let window = Duration::from_secs(60);

        // Remove expired timestamps
        self.timestamps.retain(|&t| now.duration_since(t) < window);

        if self.timestamps.len() >= self.max as usize {
            // Calculate when the oldest request will expire from the window
            let oldest = self.timestamps[0];
            let wait = window
                .checked_sub(now.duration_since(oldest))
                .unwrap_or(Duration::from_millis(100));
            Some(wait)
        } else {
            self.timestamps.push(now);
            None
        }
    }
}

/// Rate limiter that enforces concurrent and per-minute limits.
pub struct RateLimiter {
    /// Semaphore for concurrent request limit.
    concurrent: Arc<Semaphore>,

    /// Per-minute request tracking.
    minute_tracker: Arc<RwLock<MinuteTracker>>,

    /// Rate limits reported by Xero response headers.
    header_limits: Arc<RwLock<HeaderLimits>>,

    /// Configuration.
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Creates a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            concurrent: Arc::new(Semaphore::new(config.max_concurrent)),
            minute_tracker: Arc::new(RwLock::new(MinuteTracker::new(config.max_per_minute))),
            header_limits: Arc::new(RwLock::new(HeaderLimits::default())),
            config,
        }
    }

    /// Creates a rate limiter with default settings.
    pub fn default_limiter() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Creates a disabled rate limiter (for tests).
    pub fn disabled() -> Self {
        Self::new(RateLimitConfig {
            enabled: false,
            ..Default::default()
        })
    }

    /// Acquires a rate limit permit. Returns a guard that must be held for
    /// the duration of the request.
    ///
    /// This method will block if:
    /// - The concurrent limit is reached (waits for a permit).
    /// - The per-minute limit is reached (sleeps until the window moves).
    /// - Xero headers indicate limits are nearly exhausted.
    pub async fn acquire(&self) -> crate::error::Result<RateLimitGuard<'_>> {
        if !self.config.enabled {
            return Ok(RateLimitGuard {
                _permit: None,
                limiter: self,
            });
        }

        // Check header-reported limits
        {
            let limits = self.header_limits.read().await;
            if let Some(remaining) = limits.min_remaining
                && remaining <= 2
            {
                debug!("Xero X-MinLimit-Remaining is {remaining}, throttling");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            if let Some(remaining) = limits.app_min_remaining
                && remaining <= 5
            {
                debug!("Xero X-AppMinLimit-Remaining is {remaining}, throttling");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            if let Some(remaining) = limits.day_remaining
                && remaining <= 10
            {
                warn!("Xero daily limit nearly exhausted: {remaining} remaining");
            }
        }

        // Check per-minute limit
        loop {
            let delay = {
                let mut tracker = self.minute_tracker.write().await;
                tracker.record()
            };

            match delay {
                Some(wait) => {
                    debug!("Per-minute limit reached, waiting {wait:?}");
                    tokio::time::sleep(wait).await;
                }
                None => break,
            }
        }

        // Acquire concurrent permit
        let permit = self.concurrent.clone().acquire_owned().await.map_err(|_| {
            crate::error::ChoSdkError::Config {
                message: "Rate limiter semaphore closed".to_string(),
            }
        })?;

        Ok(RateLimitGuard {
            _permit: Some(permit),
            limiter: self,
        })
    }

    /// Updates rate limit state from response headers.
    pub async fn update_from_headers(&self, headers: &reqwest::header::HeaderMap) {
        let mut limits = self.header_limits.write().await;

        if let Some(val) = headers.get("X-MinLimit-Remaining")
            && let Ok(s) = val.to_str()
        {
            limits.min_remaining = s.parse().ok();
        }

        if let Some(val) = headers.get("X-DayLimit-Remaining")
            && let Ok(s) = val.to_str()
        {
            limits.day_remaining = s.parse().ok();
        }

        if let Some(val) = headers.get("X-AppMinLimit-Remaining")
            && let Ok(s) = val.to_str()
        {
            limits.app_min_remaining = s.parse().ok();
        }
    }

    /// Handles a 429 response by sleeping for the Retry-After duration plus jitter.
    ///
    /// Random jitter (0-2 seconds) is added to the Retry-After value to prevent
    /// thundering herd when multiple clients hit rate limits simultaneously.
    pub async fn handle_rate_limited(
        &self,
        headers: &reqwest::header::HeaderMap,
    ) -> crate::error::Result<()> {
        let retry_after = headers
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(5);

        // Add random jitter (0-2000ms) to prevent thundering herd
        let jitter_ms = rand::rng().random_range(0..2000);
        let total_wait = Duration::from_secs(retry_after) + Duration::from_millis(jitter_ms);

        warn!(
            retry_after_secs = retry_after,
            jitter_ms = jitter_ms,
            "Rate limited (429), retrying after {}ms",
            total_wait.as_millis()
        );
        tokio::time::sleep(total_wait).await;
        Ok(())
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("config", &self.config)
            .finish()
    }
}

/// Guard that holds a rate limit permit for the duration of a request.
///
/// The concurrent permit is released when the guard is dropped.
pub struct RateLimitGuard<'a> {
    _permit: Option<tokio::sync::OwnedSemaphorePermit>,
    limiter: &'a RateLimiter,
}

impl<'a> RateLimitGuard<'a> {
    /// Updates the limiter with response headers after the request completes.
    pub async fn complete(&self, headers: &reqwest::header::HeaderMap) {
        self.limiter.update_from_headers(headers).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minute_tracker_allows_within_limit() {
        let mut tracker = MinuteTracker::new(5);
        for _ in 0..5 {
            assert!(tracker.record().is_none());
        }
        // 6th request should be delayed
        assert!(tracker.record().is_some());
    }

    #[test]
    fn rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_concurrent, 5);
        assert_eq!(config.max_per_minute, 60);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn disabled_limiter_always_acquires() {
        let limiter = RateLimiter::disabled();
        let _guard = limiter.acquire().await.unwrap();
        // Should succeed immediately without blocking
    }

    #[tokio::test]
    async fn update_from_headers() {
        let limiter = RateLimiter::default_limiter();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-MinLimit-Remaining", "10".parse().unwrap());
        headers.insert("X-DayLimit-Remaining", "4500".parse().unwrap());
        headers.insert("X-AppMinLimit-Remaining", "9500".parse().unwrap());
        limiter.update_from_headers(&headers).await;

        let limits = limiter.header_limits.read().await;
        assert_eq!(limits.min_remaining, Some(10));
        assert_eq!(limits.day_remaining, Some(4500));
        assert_eq!(limits.app_min_remaining, Some(9500));
    }
}
