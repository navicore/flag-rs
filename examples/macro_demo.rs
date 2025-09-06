//! Demonstrates the simplified macro API for creating CLI applications
//!
//! This example shows how to use the new declarative macros to reduce
//! boilerplate when defining commands, flags, and completions.

use flag_rs::{CommandBuilder, Shell, completion, flag, flags};

fn main() {
    let app = build_app();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn build_app() -> flag_rs::Command {
    // Define completion functions using the completion! macro
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
        environments {
            completions: ["dev", "staging", "prod"]
        }
    }

    // Create flags using the simplified flag! macro
    let verbose_flag = flag!(verbose(v): bool, default = false, usage = "Enable verbose output");
    let log_level_flag = flag!(log_level(l): string, default = "info", usage = "Set the logging level", completion = log_levels());
    let port_flag = flag!(port(p): int, default = 8080, usage = "Port to listen on");
    let config_flag = flag!(config(c): string, usage = "Configuration file", required = true);

    // Or create multiple flags at once using flags!
    let additional_flags = flags![
        timeout(t): int, default = 30, usage = "Request timeout in seconds";
        env(e): string, default = "dev", usage = "Target environment", completion = environments();
        dry_run: bool, default = false, usage = "Show what would be done without executing";
    ];

    CommandBuilder::new("macro_demo")
        .short("A demo CLI application using macros")
        .long(
            "This example demonstrates the new declarative macro API that makes \
               creating CLI applications much more readable and concise.",
        )
        .flag(verbose_flag)
        .flag(log_level_flag)
        .flag(port_flag)
        .flag(config_flag)
        .flags(additional_flags)
        .subcommand(build_serve_command())
        .subcommand(build_deploy_command())
        .subcommand(build_completion_command())
        .run(|ctx| {
            // Type-safe context access (still uses the original API for now)
            let verbose = ctx.flag_bool("verbose").unwrap_or(false);
            let log_level = ctx.flag_str_or("log_level", "info");
            let port = ctx.flag_int("port").unwrap_or(8080);
            let config = ctx.flag("config").expect("Config is required");

            if verbose {
                println!("Verbose mode enabled");
                println!("Log level: {}", log_level);
                println!("Port: {}", port);
                println!("Config file: {}", config);
            }

            println!("Application started successfully!");
            Ok(())
        })
        .build()
}

fn build_serve_command() -> flag_rs::Command {
    CommandBuilder::new("serve")
        .short("Start the web server")
        .long("Start the web server with the specified configuration")
        .flags(flags![
            workers(w): int, default = 4, usage = "Number of worker threads";
            bind(b): string, default = "0.0.0.0", usage = "Address to bind to";
            tls: bool, default = false, usage = "Enable TLS";
        ])
        .run(|ctx| {
            let workers = ctx.flag_int("workers").unwrap_or(4);
            let bind = ctx.flag_str_or("bind", "0.0.0.0");
            let tls = ctx.flag_bool("tls").unwrap_or(false);

            println!("Starting server:");
            println!("  Workers: {}", workers);
            println!("  Bind address: {}", bind);
            println!("  TLS enabled: {}", tls);

            Ok(())
        })
        .build()
}

fn build_deploy_command() -> flag_rs::Command {
    completion! {
        deploy_targets {
            completions: [
                ("kubernetes", "Deploy to Kubernetes cluster"),
                ("docker", "Deploy as Docker container"),
                ("lambda", "Deploy as AWS Lambda function"),
            ],
            help: "Available deployment targets"
        }
    }

    CommandBuilder::new("deploy")
        .short("Deploy the application")
        .long("Deploy the application to the specified target environment")
        .flag(flag!(target: string, usage = "Deployment target", required = true, completion = deploy_targets()))
        .flags(flags![
            replicas(r): int, default = 1, usage = "Number of replicas";
            namespace(n): string, default = "default", usage = "Kubernetes namespace";
            force: bool, default = false, usage = "Force deployment even if validation fails";
        ])
        .run(|ctx| {
            let target = ctx.flag("target").expect("Target is required");
            let replicas = ctx.flag_int("replicas").unwrap_or(1);
            let namespace = ctx.flag_str_or("namespace", "default");
            let force = ctx.flag_bool("force").unwrap_or(false);

            println!("Deploying application:");
            println!("  Target: {}", target);
            println!("  Replicas: {}", replicas);
            println!("  Namespace: {}", namespace);
            println!("  Force: {}", force);

            Ok(())
        })
        .build()
}

fn build_completion_command() -> flag_rs::Command {
    CommandBuilder::new("completion")
        .short("Generate shell completion scripts")
        .long(
            "Generate shell completion scripts for macro_demo.\n\n\
             To load completions:\n\n\
             Bash:\n  $ source <(macro_demo completion bash)\n\n\
             Zsh:\n  $ source <(macro_demo completion zsh)\n\n\
             Fish:\n  $ macro_demo completion fish | source\n\n\
             To load completions for each session, execute once:\n\
             $ macro_demo completion zsh > \"${fpath[1]}/_macro_demo\"",
        )
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

            // Recreate the root command for completion generation
            let root = build_app();
            println!("{}", root.generate_completion(shell));

            Ok(())
        })
        .build()
}
