//! # Flag-rs - A Cobra-inspired CLI Framework for Rust
//!
//! Flag-rs is a powerful command-line interface framework for Rust that brings the best features
//! of Go's Cobra library to the Rust ecosystem. The key differentiator is **dynamic runtime
//! completions** - unlike other Rust CLI frameworks, Flag-rs can generate completions based on
//! current system state, making it perfect for tools like `kubectl` that need to complete
//! resource names from a live API.
//!
//! ## Key Features
//!
//! - **Dynamic Completions**: Generate completions at runtime based on application state
//! - **Zero Dependencies**: Pure Rust implementation with no external crates
//! - **Subcommand Support**: Organize complex CLIs with nested subcommands
//! - **Flag Inheritance**: Global flags automatically available to all subcommands
//! - **Shell Completion**: Generate completion scripts for bash, zsh, and fish
//! - **Colored Output**: Beautiful help messages with automatic TTY detection
//! - **Flexible Architecture**: Use builder pattern or direct construction
//! - **Advanced Flag Types**: Choice, Range, File, Directory validation
//! - **Flag Constraints**: `RequiredIf`, `ConflictsWith`, `Requires` relationships
//! - **Completion Caching**: Cache expensive completion operations
//! - **Timeout Protection**: Prevent slow completions from hanging
//! - **Memory Optimization**: String interning and efficient data structures
//!
//! ## Quick Start
//!
//! ```rust
//! use flag_rs::{CommandBuilder, Flag, FlagType, FlagValue};
//!
//! let app = CommandBuilder::new("myapp")
//!     .short("A simple CLI application")
//!     .long("This is my awesome command-line application that does great things")
//!     .flag(
//!         Flag::new("verbose")
//!             .short('v')
//!             .usage("Enable verbose output")
//!             .value_type(FlagType::Bool)
//!             .default(FlagValue::Bool(false))
//!     )
//!     .subcommand(
//!         CommandBuilder::new("serve")
//!             .short("Start the server")
//!             .flag(
//!                 Flag::new("port")
//!                     .short('p')
//!                     .usage("Port to listen on")
//!                     .value_type(FlagType::Int)
//!                     .default(FlagValue::Int(8080))
//!             )
//!             .run(|ctx| {
//!                 let verbose = ctx.flag("verbose")
//!                     .and_then(|s| s.parse::<bool>().ok())
//!                     .unwrap_or(false);

//!                 let port = ctx.flag("port")
//!                     .and_then(|s| s.parse::<i64>().ok())
//!                     .unwrap_or(8080);
//!
//!                 if verbose {
//!                     println!("Starting server on port {}", port);
//!                 }
//!                 Ok(())
//!             })
//!             .build()
//!     )
//!     .build();
//!
//! // In main():
//! // let args: Vec<String> = std::env::args().skip(1).collect();
//! // if let Err(e) = app.execute(args) {
//! //     eprintln!("Error: {}", e);
//! //     std::process::exit(1);
//! // }
//! ```
//!
//! ## Dynamic Completions
//!
//! The killer feature that sets Flag-rs apart from other Rust CLI libraries:
//!
//! ```rust
//! use flag_rs::{CommandBuilder, CompletionResult};
//!
//! let cmd = CommandBuilder::new("kubectl")
//!     .subcommand(
//!         CommandBuilder::new("get")
//!             .subcommand(
//!                 CommandBuilder::new("pods")
//!                     .arg_completion(|ctx, prefix| {
//!                         // This runs when the user presses TAB!
//!                         // In a real app, you'd query the Kubernetes API here
//!                         let namespace = ctx.flag("namespace")
//!                             .map(|s| s.as_str())
//!                             .unwrap_or("default");
//!
//!                         let pods = vec!["nginx-abc123", "redis-def456", "postgres-ghi789"];
//!                         Ok(CompletionResult::new().extend(
//!                             pods.into_iter()
//!                                 .filter(|p| p.starts_with(prefix))
//!                                 .map(String::from)
//!                         ))
//!                     })
//!                     .build()
//!             )
//!             .build()
//!     )
//!     .build();
//! ```
//!
//! ## Shell Completion Setup
//!
//! Add a completion command to enable shell completions:
//!
//! ```rust
//! use flag_rs::{CommandBuilder, Shell};
//!
//! fn build_completion_command() -> flag_rs::Command {
//!     CommandBuilder::new("completion")
//!         .short("Generate shell completion script")
//!         .run(|ctx| {
//!             let shell_name = ctx.args().first()
//!                 .ok_or(flag_rs::Error::ArgumentParsing("shell name required".to_string()))?;
//!
//!             // In a real app, get the root command here
//!             // let script = match shell_name.as_str() {
//!             //     "bash" => root_cmd.generate_completion(Shell::Bash),
//!             //     "zsh" => root_cmd.generate_completion(Shell::Zsh),
//!             //     "fish" => root_cmd.generate_completion(Shell::Fish),
//!             //     _ => return Err(flag_rs::Error::ArgumentParsing("unsupported shell".to_string())),
//!             // };
//!             // println!("{}", script);
//!             Ok(())
//!         })
//!         .build()
//! }
//! ```
//!
//! Users can then enable completions:
//!
//! ```bash
//! # Bash
//! source <(myapp completion bash)
//!
//! # Zsh
//! source <(myapp completion zsh)
//!
//! # Fish
//! myapp completion fish | source
//! ```
//!
//! ## Advanced Flag Types
//!
//! Flag-rs now supports advanced flag types with built-in validation:
//!
//! ```rust
//! use flag_rs::{CommandBuilder, Flag, FlagType, FlagValue};
//!
//! let cmd = CommandBuilder::new("config")
//!     .flag(
//!         Flag::new("environment")
//!             .value_type(FlagType::Choice(vec![
//!                 "dev".to_string(),
//!                 "staging".to_string(),
//!                 "prod".to_string()
//!             ]))
//!     )
//!     .flag(
//!         Flag::new("workers")
//!             .value_type(FlagType::Range(1, 16))
//!             .default(FlagValue::Int(4))
//!     )
//!     .flag(
//!         Flag::new("config-file")
//!             .value_type(FlagType::File)
//!     )
//!     .build();
//! ```
//!
//! ## Flag Constraints
//!
//! Define relationships between flags:
//!
//! ```rust
//! use flag_rs::{Flag, FlagType, FlagConstraint};
//!
//! let ssl_cert_flag = Flag::new("ssl-cert")
//!     .value_type(FlagType::File)
//!     .constraint(FlagConstraint::RequiredIf("ssl".to_string()));
//!
//! let json_flag = Flag::new("json")
//!     .value_type(FlagType::Bool)
//!     .constraint(FlagConstraint::ConflictsWith(vec!["xml".to_string()]));
//! ```
//!
//! ## Performance Features
//!
//! ### Completion Caching
//!
//! Cache expensive completion operations:
//!
//! ```rust,ignore
//! use flag_rs::completion_cache::CompletionCache;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! let cache = Arc::new(CompletionCache::new(Duration::from_secs(5)));
//! ```
//!
//! ### Timeout Protection
//!
//! Prevent slow completions from hanging:
//!
//! ```rust,ignore
//! use flag_rs::completion_timeout::make_timeout_completion;
//! use std::time::Duration;
//!
//! let safe_completion = make_timeout_completion(
//!     Duration::from_millis(100),
//!     expensive_completion_fn
//! );
//! ```
//!
//! ## Modular Command Structure
//!
//! For larger applications, Flag-rs supports a modular architecture:
//!
//! ```rust,ignore
//! // src/commands/mod.rs
//! pub fn register_commands(root: &mut flag_rs::Command) {
//!     // Register each command module
//!     serve::register(root);
//!     config::register(root);
//!     migrate::register(root);
//! }
//!
//! // src/commands/serve.rs
//! use flag_rs::{CommandBuilder, Flag, FlagType};
//!
//! pub fn register(parent: &mut flag_rs::Command) {
//!     let cmd = CommandBuilder::new("serve")
//!         .short("Start the application server")
//!         .flag(
//!             Flag::new("port")
//!                 .short('p')
//!                 .usage("Port to bind to")
//!                 .value_type(FlagType::Int)
//!         )
//!         .run(|ctx| {
//!             // Server implementation
//!             Ok(())
//!         })
//!         .build();
//!
//!     parent.add_command(cmd);
//! }
//! ```
//!
//! ## Error Handling
//!
//! Flag-rs uses idiomatic Rust error handling:
//!
//! ```rust
//! use flag_rs::{CommandBuilder, Error};
//!
//! let cmd = CommandBuilder::new("deploy")
//!     .run(|ctx| {
//!         let env = ctx.args().first()
//!             .ok_or(Error::ArgumentParsing("environment required".to_string()))?;
//!
//!         if env != "production" && env != "staging" {
//!             return Err(Error::Validation(
//!                 format!("unknown environment: {}", env)
//!             ));
//!         }
//!
//!         Ok(())
//!     })
//!     .build();
//! ```

/// Color support for terminal output
pub mod color;

/// Core command structures and execution
pub mod command;

/// Dynamic completion support
pub mod completion;

/// Runtime context for command execution
pub mod context;

/// Error types and result handling
pub mod error;

/// Flag parsing and value types
pub mod flag;

/// Shell completion script generation
pub mod shell;

/// Completion format handling
pub mod completion_format;

/// Completion caching for performance
pub mod completion_cache;

/// Completion timeout handling
pub mod completion_timeout;

/// Terminal utilities for enhanced CLI output
pub mod terminal;

/// Argument validation for commands
pub mod validator;

/// Command and flag suggestion support
pub mod suggestion;

/// ActiveHelp system for contextual hints
pub mod active_help;

/// Memory-efficient completion items
pub mod completion_item;

/// Memory-optimized completion results
pub mod completion_optimized;

/// String interning pool for reducing memory usage
pub mod string_pool;

/// Memory-optimized flag parsing
pub mod parse_optimized;

// Re-export main types for convenience
pub use command::{Command, CommandBuilder};
pub use completion::{CompletionFunc, CompletionResult};
pub use completion_cache::CompletionCache;
pub use context::Context;
pub use error::{Error, Result};
pub use flag::{Flag, FlagConstraint, FlagType, FlagValue};
pub use shell::Shell;
pub use validator::ArgValidator;
