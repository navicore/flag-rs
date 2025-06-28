//! String interning pool for reducing memory usage
//!
//! This module provides a string interning mechanism to avoid storing
//! duplicate strings across the application, particularly useful for
//! flag names and command names in large CLIs.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// A thread-safe string interning pool
///
/// This structure ensures that identical strings are stored only once in memory,
/// returning shared references (`Arc<str>`) to the same underlying data.
#[derive(Clone)]
pub struct StringPool {
    pool: Arc<Mutex<HashSet<Arc<str>>>>,
}

impl StringPool {
    /// Creates a new empty string pool
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Interns a string, returning a shared reference
    ///
    /// If the string already exists in the pool, returns the existing reference.
    /// Otherwise, adds it to the pool and returns a new reference.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to intern
    ///
    /// # Returns
    ///
    /// An `Arc<str>` pointing to the interned string
    #[must_use]
    pub fn intern(&self, s: &str) -> Arc<str> {
        if let Ok(mut pool) = self.pool.lock() {
            // Check if string already exists
            if let Some(existing) = pool.get(s) {
                return Arc::clone(existing);
            }

            // Add new string to pool
            let arc_str: Arc<str> = Arc::from(s);
            pool.insert(Arc::clone(&arc_str));
            arc_str
        } else {
            // Fallback if lock fails
            Arc::from(s)
        }
    }

    /// Returns the number of unique strings in the pool
    #[must_use]
    pub fn size(&self) -> usize {
        self.pool.lock().map(|p| p.len()).unwrap_or(0)
    }

    /// Clears all strings from the pool
    pub fn clear(&self) {
        if let Ok(mut pool) = self.pool.lock() {
            pool.clear();
        }
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Global string pool instance
///
/// This is created lazily on first use.
fn global_pool() -> &'static StringPool {
    static POOL: std::sync::OnceLock<StringPool> = std::sync::OnceLock::new();
    POOL.get_or_init(StringPool::new)
}

/// Interns a string in the global pool
#[must_use]
pub fn intern(s: &str) -> Arc<str> {
    global_pool().intern(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let pool = StringPool::new();

        let s1 = pool.intern("hello");
        let s2 = pool.intern("hello");
        let s3 = pool.intern("world");

        // s1 and s2 should point to the same memory
        assert!(Arc::ptr_eq(&s1, &s2));
        // s3 should be different
        assert!(!Arc::ptr_eq(&s1, &s3));

        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_global_pool() {
        let s1 = intern("global");
        let s2 = intern("global");

        assert!(Arc::ptr_eq(&s1, &s2));
    }

    #[test]
    fn test_clear() {
        let pool = StringPool::new();
        let _ = pool.intern("test1");
        let _ = pool.intern("test2");
        assert_eq!(pool.size(), 2);

        pool.clear();
        assert_eq!(pool.size(), 0);
    }
}
