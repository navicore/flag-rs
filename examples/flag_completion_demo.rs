//! Demonstrates inline flag completion support
use flag_rs::{CommandBuilder, CompletionResult, Flag};

fn main() {
    let app = CommandBuilder::new("deploy")
        .short("Deploy application to various environments")
        .flag(
            Flag::choice("environment", &["dev", "staging", "prod"])
                .short('e')
                .usage("Target environment")
                .required()
                .completion(|_ctx, prefix| {
                    // Dynamic completion - could check available environments
                    let environments = vec![
                        ("dev", "Development environment"),
                        ("staging", "Staging environment (pre-production)"),
                        ("prod", "Production environment"),
                    ];

                    let mut result = CompletionResult::new();
                    for (env, desc) in environments {
                        if env.starts_with(prefix) {
                            result = result.add_with_description(env.to_string(), desc.to_string());
                        }
                    }

                    // Add active help if no prefix
                    if prefix.is_empty() {
                        result = result.add_help_text("Available environments: dev, staging, prod");
                    }

                    Ok(result)
                }),
        )
        .flag(
            Flag::file("config")
                .short('c')
                .usage("Configuration file")
                .default_str("config.yaml")
                .completion(|_ctx, prefix| {
                    // In a real app, you might list files in the config directory
                    let configs = vec![
                        "config.yaml",
                        "config.dev.yaml",
                        "config.staging.yaml",
                        "config.prod.yaml",
                        "secrets.yaml",
                    ];

                    Ok(CompletionResult::new().extend(
                        configs
                            .into_iter()
                            .filter(|c| c.starts_with(prefix))
                            .map(String::from),
                    ))
                }),
        )
        .flag(
            Flag::directory("output")
                .short('o')
                .usage("Output directory for deployment artifacts")
                .completion(|_ctx, prefix| {
                    // In a real app, you might list actual directories
                    let dirs = vec!["./build", "./dist", "./output", "/tmp/deploy"];

                    Ok(CompletionResult::new().extend(
                        dirs.into_iter()
                            .filter(|d| d.starts_with(prefix))
                            .map(String::from),
                    ))
                }),
        )
        .flag(
            Flag::string_slice("service")
                .short('s')
                .usage("Services to deploy (can be specified multiple times)")
                .completion(|ctx, prefix| {
                    // Context-aware completion based on environment
                    let env = ctx.flag_str_or("environment", "dev");

                    let services = match env {
                        "dev" => vec!["api", "web", "worker", "scheduler", "debug-panel"],
                        "staging" => vec!["api", "web", "worker", "scheduler"],
                        "prod" => vec!["api", "web", "worker"],
                        _ => vec!["api", "web"],
                    };

                    let mut result = CompletionResult::new().extend(
                        services
                            .into_iter()
                            .filter(|s| s.starts_with(prefix))
                            .map(String::from),
                    );

                    // Add context-aware help
                    if prefix.is_empty() {
                        result = result
                            .add_help_text(format!("Available services for {} environment", env));
                    }

                    Ok(result)
                }),
        )
        .flag(
            Flag::range("replicas", 1, 10)
                .short('r')
                .usage("Number of replicas to deploy")
                .default_int(2)
                .completion(|_ctx, _prefix| {
                    // Suggest common replica counts
                    Ok(CompletionResult::new()
                        .add_with_description(
                            "1".to_string(),
                            "Single instance (dev/test)".to_string(),
                        )
                        .add_with_description("2".to_string(), "Basic redundancy".to_string())
                        .add_with_description("3".to_string(), "Standard production".to_string())
                        .add_with_description("5".to_string(), "High availability".to_string()))
                }),
        )
        .run(|ctx| {
            let env = ctx.flag_str_or("environment", "dev");
            let config = ctx.flag_str_or("config", "config.yaml");
            let replicas = ctx.flag_int_or("replicas", 2);

            println!("Deploying to {} environment", env);
            println!("Using configuration: {}", config);
            println!("Replica count: {}", replicas);

            if let Some(output) = ctx.flag("output") {
                println!("Output directory: {}", output);
            }

            if let Some(services) = ctx.flag("service") {
                println!("Deploying services: {}", services);
            } else {
                println!("Deploying all services");
            }

            Ok(())
        })
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
