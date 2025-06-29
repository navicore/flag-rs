//! Demo of enhanced error messages with helpful suggestions
//!
//! Run with: cargo run --example `enhanced_errors_demo` -- <args>

use flag_rs::{CommandBuilder, Flag, FlagConstraint, FlagType};

fn main() {
    let app = CommandBuilder::new("errordemo")
        .short("Demo of enhanced error messages")
        .flag(
            Flag::new("verbose")
                .short('v')
                .value_type(FlagType::Bool)
                .usage("Enable verbose output"),
        )
        .flag(
            Flag::new("workers")
                .short('w')
                .value_type(FlagType::Range(1, 10))
                .usage("Number of worker threads"),
        )
        .flag(
            Flag::new("format")
                .short('f')
                .value_type(FlagType::Choice(vec![
                    "json".to_string(),
                    "yaml".to_string(),
                    "xml".to_string(),
                    "toml".to_string(),
                ]))
                .usage("Output format"),
        )
        .flag(
            Flag::new("config")
                .short('c')
                .value_type(FlagType::File)
                .usage("Configuration file"),
        )
        .flag(
            Flag::new("outdir")
                .short('o')
                .value_type(FlagType::Directory)
                .usage("Output directory"),
        )
        .flag(
            Flag::new("ssl")
                .value_type(FlagType::Bool)
                .usage("Enable SSL"),
        )
        .flag(
            Flag::new("ssl-cert")
                .value_type(FlagType::File)
                .constraint(FlagConstraint::RequiredIf("ssl".to_string()))
                .usage("SSL certificate file"),
        )
        .flag(
            Flag::new("json")
                .value_type(FlagType::Bool)
                .constraint(FlagConstraint::ConflictsWith(vec!["xml".to_string()]))
                .usage("Output in JSON format"),
        )
        .flag(
            Flag::new("xml")
                .value_type(FlagType::Bool)
                .usage("Output in XML format"),
        )
        .flag(
            Flag::new("required-flag")
                .value_type(FlagType::String)
                .required()
                .usage("This flag is required"),
        )
        .subcommand(
            CommandBuilder::new("process")
                .short("Process data")
                .run(|ctx| {
                    println!("Processing with flags: {:?}", ctx.flags());
                    Ok(())
                })
                .build(),
        )
        .build();

    println!("Try these commands to see enhanced error messages:\n");
    println!("  # Boolean parsing error");
    println!("  cargo run --example enhanced_errors_demo -- --verbose=maybe\n");

    println!("  # Integer parsing error");
    println!("  cargo run --example enhanced_errors_demo -- --workers=abc\n");

    println!("  # Range validation error");
    println!("  cargo run --example enhanced_errors_demo -- --workers=20\n");

    println!("  # Choice validation error");
    println!("  cargo run --example enhanced_errors_demo -- --format=csv\n");

    println!("  # File not found error");
    println!("  cargo run --example enhanced_errors_demo -- --config=/tmp/nonexistent.conf\n");

    println!("  # Directory not found error");
    println!("  cargo run --example enhanced_errors_demo -- --outdir=/tmp/nonexistent_dir\n");

    println!("  # Flag constraint error (RequiredIf)");
    println!("  cargo run --example enhanced_errors_demo -- --ssl --required-flag=test process\n");

    println!("  # Flag constraint error (ConflictsWith)");
    println!(
        "  cargo run --example enhanced_errors_demo -- --json --xml --required-flag=test process\n"
    );

    println!("  # Required flag error");
    println!("  cargo run --example enhanced_errors_demo -- process\n");

    println!("  # Unknown command error");
    println!("  cargo run --example enhanced_errors_demo -- --required-flag=test proces\n");

    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        println!("\n{}", "=".repeat(60));
        println!("Running with args: {:?}\n", args);

        if let Err(e) = app.execute(args) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
