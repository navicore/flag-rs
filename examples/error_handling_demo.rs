//! Demonstrates enhanced error handling with colored output and suggestions
//!
//! This example shows how flag-rs provides helpful error messages with:
//! - Clear error formatting
//! - Command suggestions for typos
//! - Helpful hints for common mistakes
//! - Validation error details

use flag_rs::validator::ArgValidator;
use flag_rs::{CommandBuilder, Error, Flag, FlagType};

fn main() {
    let app = CommandBuilder::new("error-demo")
        .short("Demonstrates error handling")
        .long("This example shows various error scenarios and how flag-rs handles them")
        .subcommand(
            CommandBuilder::new("deploy")
                .short("Deploy the application")
                .args(ArgValidator::ExactArgs(1))
                .run(|ctx| {
                    let env = ctx.args().first().unwrap();
                    if !["dev", "staging", "production"].contains(&env.as_str()) {
                        return Err(Error::Validation(format!(
                            "invalid environment '{}', must be one of: dev, staging, production",
                            env
                        )));
                    }
                    println!("Deploying to {}", env);
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("serve")
                .short("Start the server")
                .flag(
                    Flag::new("port")
                        .short('p')
                        .usage("Port to listen on")
                        .value_type(FlagType::Int)
                        .required(),
                )
                .run(|ctx| {
                    let port = ctx.flag("port").unwrap();
                    println!("Server starting on port {}", port);
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("migrate")
                .short("Run database migrations")
                .subcommand(
                    CommandBuilder::new("up")
                        .short("Run pending migrations")
                        .run(|_| {
                            println!("Running migrations...");
                            Ok(())
                        })
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("down")
                        .short("Rollback migrations")
                        .args(ArgValidator::MaximumArgs(1))
                        .run(|ctx| {
                            let steps = ctx
                                .args()
                                .first()
                                .and_then(|s| s.parse::<u32>().ok())
                                .unwrap_or(1);
                            println!(
                                "Rolling back {} migration{}",
                                steps,
                                if steps == 1 { "" } else { "s" }
                            );
                            Ok(())
                        })
                        .build(),
                )
                .build(),
        )
        .subcommand(
            CommandBuilder::new("config")
                .short("Manage configuration")
                .aliases(vec!["cfg", "conf"])
                .run(|_| {
                    println!("Configuration management");
                    Ok(())
                })
                .build(),
        )
        .build();

    println!("=== Error Handling Demo ===\n");
    println!("This demo shows how flag-rs handles various error scenarios.\n");
    println!("Try these commands to see different error types:\n");
    println!(
        "1. {} deploi              (typo - will show suggestions)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "2. {} deploy              (missing required argument)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "3. {} deploy dev staging  (too many arguments)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "4. {} deploy test         (invalid environment)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "5. {} serve               (missing required flag)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "6. {} serve -p abc        (invalid flag value)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "7. {} migrate             (subcommand required)",
        std::env::args().next().unwrap_or_default()
    );
    println!(
        "8. {} confi               (typo - will suggest 'config')",
        std::env::args().next().unwrap_or_default()
    );
    println!("\n---\n");

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
