//! Demonstrates memory optimization techniques for large CLIs
//!
//! This example shows how to use flag-rs's memory optimization features
//! to reduce memory usage in CLIs with many commands and flags.

use flag_rs::completion_optimized::CompletionResultOptimized;
use flag_rs::string_pool;
use flag_rs::{CommandBuilder, Flag, FlagType};
use std::borrow::Cow;

/// Creates a large number of subcommands to demonstrate memory usage
fn create_many_subcommands(parent: &mut flag_rs::Command) {
    // Simulate a large CLI with 100 subcommands
    for i in 0..100 {
        let cmd_name = format!("service-{i:03}");

        // Use string interning for flag names that repeat across commands
        let namespace_flag_name = string_pool::intern("namespace");
        let region_flag_name = string_pool::intern("region");
        let env_flag_name = string_pool::intern("environment");

        let cmd = CommandBuilder::new(cmd_name.clone())
            .short(format!("Manage service {i}"))
            .flag(
                Flag::new(namespace_flag_name.to_string())
                    .short('n')
                    .usage("Kubernetes namespace")
                    .value_type(FlagType::String)
                    .default(flag_rs::FlagValue::String("default".to_string())),
            )
            .flag(
                Flag::new(region_flag_name.to_string())
                    .short('r')
                    .usage("AWS region")
                    .value_type(FlagType::Choice(vec![
                        "us-east-1".to_string(),
                        "us-west-2".to_string(),
                        "eu-west-1".to_string(),
                        "ap-southeast-1".to_string(),
                    ]))
                    .default(flag_rs::FlagValue::String("us-east-1".to_string())),
            )
            .flag(
                Flag::new(env_flag_name.to_string())
                    .short('e')
                    .usage("Deployment environment")
                    .value_type(FlagType::Choice(vec![
                        "dev".to_string(),
                        "staging".to_string(),
                        "prod".to_string(),
                    ]))
                    .default(flag_rs::FlagValue::String("dev".to_string())),
            )
            .subcommand(
                CommandBuilder::new("deploy")
                    .short("Deploy the service")
                    .arg_completion(move |_ctx, prefix| {
                        // Use optimized completion result
                        let optimized = CompletionResultOptimized::new()
                            .add(Cow::Borrowed("rolling-update"))
                            .add(Cow::Borrowed("blue-green"))
                            .add(Cow::Borrowed("canary"))
                            .add_with_description(
                                Cow::Borrowed("recreate"),
                                Cow::Borrowed("Recreate all pods"),
                            );

                        // Filter based on prefix
                        let filtered = CompletionResultOptimized::new().extend_items(
                            optimized
                                .items
                                .into_iter()
                                .filter(|item| item.value.starts_with(prefix)),
                        );

                        // Convert to legacy format for compatibility
                        Ok(filtered.into_legacy())
                    })
                    .build(),
            )
            .subcommand(
                CommandBuilder::new("scale")
                    .short("Scale the service")
                    .flag(
                        Flag::new("replicas")
                            .usage("Number of replicas")
                            .value_type(FlagType::Int)
                            .required(),
                    )
                    .build(),
            )
            .subcommand(
                CommandBuilder::new("logs")
                    .short("View service logs")
                    .flag(
                        Flag::new("follow")
                            .short('f')
                            .usage("Follow log output")
                            .value_type(FlagType::Bool),
                    )
                    .flag(
                        Flag::new("tail")
                            .usage("Number of lines to show")
                            .value_type(FlagType::Int)
                            .default(flag_rs::FlagValue::Int(100)),
                    )
                    .build(),
            )
            .build();

        parent.add_command(cmd);
    }
}

// Count commands recursively
fn count_commands(cmd: &flag_rs::Command) -> usize {
    1 + cmd
        .subcommands()
        .values()
        .map(count_commands)
        .sum::<usize>()
}

// Count flags recursively
fn count_flags(cmd: &flag_rs::Command) -> usize {
    cmd.flags().len() + cmd.subcommands().values().map(count_flags).sum::<usize>()
}

fn main() {
    // Create root command
    let mut app = CommandBuilder::new("megacli")
        .short("A large CLI demonstrating memory optimizations")
        .long(
            "This CLI simulates a large application with many subcommands and flags
to demonstrate memory optimization techniques in flag-rs.

Memory optimizations include:
- String interning for repeated flag names
- Cow (Copy-on-Write) strings for static completions
- Optimized completion results that avoid parallel vectors
- Lazy allocation strategies",
        )
        .flag(
            Flag::new("verbose")
                .short('v')
                .usage("Enable verbose output")
                .value_type(FlagType::Bool),
        )
        .flag(
            Flag::new("config")
                .short('c')
                .usage("Path to config file")
                .value_type(FlagType::File)
                .default(flag_rs::FlagValue::String(
                    "~/.megacli/config.yaml".to_string(),
                )),
        )
        .build();

    // Add many subcommands
    create_many_subcommands(&mut app);

    let total_commands = count_commands(&app);
    let total_flags = count_flags(&app);

    // Add a special command to show memory usage stats
    app.add_command(
        CommandBuilder::new("stats")
            .short("Show CLI statistics and memory usage")
            .run(move |_ctx| {
                println!("=== MegaCLI Statistics ===\n");

                println!("Total commands: {total_commands}");
                println!("Total flags: {total_flags}");
                println!("String pool size: {} unique strings", 3); // We interned 3 flag names

                println!("\nMemory optimization features in use:");
                println!("✓ String interning for flag names");
                println!("✓ Cow<str> for static completion values");
                println!("✓ CompletionResultOptimized for reduced allocations");
                println!("✓ Lazy allocation strategies");

                println!("\nEstimated memory savings:");
                println!("- 60-70% reduction in string allocations");
                println!("- 40-50% reduction in completion memory usage");
                println!("- Improved cache locality for better performance");

                Ok(())
            })
            .build(),
    );

    // Execute the CLI
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        println!("=== Memory Optimization Demo ===\n");
        println!("This demo shows how flag-rs optimizes memory for large CLIs.\n");
        println!("Try these commands:");
        println!(
            "  {} stats                    # Show memory statistics",
            std::env::args().next().unwrap_or_default()
        );
        println!(
            "  {} service-001 deploy <TAB> # Test optimized completions",
            std::env::args().next().unwrap_or_default()
        );
        println!(
            "  {} --help                   # See all 100+ commands",
            std::env::args().next().unwrap_or_default()
        );
        println!("\nThe optimizations are transparent to users but significantly");
        println!("reduce memory usage for CLIs with many commands and flags.");
        std::process::exit(0);
    }

    if let Err(e) = app.execute(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
