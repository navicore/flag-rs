use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType};

fn main() {
    let cmd = CommandBuilder::new("test")
        .short("Test completion with descriptions")
        .flag(
            Flag::new("environment")
                .short('e')
                .usage("Target environment")
                .value_type(FlagType::String),
        )
        .flag_completion("environment", |_ctx, prefix| {
            let mut result = CompletionResult::new();
            let envs = vec![
                ("dev", "Development environment - safe for testing"),
                ("staging", "Staging environment - mirror of production"),
                ("production", "Production environment - BE CAREFUL!"),
            ];

            for (env, desc) in envs {
                if env.starts_with(prefix) {
                    result = result.add_with_description(env, desc);
                }
            }

            Ok(result)
        })
        .subcommand(
            CommandBuilder::new("deploy")
                .short("Deploy the application")
                .long("Deploy the application to the specified environment")
                .build(),
        )
        .subcommand(
            CommandBuilder::new("rollback")
                .short("Rollback to previous version")
                .long("Rollback the application to the previous deployed version")
                .build(),
        )
        .build();

    // Test completion output
    if let Ok(shell_type) = std::env::var("TEST_COMPLETE") {
        // For testing, override shell type to "display" to see formatted output
        let display_mode = std::env::var("DISPLAY_MODE").is_ok();
        let args: Vec<String> = std::env::args().skip(1).collect();

        if display_mode {
            // This is a hack for testing - normally shells would handle this
            println!("Display mode - showing formatted completions:");
            match cmd.handle_completion_request(&args) {
                Ok(completions) => {
                    for completion in completions {
                        println!("{}", completion);
                    }
                }
                Err(e) => eprintln!("Completion error: {}", e),
            }
        } else {
            match cmd.handle_completion_request(&args) {
                Ok(completions) => {
                    println!("Shell type: {}", shell_type);
                    println!("Completions:");
                    for completion in completions {
                        println!("{}", completion);
                    }
                }
                Err(e) => eprintln!("Completion error: {}", e),
            }
        }
    } else {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if let Err(e) = cmd.execute(args) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
