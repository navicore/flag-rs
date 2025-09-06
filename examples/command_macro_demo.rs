//! Demonstrates the comprehensive command! macro from flag-rs Phase II
//!
//! This example shows how to use the new command! macro to eliminate
//! CommandBuilder boilerplate while maintaining all functionality including:
//! - Flags with defaults and completions
//! - Subcommands with their own flags
//! - Dynamic and static completions
//! - Clean, readable macro syntax

use flag_rs::{Shell, command, completion, flag};
use std::fs;

fn main() {
    let app = build_main_app();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn build_main_app() -> flag_rs::Command {
    // Define completions using both static and dynamic patterns
    completion! {
        log_levels {
            completions: [
                ("debug", "Show all messages including debug"),
                ("info", "Show informational messages and above"),
                ("warn", "Show warnings and errors only"),
                ("error", "Show errors only"),
            ],
            help: "Available log levels"
        }
    }

    completion! {
        config_files {
            dynamic: |_ctx, prefix| {
                let mut result = flag_rs::CompletionResult::new();

                // Look for config files in current directory
                if let Ok(entries) = fs::read_dir(".") {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if (name.ends_with(".toml") || name.ends_with(".yaml") || name.ends_with(".json")) && name.starts_with(prefix) {
                                result = result.add_with_description(
                                    name.to_string(),
                                    "Configuration file".to_string()
                                );
                            }
                        }
                    }
                }

                Ok(result)
            },
            help: "Configuration files in current directory"
        }
    }

    // Main application using command! macro - eliminates all CommandBuilder verbosity!
    command! {
        command_macro_demo {
            short: "Phase II command macro demonstration",
            long: "This example demonstrates the new command! macro which completely eliminates CommandBuilder boilerplate while supporting all features including flags, subcommands, and completions.",

            flags: [
                verbose(v): bool = false, usage = "Enable verbose output";
                config(c): string = "config.toml", usage = "Configuration file", completion = config_files();
                log_level(l): string = "info", usage = "Set the logging level", completion = log_levels();
                workers(w): int = 4, usage = "Number of worker threads";
            ],

            subcommands: [build_serve, build_process, build_completion],

            run: |ctx| {
                let verbose = ctx.flag_bool("verbose").unwrap_or(false);
                let config = ctx.flag_str_or("config", "config.toml");
                let log_level = ctx.flag_str_or("log_level", "info");
                let workers = ctx.flag_int("workers").unwrap_or(4);

                if verbose {
                    println!("🚀 Phase II Command Macro Demo");
                    println!("Configuration: {}", config);
                    println!("Log level: {}", log_level);
                    println!("Workers: {}", workers);
                    println!();
                    println!("This command was defined with the command! macro, eliminating");
                    println!("all CommandBuilder boilerplate while maintaining full functionality!");
                } else {
                    println!("Phase II command macro demo running (use --verbose for details)");
                }

                Ok(())
            }
        }
    }

    command_macro_demo()
}

// Subcommands also use the command! macro
fn build_serve() -> flag_rs::Command {
    completion! {
        bind_addresses {
            completions: [
                ("0.0.0.0", "Listen on all interfaces"),
                ("127.0.0.1", "Listen on localhost only"),
                ("::1", "Listen on IPv6 localhost"),
            ],
            help: "Available bind addresses"
        }
    }

    command! {
        serve {
            short: "Start the web server",
            long: "Start the web server with the specified configuration and options",

            flags: [
                port(p): int = 8080, usage = "Port to listen on";
                bind(b): string = "0.0.0.0", usage = "Address to bind to", completion = bind_addresses();
                tls: bool = false, usage = "Enable TLS/HTTPS";
                reload: bool = false, usage = "Enable auto-reload on file changes";
            ],

            run: |ctx| {
                let port = ctx.flag_int("port").unwrap_or(8080);
                let bind = ctx.flag_str_or("bind", "0.0.0.0");
                let tls = ctx.flag_bool("tls").unwrap_or(false);
                let reload = ctx.flag_bool("reload").unwrap_or(false);

                println!("🌐 Starting web server:");
                println!("  Address: {}:{}", bind, port);
                println!("  TLS: {}", if tls { "enabled" } else { "disabled" });
                println!("  Auto-reload: {}", if reload { "enabled" } else { "disabled" });

                if tls {
                    println!("  🔒 HTTPS server ready at https://{}:{}", bind, port);
                } else {
                    println!("  🔓 HTTP server ready at http://{}:{}", bind, port);
                }

                Ok(())
            }
        }
    }

    serve()
}

fn build_process() -> flag_rs::Command {
    completion! {
        process_signals {
            completions: [
                ("SIGTERM", "Graceful termination"),
                ("SIGKILL", "Force termination"),
                ("SIGUSR1", "User-defined signal 1"),
                ("SIGUSR2", "User-defined signal 2"),
            ],
            help: "Available process signals"
        }
    }

    command! {
        process {
            short: "Process management operations",
            long: "Manage application processes with various operations like start, stop, restart, and signal handling",

            flags: [
                signal(s): string = "SIGTERM", usage = "Signal to send", completion = process_signals();
                timeout(t): int = 30, usage = "Timeout in seconds";
                force: bool = false, usage = "Force operation (no confirmation)";
            ],

            run: |ctx| {
                let signal = ctx.flag_str_or("signal", "SIGTERM");
                let timeout = ctx.flag_int("timeout").unwrap_or(30);
                let force = ctx.flag_bool("force").unwrap_or(false);

                println!("🔧 Process Management:");
                println!("  Signal: {}", signal);
                println!("  Timeout: {}s", timeout);
                println!("  Force: {}", force);

                if !force {
                    println!("  ⚠️  Use --force to actually send the signal");
                    println!("  (This is just a demo - no actual signal sent)");
                } else {
                    println!("  ✅ Signal {} would be sent (demo mode)", signal);
                }

                Ok(())
            }
        }
    }

    process()
}

fn build_completion() -> flag_rs::Command {
    command! {
        completion {
            short: "Generate shell completion scripts",
            long: "Generate shell completion scripts that work with all macro-defined completions",

            run: |ctx| {
                let shell_name = ctx.args().first().ok_or_else(|| {
                    flag_rs::Error::ArgumentParsing(
                        "shell name required (bash, zsh, or fish)".to_string(),
                    )
                })?;

                let shell = match shell_name.as_str() {
                    "bash" => Shell::Bash,
                    "zsh" => Shell::Zsh,
                    "fish" => Shell::Fish,
                    _ => {
                        return Err(flag_rs::Error::ArgumentParsing(format!(
                            "unsupported shell: {}",
                            shell_name
                        )));
                    }
                };

                let root = build_main_app();
                println!("{}", root.generate_completion(shell));

                Ok(())
            }
        }
    }

    completion()
}
