//! Integration tests for `ActiveHelp` functionality
use flag_rs::{CommandBuilder, CompletionResult, Context};

#[test]
fn test_active_help_in_completions() {
    let cmd = CommandBuilder::new("test")
        .flag(
            flag_rs::Flag::new("verbose")
                .short('v')
                .usage("Enable verbose output")
                .value_type(flag_rs::FlagType::Bool),
        )
        .subcommand(
            CommandBuilder::new("deploy")
                .arg_completion(|ctx, _prefix| {
                    let mut result = CompletionResult::new().add("staging").add("production");

                    // Add conditional help based on flags
                    if ctx.flag("verbose").is_none() {
                        result = result.add_help_text("Tip: Use -v for detailed deployment logs");
                    }

                    result = result.add_conditional_help(
                        "Warning: Production deployments require approval",
                        |_| true,
                    );

                    Ok(result)
                })
                .build(),
        )
        .build();

    // Test completion request
    let completion_args = vec![
        "__complete".to_string(),
        "deploy".to_string(),
        String::new(),
    ];

    let result = cmd.handle_completion_request(&completion_args);
    assert!(result.is_ok());

    let completions = result.unwrap();

    // Should have both completion values and help messages
    assert!(completions.iter().any(|s| s == "staging"));
    assert!(completions.iter().any(|s| s == "production"));

    // ActiveHelp messages should be included with special prefix
    assert!(
        completions
            .iter()
            .any(|s| s.contains("Use -v for detailed deployment logs"))
    );
    assert!(
        completions
            .iter()
            .any(|s| s.contains("Production deployments require approval"))
    );
}

#[test]
fn test_conditional_active_help() {
    let cmd = CommandBuilder::new("test")
        .flag(
            flag_rs::Flag::new("format")
                .short('f')
                .usage("Output format")
                .value_type(flag_rs::FlagType::String),
        )
        .subcommand(
            CommandBuilder::new("list")
                .arg_completion(|_ctx, _prefix| {
                    let mut result = CompletionResult::new().add("items").add("details");

                    // Only show this help if format flag is not set
                    result = result.add_conditional_help(
                        "Use --format json for machine-readable output",
                        |context| context.flag("format").is_none(),
                    );

                    // Only show this help if format is set to table
                    result = result
                        .add_conditional_help("Table format supports color output", |context| {
                            context.flag("format").is_some_and(|f| f == "table")
                        });

                    Ok(result)
                })
                .build(),
        )
        .build();

    // Test without format flag
    let args1 = vec!["__complete".to_string(), "list".to_string(), String::new()];
    let result1 = cmd.handle_completion_request(&args1).unwrap();
    assert!(
        result1
            .iter()
            .any(|s| s.contains("Use --format json for machine-readable output"))
    );
    assert!(
        !result1
            .iter()
            .any(|s| s.contains("Table format supports color output"))
    );

    // Test with format=table
    let args2 = vec![
        "__complete".to_string(),
        "--format".to_string(),
        "table".to_string(),
        "list".to_string(),
        String::new(),
    ];
    let result2 = cmd.handle_completion_request(&args2).unwrap();
    assert!(
        !result2
            .iter()
            .any(|s| s.contains("Use --format json for machine-readable output"))
    );
    assert!(
        result2
            .iter()
            .any(|s| s.contains("Table format supports color output"))
    );
}

#[test]
fn test_active_help_with_different_shells() {
    let cmd = CommandBuilder::new("test")
        .subcommand(
            CommandBuilder::new("info")
                .arg_completion(|_ctx, _prefix| {
                    Ok(CompletionResult::new()
                        .add("version")
                        .add("license")
                        .add_help_text("This is a help message"))
                })
                .build(),
        )
        .build();

    // Test with bash shell
    unsafe { std::env::set_var("TEST_COMPLETE", "bash") };
    let bash_result = cmd
        .handle_completion_request(&["__complete".to_string(), "info".to_string(), String::new()])
        .unwrap();

    // Bash format should have the _activehelp_ prefix
    assert!(
        bash_result
            .iter()
            .any(|s| s == "_activehelp_ This is a help message")
    );

    // Clean up
    unsafe { std::env::remove_var("TEST_COMPLETE") };
}

#[test]
fn test_active_help_config() {
    use flag_rs::active_help::ActiveHelpConfig;

    // Test default config
    let config = ActiveHelpConfig::default();
    assert!(config.is_enabled());
    assert!(config.show_on_double_tab);
    assert!(config.show_on_no_completions);

    // Test with environment variable
    unsafe { std::env::set_var("COBRA_ACTIVE_HELP", "0") };
    assert!(!config.is_enabled());

    unsafe { std::env::set_var("COBRA_ACTIVE_HELP", "false") };
    assert!(!config.is_enabled());

    unsafe { std::env::set_var("COBRA_ACTIVE_HELP", "1") };
    assert!(config.is_enabled());

    // Clean up
    unsafe { std::env::remove_var("COBRA_ACTIVE_HELP") };
}

#[test]
fn test_no_active_help_without_context() {
    use flag_rs::completion_format::CompletionFormat;

    let result = CompletionResult::new()
        .add("option1")
        .add_help_text("This should not appear without context");

    // Format without context - ActiveHelp should not be included
    let formatted = CompletionFormat::Bash.format(&result, None);

    assert!(formatted.contains(&"option1".to_string()));
    assert!(!formatted.iter().any(|s| s.contains("_activehelp_")));
}

#[test]
fn test_active_help_in_flag_completion() {
    let cmd = CommandBuilder::new("test")
        .flag(
            flag_rs::Flag::new("environment")
                .short('e')
                .usage("Target environment")
                .value_type(flag_rs::FlagType::String),
        )
        .flag_completion("environment", |_ctx, prefix| {
            let mut result = CompletionResult::new();

            let envs = vec![
                ("development", "Local development environment"),
                ("staging", "Pre-production environment"),
                ("production", "Live production environment"),
            ];

            for (env, desc) in envs {
                if env.starts_with(prefix) {
                    result = result.add_with_description(env, desc);
                }
            }

            // Add help based on the prefix
            if prefix.is_empty() {
                result = result.add_help_text("Choose an environment to deploy to");
            }

            if prefix == "prod" {
                result = result.add_help_text(
                    "Warning: Production deployments are permanent and require approval",
                );
            }

            Ok(result)
        })
        .build();

    // Test flag completion with empty prefix
    let args = vec![
        "__complete".to_string(),
        "--environment".to_string(),
        String::new(),
    ];
    let result = cmd.handle_completion_request(&args).unwrap();

    assert!(result.iter().any(|s| s.contains("development")));
    assert!(
        result
            .iter()
            .any(|s| s.contains("Choose an environment to deploy to"))
    );

    // Test with "prod" prefix
    let args2 = vec![
        "__complete".to_string(),
        "--environment".to_string(),
        "prod".to_string(),
    ];
    let result2 = cmd.handle_completion_request(&args2).unwrap();

    assert!(result2.iter().any(|s| s.contains("production")));
    assert!(
        result2
            .iter()
            .any(|s| s.contains("Production deployments are permanent"))
    );
}

#[test]
fn test_multiple_active_help_messages() {
    use flag_rs::completion_format::CompletionFormat;

    let result = CompletionResult::new()
        .add("option1")
        .add_help_text("First help message")
        .add_help_text("Second help message")
        .add_conditional_help("Third conditional help", |_| true)
        .add_conditional_help("Hidden conditional help", |_| false);

    let ctx = Context::new(vec![]);
    let formatted = CompletionFormat::Bash.format(&result, Some(&ctx));

    // Count help messages
    let help_count = formatted
        .iter()
        .filter(|s| s.starts_with("_activehelp_"))
        .count();

    assert_eq!(help_count, 3); // First, Second, and Third (not Hidden)
}
