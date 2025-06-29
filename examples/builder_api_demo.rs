//! Demonstrates the improved builder API features
use flag_rs::{CommandBuilder, Flag};

fn main() {
    // Demonstrates all the new builder API improvements
    let app = CommandBuilder::new("app")
        .short("Modern CLI app with improved API")
        .long(
            "This example showcases the builder API improvements including:\n\
               - Type-specific flag constructors\n\
               - Inline flag completions\n\
               - Bulk flag/subcommand methods\n\
               - Type-safe context access",
        )
        // Bulk flag addition
        .flags(vec![
            // Type-specific constructors
            Flag::bool("verbose")
                .short('v')
                .usage("Enable verbose output")
                .default_bool(false),
            Flag::bool("quiet").short('q').usage("Suppress all output"),
            Flag::int("threads")
                .short('t')
                .usage("Number of worker threads")
                .default_int(4),
            Flag::choice("log-level", &["debug", "info", "warn", "error"])
                .usage("Set the logging level")
                .default_str("info")
                .completion(|_ctx, prefix| {
                    // Inline completion with descriptions
                    let levels = vec![
                        ("debug", "Show all messages including debug"),
                        ("info", "Show informational messages and above"),
                        ("warn", "Show warnings and errors only"),
                        ("error", "Show errors only"),
                    ];

                    let mut result = flag_rs::CompletionResult::new();
                    for (level, desc) in levels {
                        if level.starts_with(prefix) {
                            result =
                                result.add_with_description(level.to_string(), desc.to_string());
                        }
                    }
                    Ok(result)
                }),
        ])
        // Bulk subcommand addition
        .subcommands(vec![
            CommandBuilder::new("init")
                .short("Initialize a new project")
                .flags(vec![
                    Flag::string("name")
                        .short('n')
                        .usage("Project name")
                        .required(),
                    Flag::choice("template", &["basic", "web", "api", "full"])
                        .usage("Project template to use")
                        .default_str("basic"),
                    Flag::directory("path")
                        .short('p')
                        .usage("Directory to create project in")
                        .completion(|_ctx, prefix| {
                            // In a real app, list actual directories
                            let dirs = vec!["./", "../", "/tmp/", "~/projects/"];
                            Ok(flag_rs::CompletionResult::new().extend(
                                dirs.into_iter()
                                    .filter(|d| d.starts_with(prefix))
                                    .map(String::from),
                            ))
                        }),
                ])
                .run(|ctx| {
                    // Type-safe flag access
                    let name = ctx.flag_str_or("name", "my-project");
                    let template = ctx.flag_str_or("template", "basic");
                    let path = ctx.flag_str_or("path", ".");
                    let verbose = ctx.flag_bool_or("verbose", false);

                    if verbose {
                        println!("Initializing {} project '{}' in {}", template, name, path);
                    } else {
                        println!("Creating project '{}'...", name);
                    }

                    Ok(())
                })
                .build(),
            CommandBuilder::new("build")
                .short("Build the project")
                .flags(vec![
                    Flag::bool("release")
                        .short('r')
                        .usage("Build in release mode"),
                    Flag::string_slice("features")
                        .short('f')
                        .usage("Enable features (can be specified multiple times)")
                        .completion(|_ctx, prefix| {
                            let features = vec!["async", "tls", "compression", "metrics"];
                            Ok(flag_rs::CompletionResult::new().extend(
                                features
                                    .into_iter()
                                    .filter(|f| f.starts_with(prefix))
                                    .map(String::from),
                            ))
                        }),
                    Flag::range("jobs", 1, 32)
                        .short('j')
                        .usage("Number of parallel jobs")
                        .default_int(i64::try_from(num_cpus()).unwrap_or(4)),
                ])
                .run(|ctx| {
                    let release = ctx.flag_bool_or("release", false);
                    let jobs = ctx.flag_int_or("jobs", i64::try_from(num_cpus()).unwrap_or(4));
                    let quiet = ctx.flag_bool_or("quiet", false);

                    if !quiet {
                        println!(
                            "Building in {} mode with {} jobs",
                            if release { "release" } else { "debug" },
                            jobs
                        );

                        if let Some(features) = ctx.flag("features") {
                            println!("Features: {}", features);
                        }
                    }

                    Ok(())
                })
                .build(),
            CommandBuilder::new("test")
                .short("Run tests")
                .flags(vec![
                    Flag::string("filter").usage("Only run tests matching this pattern"),
                    Flag::bool("nocapture").usage("Don't capture test output"),
                    Flag::int("test-threads")
                        .usage("Number of threads to use for running tests")
                        .default_int(1),
                ])
                .run(|ctx| {
                    let threads = ctx.flag_int_or("test-threads", 1);
                    let nocapture = ctx.flag_bool_or("nocapture", false);
                    let verbose = ctx.flag_bool_or("verbose", false);

                    if verbose {
                        println!("Running tests with {} thread(s)", threads);
                        if nocapture {
                            println!("Output capture disabled");
                        }

                        if let Some(filter) = ctx.flag("filter") {
                            println!("Filter: {}", filter);
                        }
                    }

                    println!("Running tests...");
                    Ok(())
                })
                .build(),
        ])
        .run(|_ctx| {
            // Root command - show help by default
            println!("Use --help for usage information");
            Ok(())
        })
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// Dummy function for example
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4)
}
