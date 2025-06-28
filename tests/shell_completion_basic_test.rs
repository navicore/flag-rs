//! Basic integration tests for shell completion functionality
//!
//! These tests verify the basic completion functionality that is implemented.

use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType};

/// Creates a test CLI with various command structures
fn create_test_cli() -> flag_rs::Command {
    CommandBuilder::new("testcli")
        .short("A test CLI for completion testing")
        .flag(
            Flag::new("verbose")
                .short('v')
                .usage("Enable verbose output")
                .value_type(FlagType::Bool),
        )
        .flag(
            Flag::new("config")
                .short('c')
                .usage("Config file path")
                .value_type(FlagType::File),
        )
        .subcommand(
            CommandBuilder::new("server")
                .short("Server management commands")
                .flag(
                    Flag::new("port")
                        .short('p')
                        .usage("Port to listen on")
                        .value_type(FlagType::Int)
                        .default(flag_rs::FlagValue::Int(8080)),
                )
                .subcommand(
                    CommandBuilder::new("start")
                        .short("Start the server")
                        .flag(
                            Flag::new("daemon")
                                .short('d')
                                .usage("Run as daemon")
                                .value_type(FlagType::Bool),
                        )
                        .arg_completion(|_ctx, prefix| {
                            let profiles = vec!["development", "staging", "production"];
                            Ok(CompletionResult::new().extend(
                                profiles
                                    .into_iter()
                                    .filter(|p| p.starts_with(prefix))
                                    .map(String::from),
                            ))
                        })
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("stop")
                        .short("Stop the server")
                        .flag(
                            Flag::new("force")
                                .short('f')
                                .usage("Force stop")
                                .value_type(FlagType::Bool),
                        )
                        .build(),
                )
                .build(),
        )
        .subcommand(
            CommandBuilder::new("database")
                .short("Database operations")
                .alias("db")
                .flag(
                    Flag::new("host")
                        .short('h')
                        .usage("Database host")
                        .value_type(FlagType::String)
                        .default(flag_rs::FlagValue::String("localhost".to_string())),
                )
                .flag_completion("host", |_ctx, prefix| {
                    let hosts = vec!["localhost", "db.example.com", "192.168.1.100"];
                    Ok(CompletionResult::new().extend(
                        hosts
                            .into_iter()
                            .filter(|h| h.starts_with(prefix))
                            .map(String::from),
                    ))
                })
                .subcommand(
                    CommandBuilder::new("migrate")
                        .short("Run database migrations")
                        .arg_completion(|_ctx, prefix| {
                            let migrations = vec![
                                "001_initial_schema",
                                "002_add_users_table",
                                "003_add_indexes",
                            ];
                            Ok(CompletionResult::new().extend(
                                migrations
                                    .into_iter()
                                    .filter(|m| m.starts_with(prefix))
                                    .map(String::from),
                            ))
                        })
                        .build(),
                )
                .build(),
        )
        .build()
}

#[test]
fn test_basic_command_structure() {
    let app = create_test_cli();

    // Test basic properties
    assert_eq!(app.name(), "testcli");
    assert_eq!(app.short(), "A test CLI for completion testing");

    // Test flags exist
    assert!(app.flags().contains_key("verbose"));
    assert!(app.flags().contains_key("config"));

    // Test subcommands exist
    assert!(app.subcommands().contains_key("server"));
    assert!(app.subcommands().contains_key("database"));
}

#[test]
fn test_completion_with_aliases() {
    let app = create_test_cli();

    // Find database command by alias
    let db_cmd = app.find_subcommand("db");
    assert!(db_cmd.is_some());
    assert_eq!(db_cmd.unwrap().name(), "database");
}

#[test]
fn test_dynamic_arg_completion() {
    let app = create_test_cli();
    let ctx = flag_rs::Context::new(vec![
        "testcli".to_string(),
        "server".to_string(),
        "start".to_string(),
    ]);

    // Find the start subcommand
    let server_cmd = app.find_subcommand("server").unwrap();
    let start_cmd = server_cmd.find_subcommand("start").unwrap();

    // Test argument completion
    let result = start_cmd.get_completions(&ctx, "dev", None).unwrap();
    assert_eq!(result.values, vec!["development"]);

    let result = start_cmd.get_completions(&ctx, "prod", None).unwrap();
    assert_eq!(result.values, vec!["production"]);

    let result = start_cmd.get_completions(&ctx, "", None).unwrap();
    assert_eq!(result.values.len(), 3);
}

#[test]
fn test_dynamic_flag_completion() {
    let app = create_test_cli();
    let ctx = flag_rs::Context::new(vec!["testcli".to_string(), "database".to_string()]);

    // Find the database command
    let db_cmd = app.find_subcommand("database").unwrap();

    // Test flag value completion
    let result = db_cmd.get_completions(&ctx, "local", Some("host")).unwrap();
    assert_eq!(result.values, vec!["localhost"]);

    let result = db_cmd.get_completions(&ctx, "db.", Some("host")).unwrap();
    assert_eq!(result.values, vec!["db.example.com"]);
}

#[test]
fn test_nested_subcommand_structure() {
    let app = create_test_cli();

    // Test that nested subcommands exist
    let server = app.find_subcommand("server").unwrap();
    assert!(server.subcommands().contains_key("start"));
    assert!(server.subcommands().contains_key("stop"));

    let database = app.find_subcommand("database").unwrap();
    assert!(database.subcommands().contains_key("migrate"));
}

#[test]
fn test_flag_types() {
    let app = CommandBuilder::new("typetest")
        .flag(Flag::new("choice").value_type(FlagType::Choice(vec![
            "option1".to_string(),
            "option2".to_string(),
            "option3".to_string(),
        ])))
        .flag(Flag::new("file").value_type(FlagType::File))
        .flag(Flag::new("dir").value_type(FlagType::Directory))
        .build();

    // Test that flags are properly stored with their types
    let choice_flag = app.flags().get("choice").unwrap();
    match &choice_flag.value_type {
        FlagType::Choice(choices) => {
            assert_eq!(choices.len(), 3);
            assert!(choices.contains(&"option1".to_string()));
        }
        _ => panic!("Expected Choice type"),
    }
}

#[test]
fn test_completion_with_global_flags() {
    let app = CommandBuilder::new("globaltest")
        .flag(
            Flag::new("global")
                .usage("A global flag")
                .value_type(FlagType::Bool),
        )
        .subcommand(
            CommandBuilder::new("sub")
                .flag(
                    Flag::new("local")
                        .usage("A local flag")
                        .value_type(FlagType::Bool),
                )
                .build(),
        )
        .build();

    // Verify flags are accessible at correct levels
    assert!(app.flags().contains_key("global"));

    let sub = app.find_subcommand("sub").unwrap();
    assert!(sub.flags().contains_key("local"));
    // Global flags are inherited through parent pointer
}

#[test]
fn test_completion_special_characters() {
    let app = CommandBuilder::new("special-test")
        .subcommand(
            CommandBuilder::new("sub-command")
                .flag(
                    Flag::new("with-dash")
                        .usage("Flag with dash")
                        .value_type(FlagType::Bool),
                )
                .build(),
        )
        .build();

    // Verify special characters are properly handled in names
    let sub = app.find_subcommand("sub-command");
    assert!(sub.is_some());
    assert!(sub.unwrap().flags().contains_key("with-dash"));
}

#[test]
fn test_empty_completion_result() {
    let app = CommandBuilder::new("emptytest")
        .arg_completion(|_ctx, prefix| {
            if prefix.starts_with('z') {
                Ok(CompletionResult::new())
            } else {
                Ok(CompletionResult::new().add("something"))
            }
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["emptytest".to_string()]);

    // Should return empty result for "z" prefix
    let result = app.get_completions(&ctx, "z", None).unwrap();
    assert!(result.values.is_empty());

    // Should return result for other prefixes
    let result = app.get_completions(&ctx, "s", None).unwrap();
    assert_eq!(result.values, vec!["something"]);
}

#[test]
fn test_completion_with_descriptions() {
    let app = CommandBuilder::new("desctest")
        .arg_completion(|_ctx, _prefix| {
            Ok(CompletionResult::new()
                .add_with_description("option1", "First option")
                .add_with_description("option2", "Second option")
                .add("option3"))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["desctest".to_string()]);
    let result = app.get_completions(&ctx, "", None).unwrap();

    assert_eq!(result.values.len(), 3);
    assert_eq!(result.descriptions[0], "First option");
    assert_eq!(result.descriptions[1], "Second option");
    assert_eq!(result.descriptions[2], "");
}
