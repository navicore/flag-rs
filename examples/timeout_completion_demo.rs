//! Demonstrates timeout handling for slow completion operations
//!
//! This example shows how to use completion timeouts to ensure
//! that slow operations don't hang the shell completion experience.

#![allow(clippy::or_fun_call)]

use flag_rs::completion_timeout::{make_timeout_completion, DEFAULT_COMPLETION_TIMEOUT};
use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType};
use std::thread;
use std::time::Duration;

/// Simulates a slow API call or database query
fn slow_completion(prefix: &str, delay_ms: u64) -> Vec<String> {
    thread::sleep(Duration::from_millis(delay_ms));

    vec![
        "slow-result-1",
        "slow-result-2",
        "slow-result-3",
        "slow-match-1",
        "slow-match-2",
    ]
    .into_iter()
    .filter(|item| item.starts_with(prefix))
    .map(String::from)
    .collect()
}

fn main() {
    let app = CommandBuilder::new("cloud-cli")
        .short("Cloud CLI with timeout-protected completions")
        .flag(
            Flag::new("region")
                .short('r')
                .usage("Cloud region")
                .value_type(FlagType::String)
                .default(flag_rs::FlagValue::String("us-east-1".to_string())),
        )
        .subcommand(
            CommandBuilder::new("instances")
                .short("Manage cloud instances")
                .subcommand(
                    CommandBuilder::new("list")
                        .short("List instances (fast completion)")
                        .arg_completion(|_ctx, prefix| {
                            // Fast completion - no timeout needed
                            let instances = vec![
                                "web-server-001",
                                "web-server-002",
                                "db-primary",
                                "db-replica",
                                "cache-node-1",
                                "cache-node-2",
                            ];

                            Ok(CompletionResult::new().extend(
                                instances
                                    .into_iter()
                                    .filter(|i| i.starts_with(prefix))
                                    .map(String::from),
                            ))
                        })
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("describe")
                        .short("Describe instance (slow completion with timeout)")
                        .arg_completion({
                            // Wrap slow completion with timeout
                            make_timeout_completion(
                                Duration::from_millis(500), // 500ms timeout
                                |ctx, prefix| {
                                    eprintln!("Fetching instance details from API...");

                                    // Simulate slow API call (1 second)
                                    let instances = slow_completion(prefix, 1000);

                                    let mut result = CompletionResult::new();
                                    for instance in instances {
                                        result = result.add_with_description(
                                            instance.clone(),
                                            format!(
                                                "Instance in region {}",
                                                ctx.flag("region")
                                                    .unwrap_or(&"us-east-1".to_string())
                                            ),
                                        );
                                    }

                                    if prefix.is_empty() {
                                        result = result.add_help_text(
                                            "Fetching live instance data from cloud API...",
                                        );
                                    }

                                    Ok(result)
                                },
                            )
                        })
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("create")
                        .short("Create instance (very slow completion)")
                        .arg_completion({
                            // Use default timeout (2 seconds)
                            make_timeout_completion(DEFAULT_COMPLETION_TIMEOUT, |_ctx, prefix| {
                                eprintln!("Checking available instance types...");

                                // Simulate very slow operation (3 seconds)
                                let types = slow_completion(prefix, 3000);

                                Ok(CompletionResult::new()
                                    .extend(types)
                                    .add_help_text("Available instance types"))
                            })
                        })
                        .build(),
                )
                .build(),
        )
        .build();

    println!("=== Timeout Completion Demo ===\n");
    println!("This demo shows how timeouts protect against slow completion operations.\n");
    println!("To test timeout handling:");
    println!("1. Set up bash completion:");
    println!(
        "   source <({} completion bash)",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("2. Test different completion speeds:");
    println!();
    println!("   Fast (no timeout needed):");
    println!(
        "   {} instances list <TAB>",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("   Slow (500ms timeout, 1s operation - will timeout):");
    println!(
        "   {} instances describe <TAB>",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("   Very slow (2s timeout, 3s operation - will timeout):");
    println!(
        "   {} instances create <TAB>",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("When a timeout occurs, you'll see:");
    println!("- A warning message about the timeout");
    println!("- Any partial results that were available");
    println!("- Suggestion to use a more specific prefix");
    println!("\n---\n");

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
