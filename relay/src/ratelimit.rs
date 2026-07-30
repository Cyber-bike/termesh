//! Fixed-window rate limiting, per doc 6.5.
//!
//! In-memory and therefore reset by a restart, which doc 6.5 explicitly accepts
//! for the MVP single node. A fixed window is chosen over a sliding one because
//! the thresholds here exist to blunt brute force, not to shape traffic: the
//! worst case is 2x the nominal rate across a window boundary, which does not
//! matter for "5 login attempts per minute".

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::AppError;

/// One configured limit.
#[derive(Clone, Copy)]
pub struct Limit {
    pub max: u32,
    pub window: Duration,
    pub retry_after_secs: u32,
}

impl Limit {
    pub const fn new(max: u32, window_secs: u64, retry_after_secs: u32) -> Self {
        Self {
            max,
            window: Duration::from_secs(window_secs),
            retry_after_secs,
        }
    }
}

/// Doc 6.5, verbatim.
pub mod limits {
    use super::Limit;

    pub const LOGIN: Limit = Limit::new(5, 60, 60);
    pub const SIGN_UP: Limit = Limit::new(5, 3600, 3600);
    pub const CREATE_PAIRING_CODE: Limit = Limit::new(10, 3600, 300);
    pub const REGISTER_BY_IP: Limit = Limit::new(10, 60, 60);
    pub const REGISTER_BY_CODE: Limit = Limit::new(5, 3600, 3600);
    pub const LIST_DEVICES: Limit = Limit::new(60, 60, 10);
    pub const WS_UPGRADE: Limit = Limit::new(30, 60, 60);
}

struct Window {
    started: Instant,
    count: u32,
}

pub struct RateLimiter {
    buckets: Mutex<HashMap<String, Window>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Counts one hit against `key`. Returns Err(429) once the limit is exceeded.
    pub fn check(&self, key: &str, limit: Limit) -> Result<(), AppError> {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().expect("rate limiter mutex poisoned");

        // Opportunistic sweep so a long-running process does not accumulate a
        // bucket per attacker address forever. Cheap because it only runs when
        // the map has grown.
        if buckets.len() > 4096 {
            buckets.retain(|_, w| now.duration_since(w.started) < limit.window);
        }

        let entry = buckets.entry(key.to_string()).or_insert(Window {
            started: now,
            count: 0,
        });

        if now.duration_since(entry.started) >= limit.window {
            entry.started = now;
            entry.count = 0;
        }

        entry.count += 1;
        if entry.count > limit.max {
            return Err(AppError::rate_limited(limit.retry_after_secs));
        }

        Ok(())
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.buckets.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_limit_then_rejects() {
        let limiter = RateLimiter::new();
        let limit = Limit::new(3, 60, 42);

        for _ in 0..3 {
            assert!(limiter.check("alice", limit).is_ok());
        }

        let err = limiter.check("alice", limit).unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.retry_after, Some(42));
    }

    #[test]
    fn keys_are_independent() {
        let limiter = RateLimiter::new();
        let limit = Limit::new(1, 60, 1);

        assert!(limiter.check("alice", limit).is_ok());
        assert!(limiter.check("alice", limit).is_err());
        assert!(limiter.check("bob", limit).is_ok());
    }

    #[test]
    fn window_resets() {
        let limiter = RateLimiter::new();
        // A zero-length window expires immediately, so every call starts fresh.
        let limit = Limit::new(1, 0, 1);

        assert!(limiter.check("alice", limit).is_ok());
        assert!(limiter.check("alice", limit).is_ok());
    }

    #[test]
    fn documented_thresholds_match_the_spec() {
        // Guards against a silent edit of the numbers doc 6.5 fixes.
        assert_eq!((limits::LOGIN.max, limits::LOGIN.window.as_secs()), (5, 60));
        assert_eq!(limits::LOGIN.retry_after_secs, 60);
        assert_eq!(
            (
                limits::CREATE_PAIRING_CODE.max,
                limits::CREATE_PAIRING_CODE.window.as_secs()
            ),
            (10, 3600)
        );
        assert_eq!(
            (
                limits::REGISTER_BY_IP.max,
                limits::REGISTER_BY_IP.window.as_secs()
            ),
            (10, 60)
        );
        assert_eq!(
            (
                limits::REGISTER_BY_CODE.max,
                limits::REGISTER_BY_CODE.window.as_secs()
            ),
            (5, 3600)
        );
        assert_eq!(
            (
                limits::LIST_DEVICES.max,
                limits::LIST_DEVICES.window.as_secs()
            ),
            (60, 60)
        );
        assert_eq!(
            (limits::WS_UPGRADE.max, limits::WS_UPGRADE.window.as_secs()),
            (30, 60)
        );
    }

    #[test]
    fn sweeps_stale_buckets() {
        let limiter = RateLimiter::new();
        let limit = Limit::new(1, 0, 1);
        for i in 0..5000 {
            let _ = limiter.check(&format!("key-{i}"), limit);
        }
        assert!(
            limiter.len() <= 4097,
            "stale buckets should be swept, got {}",
            limiter.len()
        );
    }
}
