//! Demo of enhanced help formatting
//!
//! Run with: cargo run --example `help_formatting_demo` -- --help

use flag_rs::{CommandBuilder, Flag, FlagConstraint, FlagType, FlagValue};

fn main() {
    let app = CommandBuilder::new("helpdemo")
        .short("A demo of enhanced help formatting")
        .long("This is a comprehensive demonstration of the enhanced help formatting system in flag-rs. \
               It shows how help text is beautifully formatted with proper grouping, constraint \
               information, and visual hierarchy.")
        .group_id("utilities")
        .example("helpdemo config --format=yaml --output=config.yaml")
        .example("helpdemo process --workers=4 --verbose data.json")
        .example("helpdemo serve --port=8080 --ssl --ssl-cert=cert.pem")
        .flag(
            Flag::new("verbose")
                .short('v')
                .value_type(FlagType::Bool)
                .usage("Enable verbose output (can be repeated for more verbosity)"),
        )
        .flag(
            Flag::new("config")
                .short('c')
                .value_type(FlagType::File)
                .default(FlagValue::String("~/.config/app.toml".to_string()))
                .usage("Configuration file path"),
        )
        .flag(
            Flag::new("workers")
                .short('w')
                .value_type(FlagType::Range(1, 16))
                .default(FlagValue::Int(4))
                .usage("Number of worker threads"),
        )
        .flag(
            Flag::new("timeout")
                .short('t')
                .value_type(FlagType::Int)
                .default(FlagValue::Int(30))
                .usage("Request timeout in seconds"),
        )
        .subcommand(
            CommandBuilder::new("config")
                .short("Manage configuration")
                .long("The config command allows you to view, edit, and validate configuration files. \
                      It supports multiple output formats and can merge configurations from various sources.")
                .alias("cfg")
                .alias("conf")
                .example("helpdemo config show")
                .example("helpdemo config validate --strict")
                .flag(
                    Flag::new("format")
                        .short('f')
                        .value_type(FlagType::Choice(vec![
                            "json".to_string(),
                            "yaml".to_string(),
                            "toml".to_string(),
                            "ini".to_string(),
                        ]))
                        .default(FlagValue::String("yaml".to_string()))
                        .usage("Configuration file format"),
                )
                .flag(
                    Flag::new("output")
                        .short('o')
                        .value_type(FlagType::String)
                        .required()
                        .usage("Output file path"),
                )
                .flag(
                    Flag::new("merge")
                        .value_type(FlagType::StringArray)
                        .usage("Additional config files to merge"),
                )
                .flag(
                    Flag::new("strict")
                        .value_type(FlagType::Bool)
                        .usage("Enable strict validation mode"),
                )
                .subcommand(
                    CommandBuilder::new("show")
                        .short("Display current configuration")
                        .run(|_| Ok(()))
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("validate")
                        .short("Validate configuration syntax")
                        .run(|_| Ok(()))
                        .build(),
                )
                .build(),
        )
        .subcommand(
            CommandBuilder::new("process")
                .short("Process data files")
                .group_id("operations")
                .flag(
                    Flag::new("input")
                        .short('i')
                        .value_type(FlagType::File)
                        .required()
                        .usage("Input data file"),
                )
                .flag(
                    Flag::new("output")
                        .short('o')
                        .value_type(FlagType::String)
                        .required()
                        .usage("Output file path"),
                )
                .flag(
                    Flag::new("format")
                        .short('f')
                        .value_type(FlagType::Choice(vec![
                            "csv".to_string(),
                            "json".to_string(),
                            "parquet".to_string(),
                        ]))
                        .usage("Output format"),
                )
                .flag(
                    Flag::new("compress")
                        .value_type(FlagType::Bool)
                        .usage("Compress output"),
                )
                .flag(
                    Flag::new("compression-level")
                        .value_type(FlagType::Range(1, 9))
                        .default(FlagValue::Int(6))
                        .constraint(FlagConstraint::RequiredIf("compress".to_string()))
                        .usage("Compression level"),
                )
                .run(|_| Ok(()))
                .build(),
        )
        .subcommand(
            CommandBuilder::new("serve")
                .short("Start the server")
                .group_id("operations")
                .flag(
                    Flag::new("port")
                        .short('p')
                        .value_type(FlagType::Range(1, 65535))
                        .default(FlagValue::Int(8080))
                        .usage("Port to listen on"),
                )
                .flag(
                    Flag::new("host")
                        .short('h')
                        .value_type(FlagType::String)
                        .default(FlagValue::String("127.0.0.1".to_string()))
                        .usage("Host to bind to"),
                )
                .flag(
                    Flag::new("ssl")
                        .value_type(FlagType::Bool)
                        .usage("Enable SSL/TLS"),
                )
                .flag(
                    Flag::new("ssl-cert")
                        .value_type(FlagType::File)
                        .constraint(FlagConstraint::RequiredIf("ssl".to_string()))
                        .usage("SSL certificate file"),
                )
                .flag(
                    Flag::new("ssl-key")
                        .value_type(FlagType::File)
                        .constraint(FlagConstraint::RequiredIf("ssl".to_string()))
                        .usage("SSL private key file"),
                )
                .flag(
                    Flag::new("http2")
                        .value_type(FlagType::Bool)
                        .constraint(FlagConstraint::Requires(vec!["ssl".to_string()]))
                        .usage("Enable HTTP/2 support"),
                )
                .flag(
                    Flag::new("debug")
                        .short('d')
                        .value_type(FlagType::Bool)
                        .constraint(FlagConstraint::ConflictsWith(vec!["production".to_string()]))
                        .usage("Enable debug mode"),
                )
                .flag(
                    Flag::new("production")
                        .value_type(FlagType::Bool)
                        .usage("Enable production mode"),
                )
                .run(|_| Ok(()))
                .build(),
        )
        .build();

    println!("This example demonstrates enhanced help formatting.");
    println!("Try these commands to see different help layouts:\n");
    println!("  # Main help with command groups");
    println!("  cargo run --example help_formatting_demo -- --help\n");

    println!("  # Subcommand help with required flags");
    println!("  cargo run --example help_formatting_demo -- config --help\n");

    println!("  # Subcommand with flag constraints");
    println!("  cargo run --example help_formatting_demo -- serve --help\n");

    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        if let Err(e) = app.execute(args) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
