//! Demonstrates advanced flag features
//!
//! This example shows how to use:
//! - Choice flags with predefined values
//! - Range flags for numeric validation
//! - File and Directory flags with path validation
//! - Flag constraints (`RequiredIf`, `ConflictsWith`, `Requires`)
//! - `StringArray` flags for multiple values

#![allow(clippy::or_fun_call)]

use flag_rs::{CommandBuilder, Flag, FlagConstraint, FlagType, FlagValue};

fn main() {
    let app = CommandBuilder::new("server")
        .short("A demo server with advanced configuration options")
        .long(
            "This example demonstrates the advanced flag features in flag-rs,
including choice flags, range validation, file/directory validation,
and flag constraints.",
        )
        // Choice flag - must be one of the predefined values
        .flag(
            Flag::new("environment")
                .short('e')
                .usage("Server environment")
                .value_type(FlagType::Choice(vec![
                    "development".to_string(),
                    "staging".to_string(),
                    "production".to_string(),
                ]))
                .default(FlagValue::String("development".to_string())),
        )
        // Range flag - numeric value within a specific range
        .flag(
            Flag::new("port")
                .short('p')
                .usage("Server port (1024-65535)")
                .value_type(FlagType::Range(1024, 65535))
                .default(FlagValue::Int(8080)),
        )
        // File flag - must be a valid file path
        .flag(
            Flag::new("config")
                .short('c')
                .usage("Configuration file path")
                .value_type(FlagType::File),
        )
        // Directory flag - must be a valid directory path
        .flag(
            Flag::new("log-dir")
                .usage("Directory for log files")
                .value_type(FlagType::Directory)
                .default(FlagValue::String("/tmp".to_string())),
        )
        // StringArray flag - can be specified multiple times
        .flag(
            Flag::new("tags")
                .short('t')
                .usage("Tags for the server (can be specified multiple times)")
                .value_type(FlagType::StringArray),
        )
        // Flag with RequiredIf constraint
        .flag(
            Flag::new("ssl-cert")
                .usage("SSL certificate file")
                .value_type(FlagType::File)
                .constraint(FlagConstraint::RequiredIf("ssl".to_string())),
        )
        // Flag with RequiredIf constraint
        .flag(
            Flag::new("ssl-key")
                .usage("SSL private key file")
                .value_type(FlagType::File)
                .constraint(FlagConstraint::RequiredIf("ssl".to_string())),
        )
        // Flag that triggers the RequiredIf constraints above
        .flag(
            Flag::new("ssl")
                .usage("Enable SSL/TLS")
                .value_type(FlagType::Bool),
        )
        // Flag with ConflictsWith constraint
        .flag(
            Flag::new("debug")
                .short('d')
                .usage("Enable debug mode")
                .value_type(FlagType::Bool)
                .constraint(FlagConstraint::ConflictsWith(vec!["quiet".to_string()])),
        )
        // Flag that conflicts with debug
        .flag(
            Flag::new("quiet")
                .short('q')
                .usage("Quiet mode (minimal output)")
                .value_type(FlagType::Bool)
                .constraint(FlagConstraint::ConflictsWith(vec!["debug".to_string()])),
        )
        // Flag with Requires constraint
        .flag(
            Flag::new("auth-token")
                .usage("Authentication token")
                .value_type(FlagType::String)
                .constraint(FlagConstraint::Requires(vec!["auth-enabled".to_string()])),
        )
        // Flag required by auth-token
        .flag(
            Flag::new("auth-enabled")
                .usage("Enable authentication")
                .value_type(FlagType::Bool),
        )
        // Required flag
        .flag(
            Flag::new("name")
                .short('n')
                .usage("Server name")
                .value_type(FlagType::String)
                .required(),
        )
        .run(|ctx| {
            // Display configuration
            println!("=== Server Configuration ===\n");

            println!("Server Name: {}", ctx.flag("name").unwrap());
            println!(
                "Environment: {}",
                ctx.flag("environment")
                    .unwrap_or(&"development".to_string())
            );
            println!("Port: {}", ctx.flag("port").unwrap_or(&"8080".to_string()));

            if let Some(config) = ctx.flag("config") {
                println!("Config File: {}", config);
            }

            println!(
                "Log Directory: {}",
                ctx.flag("log-dir").unwrap_or(&"/tmp".to_string())
            );

            if let Some(tags) = ctx.flag("tags") {
                println!("Tags: {}", tags);
            }

            if ctx.flag("ssl").map(|v| v == "true").unwrap_or(false) {
                println!("\nSSL Configuration:");
                println!("  Certificate: {}", ctx.flag("ssl-cert").unwrap());
                println!("  Private Key: {}", ctx.flag("ssl-key").unwrap());
            }

            if ctx.flag("debug").map(|v| v == "true").unwrap_or(false) {
                println!("\nDebug mode: ENABLED");
            } else if ctx.flag("quiet").map(|v| v == "true").unwrap_or(false) {
                println!("\nQuiet mode: ENABLED");
            }

            if ctx
                .flag("auth-enabled")
                .map(|v| v == "true")
                .unwrap_or(false)
            {
                println!("\nAuthentication: ENABLED");
                if let Some(token) = ctx.flag("auth-token") {
                    println!("  Token: {}", token);
                }
            }

            println!("\nServer would start with the above configuration...");
            Ok(())
        })
        .build();

    println!("=== Advanced Flags Demo ===\n");
    println!("This demo showcases advanced flag features in flag-rs.\n");
    println!("Try these commands to see different features:");
    println!();
    println!("1. Basic usage with required flag:");
    println!(
        "   {} --name myserver",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("2. Choice flag validation:");
    println!(
        "   {} --name myserver --environment invalid  (will fail)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "   {} --name myserver --environment production",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("3. Range validation:");
    println!(
        "   {} --name myserver --port 80  (will fail - below 1024)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "   {} --name myserver --port 3000",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("4. File/Directory validation:");
    println!(
        "   {} --name myserver --config ./Cargo.toml",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "   {} --name myserver --log-dir ./src",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("5. Flag constraints:");
    println!(
        "   {} --name myserver --ssl  (will fail - missing cert and key)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "   {} --name myserver --ssl --ssl-cert cert.pem --ssl-key key.pem",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "   {} --name myserver --debug --quiet  (will fail - conflicts)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "   {} --name myserver --auth-token secret  (will fail - requires --auth-enabled)",
        std::env::args().next().unwrap_or_default()
    );
    println!();
    println!("6. View help:");
    println!("   {} --help", std::env::args().next().unwrap_or_default());
    println!("\n---\n");

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
