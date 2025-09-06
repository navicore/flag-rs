//! Demonstrates the dynamic completion support in flag-rs macros Phase II
//!
//! This example shows how to use the enhanced completion! macro with dynamic
//! completion functions for real-world scenarios like active sessions,
//! available files, running processes, etc.

use flag_rs::{CommandBuilder, Shell, completion, flag};
use std::fs;

fn main() {
    let app = build_app();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn build_app() -> flag_rs::Command {
    // Static completion (from Phase I)
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

    // Dynamic completion for active sessions (simulated)
    completion! {
        active_sessions {
            dynamic: |_ctx, prefix| {
                // In a real app, this would query actual active sessions
                let sessions = get_mock_sessions();
                let mut result = flag_rs::CompletionResult::new();

                for (name, info) in sessions {
                    if name.starts_with(prefix) {
                        result = result.add_with_description(name, info);
                    }
                }

                Ok(result)
            },
            help: "Currently active user sessions"
        }
    }

    // Dynamic completion for files in current directory
    completion! {
        local_files {
            dynamic: |_ctx, prefix| {
                let mut result = flag_rs::CompletionResult::new();

                // Read current directory
                if let Ok(entries) = fs::read_dir(".") {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.starts_with(prefix) {
                                let file_type = if entry.path().is_dir() {
                                    "directory"
                                } else {
                                    "file"
                                };
                                result = result.add_with_description(
                                    name.to_string(),
                                    format!("Local {}", file_type)
                                );
                            }
                        }
                    }
                }

                Ok(result)
            }
        }
    }

    // Dynamic completion for running processes (simulated)
    completion! {
        running_processes {
            dynamic: |_ctx, prefix| {
                let processes = get_mock_processes();
                let mut result = flag_rs::CompletionResult::new();

                for (pid, name, desc) in processes {
                    let process_string = format!("{}", pid);
                    if process_string.starts_with(prefix) || name.starts_with(prefix) {
                        result = result.add_with_description(
                            process_string,
                            format!("{} - {}", name, desc)
                        );
                    }
                }

                Ok(result)
            },
            help: "Currently running processes (PID or name)"
        }
    }

    CommandBuilder::new("dynamic_demo")
        .short("Demo of dynamic completion macros")
        .long("This example demonstrates Phase II dynamic completion support")
        .flag(flag!(verbose(v): bool, default = false, usage = "Enable verbose output"))
        .flag(flag!(log_level(l): string, default = "info", usage = "Set log level", completion = log_levels()))
        .flag(flag!(session(s): string, usage = "Target session", completion = active_sessions()))
        .flag(flag!(file(f): string, usage = "Input file", completion = local_files()))
        .flag(flag!(process(p): string, usage = "Target process (PID or name)", completion = running_processes()))
        .subcommand(build_completion_command())
        .run(|ctx| {
            let verbose = ctx.flag_bool("verbose").unwrap_or(false);
            let log_level = ctx.flag_str_or("log_level", "info");

            if verbose {
                println!("Dynamic completion demo running!");
                println!("Log level: {}", log_level);

                if let Some(session) = ctx.flag("session") {
                    println!("Selected session: {}", session);
                }

                if let Some(file) = ctx.flag("file") {
                    println!("Input file: {}", file);
                }

                if let Some(process) = ctx.flag("process") {
                    println!("Target process: {}", process);
                }
            } else {
                println!("Dynamic completion demo (use --verbose for details)");
            }

            Ok(())
        })
        .build()
}

fn build_completion_command() -> flag_rs::Command {
    CommandBuilder::new("completion")
        .short("Generate shell completion scripts")
        .long("Generate shell completion scripts that support dynamic completions")
        .run(|ctx| {
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

            let root = build_app();
            println!("{}", root.generate_completion(shell));

            Ok(())
        })
        .build()
}

// Mock data functions (in real apps, these would query actual systems)

fn get_mock_sessions() -> Vec<(String, String)> {
    vec![
        (
            "user1_session".to_string(),
            "User 1 active session".to_string(),
        ),
        (
            "admin_console".to_string(),
            "Administrator console session".to_string(),
        ),
        (
            "bg_worker_01".to_string(),
            "Background worker session 01".to_string(),
        ),
        (
            "api_client_42".to_string(),
            "API client connection 42".to_string(),
        ),
    ]
}

fn get_mock_processes() -> Vec<(u32, String, String)> {
    vec![
        (1234, "nginx".to_string(), "Web server".to_string()),
        (5678, "postgres".to_string(), "Database server".to_string()),
        (9012, "redis".to_string(), "Cache server".to_string()),
        (
            3456,
            "my_app".to_string(),
            "Main application process".to_string(),
        ),
    ]
}
