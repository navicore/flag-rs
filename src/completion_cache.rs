//! Caching system for expensive completion operations
//!
//! This module provides a time-based cache for completion results to improve
//! performance when users repeatedly request completions for the same context.

use crate::completion::CompletionResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A cached completion entry with timestamp
#[derive(Clone)]
struct CacheEntry {
    result: CompletionResult,
    timestamp: Instant,
}

/// A thread-safe cache for completion results
///
/// The cache automatically expires entries after a configurable duration
/// to ensure that completions remain fresh while still providing performance benefits.
pub struct CompletionCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

impl CompletionCache {
    /// Creates a new completion cache with the specified time-to-live
    ///
    /// # Arguments
    ///
    /// * `ttl` - How long cached entries should remain valid
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    /// Creates a new completion cache with a default TTL of 5 seconds
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(5))
    }

    /// Generates a cache key from completion context
    ///
    /// The key includes the command path and current prefix to ensure
    /// we only return cached results for identical contexts.
    pub fn make_key(
        command_path: &[String],
        prefix: &str,
        flags: &HashMap<String, String>,
    ) -> String {
        let mut parts = vec![];

        // Include command path
        parts.extend(command_path.iter().cloned());

        // Include the prefix being completed
        parts.push(format!("__prefix:{prefix}"));

        // Include relevant flags that might affect completion
        let mut flag_parts: Vec<String> = flags.iter().map(|(k, v)| format!("{k}={v}")).collect();
        flag_parts.sort(); // Ensure consistent ordering
        parts.extend(flag_parts);

        parts.join(":")
    }

    /// Attempts to get a cached completion result
    ///
    /// Returns `Some(CompletionResult)` if a valid cached entry exists,
    /// or `None` if the entry doesn't exist or has expired.
    pub fn get(&self, key: &str) -> Option<CompletionResult> {
        let mut cache = self.cache.lock().ok()?;

        if let Some(entry) = cache.get(key) {
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.result.clone());
            }
            // Entry has expired, remove it
            cache.remove(key);
        }

        None
    }

    /// Stores a completion result in the cache
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    /// * `result` - The completion result to cache
    pub fn put(&self, key: String, result: CompletionResult) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                key,
                CacheEntry {
                    result,
                    timestamp: Instant::now(),
                },
            );

            // Opportunistically clean up expired entries
            self.cleanup_expired(&mut cache);
        }
    }

    /// Removes expired entries from the cache
    fn cleanup_expired(&self, cache: &mut HashMap<String, CacheEntry>) {
        let now = Instant::now();
        cache.retain(|_, entry| now.duration_since(entry.timestamp) < self.ttl);
    }

    /// Clears all cached entries
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Returns the number of cached entries
    pub fn size(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }
}

impl Default for CompletionCache {
    fn default() -> Self {
        Self::with_default_ttl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion::CompletionResult;

    #[test]
    fn test_cache_basic_operations() {
        let cache = CompletionCache::new(Duration::from_secs(1));
        let key = "test:key";
        let result = CompletionResult::new().add("item1").add("item2");

        // Test cache miss
        assert!(cache.get(key).is_none());

        // Test cache put and hit
        cache.put(key.to_string(), result.clone());
        let cached = cache.get(key).unwrap();
        assert_eq!(cached.values, result.values);

        // Test expiration
        std::thread::sleep(Duration::from_millis(1100));
        assert!(cache.get(key).is_none());
    }

    #[test]
    fn test_cache_key_generation() {
        let mut flags = HashMap::new();
        flags.insert("namespace".to_string(), "default".to_string());
        flags.insert("verbose".to_string(), "true".to_string());

        let key1 =
            CompletionCache::make_key(&["kubectl".to_string(), "get".to_string()], "po", &flags);
        let key2 =
            CompletionCache::make_key(&["kubectl".to_string(), "get".to_string()], "po", &flags);
        assert_eq!(key1, key2);

        // Different prefix should generate different key
        let key3 =
            CompletionCache::make_key(&["kubectl".to_string(), "get".to_string()], "pod", &flags);
        assert_ne!(key1, key3);

        // Different flags should generate different key
        flags.insert("all-namespaces".to_string(), "true".to_string());
        let key4 =
            CompletionCache::make_key(&["kubectl".to_string(), "get".to_string()], "po", &flags);
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = CompletionCache::new(Duration::from_millis(100));

        // Add multiple entries
        for i in 0..5 {
            let key = format!("key{i}");
            let result = CompletionResult::new().add(format!("item{i}"));
            cache.put(key, result);
        }

        assert_eq!(cache.size(), 5);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Trigger cleanup by adding a new entry
        cache.put("new".to_string(), CompletionResult::new());

        // Only the new entry should remain
        assert_eq!(cache.size(), 1);
    }
}
