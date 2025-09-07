//! Performance and optimization tests for shell completion
//!
//! These tests verify that completion caching and timeout features work correctly.

use flag_rs::completion_cache::CompletionCache;
use flag_rs::completion_timeout::{make_timeout_completion, with_timeout};
use flag_rs::{CommandBuilder, CompletionResult, Context};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_completion_cache_functionality() {
    let cache = CompletionCache::new(Duration::from_millis(100));

    // Create a completion result
    let result = CompletionResult::new()
        .add("item1")
        .add_with_description("item2", "Description");

    let key = CompletionCache::make_key(
        &["app".to_string(), "cmd".to_string()],
        "prefix",
        &std::collections::HashMap::new(),
    );

    // Cache miss
    assert!(cache.get(&key).is_none());

    // Store in cache
    cache.put(key.clone(), result.clone());

    // Cache hit
    let cached = cache.get(&key).unwrap();
    assert_eq!(cached.values, result.values);
    assert_eq!(cached.descriptions, result.descriptions);

    // Wait for expiration
    thread::sleep(Duration::from_millis(150));

    // Cache miss after expiration
    assert!(cache.get(&key).is_none());
}

#[test]
fn test_completion_cache_with_different_contexts() {
    let cache = CompletionCache::new(Duration::from_secs(1));

    let result1 = CompletionResult::new().add("result1");
    let result2 = CompletionResult::new().add("result2");

    // Different prefixes should have different keys
    let key1 = CompletionCache::make_key(
        &["app".to_string()],
        "prefix1",
        &std::collections::HashMap::new(),
    );
    let key2 = CompletionCache::make_key(
        &["app".to_string()],
        "prefix2",
        &std::collections::HashMap::new(),
    );

    cache.put(key1.clone(), result1);
    cache.put(key2.clone(), result2);

    assert_eq!(cache.get(&key1).unwrap().values, vec!["result1"]);
    assert_eq!(cache.get(&key2).unwrap().values, vec!["result2"]);

    // Different flags should have different keys
    let mut flags1 = std::collections::HashMap::new();
    flags1.insert("verbose".to_string(), "true".to_string());

    let mut flags2 = std::collections::HashMap::new();
    flags2.insert("quiet".to_string(), "true".to_string());

    let key3 = CompletionCache::make_key(&["app".to_string()], "prefix", &flags1);
    let key4 = CompletionCache::make_key(&["app".to_string()], "prefix", &flags2);

    assert_ne!(key3, key4);
}

#[test]
fn test_completion_cache_cleanup() {
    let cache = CompletionCache::new(Duration::from_millis(50));

    // Add multiple entries
    for i in 0..5 {
        let key = format!("key{i}");
        let result = CompletionResult::new().add(format!("value{i}"));
        cache.put(key, result);
    }

    assert_eq!(cache.size(), 5);

    // Wait for expiration
    thread::sleep(Duration::from_millis(100));

    // Add a new entry to trigger cleanup
    cache.put("new".to_string(), CompletionResult::new());

    // Old entries should be cleaned up
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_completion_timeout_success() {
    let ctx = Context::new(vec!["app".to_string()]);

    let start = Instant::now();
    let result = with_timeout(
        |_ctx, prefix| {
            // Fast completion
            Ok(CompletionResult::new().add(format!("fast-{prefix}")))
        },
        Duration::from_secs(1),
        &ctx,
        "test",
    );

    assert!(result.is_ok());
    assert!(start.elapsed() < Duration::from_millis(100));

    let completion = result.unwrap();
    assert_eq!(completion.values, vec!["fast-test"]);
}

#[test]
fn test_completion_timeout_exceeded() {
    let ctx = Context::new(vec!["app".to_string()]);

    let start = Instant::now();
    let result = with_timeout(
        |_ctx, _prefix| {
            // Slow completion that will timeout
            thread::sleep(Duration::from_millis(200));
            Ok(CompletionResult::new().add("never-returned"))
        },
        Duration::from_millis(50),
        &ctx,
        "test",
    );

    assert!(result.is_ok());
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(50));
    assert!(elapsed < Duration::from_millis(100));

    let completion = result.unwrap();
    // Should have timeout warning
    assert!(!completion.active_help.is_empty());
    assert!(completion.active_help[0].message.contains("timed out"));
    // Should not have the completion value
    assert!(completion.values.is_empty());
}

#[test]
fn test_make_timeout_completion_wrapper() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = Arc::clone(&call_count);

    let wrapped = make_timeout_completion(Duration::from_millis(100), move |_ctx, prefix| {
        count_clone.fetch_add(1, Ordering::SeqCst);
        if prefix == "slow" {
            thread::sleep(Duration::from_millis(200));
        }
        Ok(CompletionResult::new().add(format!("result-{prefix}")))
    });

    let ctx = Context::new(vec![]);

    // Fast completion should work
    let result = wrapped(&ctx, "fast").unwrap();
    assert_eq!(result.values, vec!["result-fast"]);

    // Slow completion should timeout
    let result = wrapped(&ctx, "slow").unwrap();
    assert!(
        result
            .active_help
            .iter()
            .any(|h| h.message.contains("timed out"))
    );

    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_cached_completion_integration() {
    let cache = Arc::new(CompletionCache::new(Duration::from_secs(1)));
    let call_count = Arc::new(AtomicUsize::new(0));

    let cache_clone = Arc::clone(&cache);
    let count_clone = Arc::clone(&call_count);

    let app = CommandBuilder::new("cachetest")
        .arg_completion(move |ctx, prefix| {
            let cache_key =
                CompletionCache::make_key(&["cachetest".to_string()], prefix, ctx.flags());

            // Check cache first
            if let Some(cached) = cache_clone.get(&cache_key) {
                return Ok(cached);
            }

            // Simulate expensive operation
            count_clone.fetch_add(1, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(10));

            let result = CompletionResult::new().add(format!("expensive-{prefix}"));

            // Cache the result
            cache_clone.put(cache_key, result.clone());

            Ok(result)
        })
        .build();

    let ctx = Context::new(vec!["cachetest".to_string()]);

    // First call should be expensive
    let start = Instant::now();
    let result1 = app.get_completions(&ctx, "test", None).unwrap();
    let first_duration = start.elapsed();
    assert!(first_duration >= Duration::from_millis(10));
    assert_eq!(result1.values, vec!["expensive-test"]);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    // Second call should be cached
    let start = Instant::now();
    let result2 = app.get_completions(&ctx, "test", None).unwrap();
    let second_duration = start.elapsed();
    assert!(second_duration < Duration::from_millis(5));
    assert_eq!(result2.values, vec!["expensive-test"]);
    assert_eq!(call_count.load(Ordering::SeqCst), 1); // No additional calls

    // Different prefix should not be cached
    let result3 = app.get_completions(&ctx, "other", None).unwrap();
    assert_eq!(result3.values, vec!["expensive-other"]);
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_completion_performance_with_many_items() {
    let app = CommandBuilder::new("perftest")
        .arg_completion(|_ctx, prefix| {
            // Generate many items
            let items: Vec<String> = (0..1000)
                .map(|i| format!("item{i:04}"))
                .filter(|item| item.starts_with(prefix))
                .collect();

            Ok(CompletionResult::new().extend(items))
        })
        .build();

    let ctx = Context::new(vec!["perftest".to_string()]);

    // Measure performance with different prefixes
    let start = Instant::now();
    let result = app.get_completions(&ctx, "", None).unwrap();
    let full_duration = start.elapsed();
    assert_eq!(result.values.len(), 1000);

    let start = Instant::now();
    let result = app.get_completions(&ctx, "item000", None).unwrap();
    let filtered_duration = start.elapsed();
    assert_eq!(result.values.len(), 10); // item0000 through item0009

    // Filtered should be faster than full
    assert!(filtered_duration < full_duration);

    // Both should complete quickly
    assert!(full_duration < Duration::from_millis(100));
    assert!(filtered_duration < Duration::from_millis(50));
}

#[test]
fn test_completion_thread_safety() {
    use std::sync::Mutex;

    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = Arc::clone(&results);

    let app = CommandBuilder::new("threadtest")
        .arg_completion(move |_ctx, prefix| {
            let thread_id = thread::current().id();
            results_clone.lock().unwrap().push(format!("{thread_id:?}"));
            Ok(CompletionResult::new().add(format!("thread-{prefix}")))
        })
        .build();

    let app = Arc::new(app);
    let mut handles = vec![];

    // Spawn multiple threads doing completions
    for i in 0..5 {
        let app_clone = Arc::clone(&app);
        let handle = thread::spawn(move || {
            let ctx = Context::new(vec!["threadtest".to_string()]);
            for j in 0..10 {
                let prefix = format!("{i}-{j}");
                let result = app_clone.get_completions(&ctx, &prefix, None).unwrap();
                assert_eq!(result.values, vec![format!("thread-{prefix}")]);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all completions executed
    assert_eq!(results.lock().unwrap().len(), 50); // 5 threads * 10 completions each
}
