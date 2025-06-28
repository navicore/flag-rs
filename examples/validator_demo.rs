//! Demonstrates argument validation features
use flag_rs::{ArgValidator, CommandBuilder, Error};

fn main() {
    let app = CommandBuilder::new("validator-demo")
        .short("Demonstrates argument validation")
        .subcommand(
            CommandBuilder::new("copy")
                .short("Copy files (requires exactly 2 arguments)")
                .args(ArgValidator::ExactArgs(2))
                .run(|ctx| {
                    let args = ctx.args();
                    println!("Copying from '{}' to '{}'", args[0], args[1]);
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("delete")
                .short("Delete files (requires at least 1 argument)")
                .args(ArgValidator::MinimumArgs(1))
                .run(|ctx| {
                    println!("Deleting {} file(s):", ctx.args().len());
                    for file in ctx.args() {
                        println!("  - {}", file);
                    }
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("list")
                .short("List items (accepts 0-3 arguments)")
                .args(ArgValidator::RangeArgs(0, 3))
                .run(|ctx| {
                    if ctx.args().is_empty() {
                        println!("Listing all items");
                    } else {
                        println!("Listing specific items:");
                        for item in ctx.args() {
                            println!("  - {}", item);
                        }
                    }
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("action")
                .short("Perform action (only start/stop/restart allowed)")
                .args(ArgValidator::OnlyValidArgs(vec![
                    "start".to_string(),
                    "stop".to_string(),
                    "restart".to_string(),
                ]))
                .run(|ctx| {
                    let action = ctx.args().first().map(String::as_str).unwrap_or("start");
                    println!("Performing action: {}", action);
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("numbers")
                .short("Process numbers (custom validator for integers)")
                .args(ArgValidator::Custom(std::sync::Arc::new(|args| {
                    if args.is_empty() {
                        return Err(Error::ArgumentValidation {
                            message: "at least one number required".to_string(),
                            expected: "numbers".to_string(),
                            received: 0,
                        });
                    }

                    for (i, arg) in args.iter().enumerate() {
                        if arg.parse::<i32>().is_err() {
                            return Err(Error::ArgumentValidation {
                                message: format!(
                                    "argument {} ('{}') must be an integer",
                                    i + 1,
                                    arg
                                ),
                                expected: "integer".to_string(),
                                received: args.len(),
                            });
                        }
                    }
                    Ok(())
                })))
                .run(|ctx| {
                    let numbers: Vec<i32> = ctx.args().iter().map(|s| s.parse().unwrap()).collect();
                    let sum: i32 = numbers.iter().sum();
                    println!("Sum of {} numbers: {}", numbers.len(), sum);
                    Ok(())
                })
                .build(),
        )
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
