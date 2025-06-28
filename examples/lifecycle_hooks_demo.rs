//! Demonstrates lifecycle hooks (`PreRun`, `PostRun`, `PersistentPreRun`, `PersistentPostRun`)
//!
//! This example shows how to use lifecycle hooks that execute before and after
//! command execution, similar to Cobra's hook system.

use flag_rs::{CommandBuilder, Flag, FlagType};
use std::sync::atomic::{AtomicU32, Ordering};

// Counter to track hook execution order
static COUNTER: AtomicU32 = AtomicU32::new(0);

fn log_hook(name: &str) {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("[{:02}] {}", count + 1, name);
}

fn main() {
    let app = CommandBuilder::new("lifecycle-demo")
        .short("Demonstrates lifecycle hooks")
        .long("This example shows how lifecycle hooks execute in the proper order")
        .flag(
            Flag::new("verbose")
                .short('v')
                .usage("Enable verbose output")
                .value_type(FlagType::Bool),
        )
        // Persistent hooks run for this command and all subcommands
        .persistent_pre_run(|ctx| {
            log_hook("Root PersistentPreRun");
            if ctx.flag("verbose").is_some() {
                println!("     -> Verbose mode enabled globally");
            }
            Ok(())
        })
        .persistent_post_run(|ctx| {
            log_hook("Root PersistentPostRun");
            if ctx.flag("verbose").is_some() {
                println!("     -> Cleaning up verbose logging");
            }
            Ok(())
        })
        .subcommand(
            CommandBuilder::new("server")
                .short("Server management commands")
                .persistent_pre_run(|_ctx| {
                    log_hook("Server PersistentPreRun");
                    println!("     -> Initializing server module");
                    Ok(())
                })
                .persistent_post_run(|_ctx| {
                    log_hook("Server PersistentPostRun");
                    println!("     -> Cleaning up server module");
                    Ok(())
                })
                .subcommand(
                    CommandBuilder::new("start")
                        .short("Start the server")
                        .flag(
                            Flag::new("port")
                                .short('p')
                                .usage("Port to listen on")
                                .value_type(FlagType::Int),
                        )
                        .pre_run(|ctx| {
                            log_hook("Start PreRun");
                            let port = ctx
                                .flag("port")
                                .and_then(|s| s.parse::<u16>().ok())
                                .unwrap_or(8080);
                            println!("     -> Validating port {}", port);
                            Ok(())
                        })
                        .run(|ctx| {
                            log_hook("Start Run");
                            let port = ctx
                                .flag("port")
                                .and_then(|s| s.parse::<u16>().ok())
                                .unwrap_or(8080);
                            println!("     -> Starting server on port {}", port);
                            Ok(())
                        })
                        .post_run(|_ctx| {
                            log_hook("Start PostRun");
                            println!("     -> Server started successfully");
                            Ok(())
                        })
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("stop")
                        .short("Stop the server")
                        .pre_run(|_ctx| {
                            log_hook("Stop PreRun");
                            println!("     -> Checking if server is running");
                            Ok(())
                        })
                        .run(|_ctx| {
                            log_hook("Stop Run");
                            println!("     -> Stopping server");
                            Ok(())
                        })
                        .post_run(|_ctx| {
                            log_hook("Stop PostRun");
                            println!("     -> Server stopped");
                            Ok(())
                        })
                        .build(),
                )
                .build(),
        )
        .subcommand(
            CommandBuilder::new("database")
                .short("Database management commands")
                .persistent_pre_run(|_ctx| {
                    log_hook("Database PersistentPreRun");
                    println!("     -> Connecting to database");
                    Ok(())
                })
                .persistent_post_run(|_ctx| {
                    log_hook("Database PersistentPostRun");
                    println!("     -> Closing database connection");
                    Ok(())
                })
                .subcommand(
                    CommandBuilder::new("migrate")
                        .short("Run database migrations")
                        .pre_run(|_ctx| {
                            log_hook("Migrate PreRun");
                            println!("     -> Checking migration status");
                            Ok(())
                        })
                        .run(|_ctx| {
                            log_hook("Migrate Run");
                            println!("     -> Running migrations");
                            Ok(())
                        })
                        .post_run(|_ctx| {
                            log_hook("Migrate PostRun");
                            println!("     -> Migrations completed");
                            Ok(())
                        })
                        .build(),
                )
                .build(),
        )
        .build();

    println!("=== Lifecycle Hooks Demo ===\n");
    println!("This demo shows the execution order of lifecycle hooks.");
    println!("Try these commands to see different hook patterns:\n");
    println!(
        "1. {} server start",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "2. {} -v server start -p 3000",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "3. {} database migrate",
        std::env::args().next().unwrap_or_default()
    );
    println!("\n---\n");

    // Reset counter before execution
    COUNTER.store(0, Ordering::SeqCst);

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("\n=== Hook Execution Summary ===");
    println!("Total hooks executed: {}", COUNTER.load(Ordering::SeqCst));
    println!("\nExecution order:");
    println!("1. Parent PersistentPreRun (root → child)");
    println!("2. Command PreRun");
    println!("3. Command Run");
    println!("4. Command PostRun");
    println!("5. Parent PersistentPostRun (child → root)");
}
