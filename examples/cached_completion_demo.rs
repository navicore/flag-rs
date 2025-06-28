//! Demonstrates completion caching for performance
//!
//! This example shows how to use the completion cache to improve performance
//! for expensive completion operations like API calls or file system scans.

#![allow(clippy::or_fun_call)]

use flag_rs::completion_cache::CompletionCache;
use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Simulates an expensive operation (e.g., API call, file system scan)
fn expensive_completion(prefix: &str) -> Vec<String> {
    // Simulate network latency or expensive computation
    thread::sleep(Duration::from_millis(500));

    // In a real app, this might query a database or API
    let items = vec![
        "nginx-deployment-abc123",
        "nginx-deployment-def456",
        "redis-cache-ghi789",
        "redis-cache-jkl012",
        "postgres-db-mno345",
        "postgres-db-pqr678",
        "mongodb-cluster-stu901",
        "mongodb-cluster-vwx234",
    ];

    items
        .into_iter()
        .filter(|item| item.starts_with(prefix))
        .map(String::from)
        .collect()
}

fn main() {
    // Create a shared cache with 10-second TTL
    let cache = Arc::new(CompletionCache::new(Duration::from_secs(10)));

    let app = CommandBuilder::new("kubectl")
        .short("Kubernetes command-line tool with cached completions")
        .flag(
            Flag::new("namespace")
                .short('n')
                .usage("The namespace scope for this CLI request")
                .value_type(FlagType::String)
                .default(flag_rs::FlagValue::String("default".to_string())),
        )
        .subcommand({
            let cache_clone = Arc::clone(&cache);
            CommandBuilder::new("get")
                .short("Display one or many resources")
                .subcommand({
                    let cache_for_pods = Arc::clone(&cache_clone);
                    CommandBuilder::new("pods")
                        .short("List pods with cached completion")
                        .arg_completion(move |ctx, prefix| {
                            let start = Instant::now();

                            // Generate cache key
                            let cache_key = CompletionCache::make_key(
                                &["kubectl".to_string(), "get".to_string(), "pods".to_string()],
                                prefix,
                                ctx.flags(),
                            );

                            // Try to get from cache first
                            if let Some(cached_result) = cache_for_pods.get(&cache_key) {
                                eprintln!("✓ Cache hit! Completed in {:?}", start.elapsed());
                                return Ok(cached_result);
                            }

                            eprintln!("✗ Cache miss - fetching completions...");

                            // Perform expensive operation
                            let items = expensive_completion(prefix);
                            let mut result = CompletionResult::new();

                            for item in items {
                                result = result.add_with_description(
                                    item.clone(),
                                    format!(
                                        "Pod in namespace {}",
                                        ctx.flag("namespace").unwrap_or(&"default".to_string())
                                    ),
                                );
                            }

                            // Add contextual help
                            if prefix.is_empty() {
                                result =
                                    result.add_help_text("Tip: Start typing to filter pod names");
                            }

                            // Cache the result
                            cache_for_pods.put(cache_key, result.clone());

                            eprintln!(
                                "✓ Completed in {:?} (cached for future use)",
                                start.elapsed()
                            );
                            Ok(result)
                        })
                        .build()
                })
                .subcommand({
                    let cache_for_services = Arc::clone(&cache_clone);
                    CommandBuilder::new("services")
                        .short("List services with cached completion")
                        .arg_completion(move |ctx, prefix| {
                            let cache_key = CompletionCache::make_key(
                                &[
                                    "kubectl".to_string(),
                                    "get".to_string(),
                                    "services".to_string(),
                                ],
                                prefix,
                                ctx.flags(),
                            );

                            if let Some(cached) = cache_for_services.get(&cache_key) {
                                eprintln!("✓ Using cached service completions");
                                return Ok(cached);
                            }

                            // Simulate expensive operation
                            let services = expensive_completion(prefix);
                            let result = CompletionResult::new().extend(services);

                            cache_for_services.put(cache_key, result.clone());
                            Ok(result)
                        })
                        .build()
                })
                .build()
        })
        .build();

    println!("=== Cached Completion Demo ===\n");
    println!("This demo shows how completion caching improves performance.\n");
    println!("To test completions with caching:");
    println!("1. Set up bash completion:");
    println!(
        "   source <({} completion bash)",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("2. Try tab completion multiple times:");
    println!(
        "   {} get pods n<TAB>     # First time: ~500ms (cache miss)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "   {} get pods n<TAB>     # Second time: <1ms (cache hit!)",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("3. Different prefixes have separate cache entries:");
    println!(
        "   {} get pods r<TAB>     # Different prefix = cache miss",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("4. Flags affect the cache key:");
    println!(
        "   {} -n prod get pods n<TAB>  # Different namespace = different cache",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("Note: Cache entries expire after 10 seconds in this demo.\n");
    println!("---\n");

    let args: Vec<String> = std::env::args().skip(1).collect();

    // Handle completion requests
    if std::env::var("KUBECTL_COMPLETE").is_ok() {
        // When testing completions, show cache status
        eprintln!("Cache size: {} entries", cache.size());
    }

    if let Err(e) = app.execute(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
