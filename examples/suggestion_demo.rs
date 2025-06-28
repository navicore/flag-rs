//! Demonstrates "Did You Mean" suggestions
use flag_rs::CommandBuilder;

fn main() {
    let app = CommandBuilder::new("suggestion-demo")
        .short("Demonstrates command suggestions")
        .subcommand(
            CommandBuilder::new("start")
                .short("Start the service")
                .run(|_| {
                    println!("Starting service...");
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("stop")
                .short("Stop the service")
                .run(|_| {
                    println!("Stopping service...");
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("restart")
                .short("Restart the service")
                .run(|_| {
                    println!("Restarting service...");
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("status")
                .short("Show service status")
                .run(|_| {
                    println!("Service is running");
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("config")
                .short("Manage configuration")
                .subcommand(
                    CommandBuilder::new("get")
                        .short("Get configuration value")
                        .run(|_| {
                            println!("Configuration value: enabled");
                            Ok(())
                        })
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("set")
                        .short("Set configuration value")
                        .run(|_| {
                            println!("Configuration updated");
                            Ok(())
                        })
                        .build(),
                )
                .build(),
        )
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();

    // Show what command was attempted
    if !args.is_empty() {
        println!("Command attempted: {}\n", args[0]);
    }

    if let Err(e) = app.execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
