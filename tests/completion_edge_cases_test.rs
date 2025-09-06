//! Edge case tests for shell completion functionality
//!
//! These tests cover edge cases and complex scenarios for completions.

use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType};

#[test]
fn test_completion_with_equals_in_flag() {
    let app = CommandBuilder::new("equalstest")
        .flag(
            Flag::new("key-value")
                .usage("Key=value pair")
                .value_type(FlagType::String),
        )
        .flag_completion("key-value", |_ctx, prefix| {
            let pairs = vec!["env=production", "env=staging", "region=us-east-1"];
            Ok(CompletionResult::new().extend(
                pairs
                    .into_iter()
                    .filter(|p| p.starts_with(prefix))
                    .map(String::from),
            ))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["equalstest".to_string()]);

    // Test completion with equals sign
    let result = app
        .get_completions(&ctx, "env=", Some("key-value"))
        .unwrap();
    assert_eq!(result.values.len(), 2);
    assert!(result.values.contains(&"env=production".to_string()));
    assert!(result.values.contains(&"env=staging".to_string()));
}

#[test]
fn test_completion_with_spaces_in_values() {
    let app = CommandBuilder::new("spacetest")
        .arg_completion(|_ctx, prefix| {
            let items = vec!["item one", "item two", "item-three"];
            Ok(CompletionResult::new().extend(
                items
                    .into_iter()
                    .filter(|i| i.starts_with(prefix))
                    .map(String::from),
            ))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["spacetest".to_string()]);

    // Test completion with space prefix
    let result = app.get_completions(&ctx, "item ", None).unwrap();
    assert_eq!(result.values.len(), 2);
    assert!(result.values.contains(&"item one".to_string()));
    assert!(result.values.contains(&"item two".to_string()));
}

#[test]
fn test_completion_with_quotes() {
    let app = CommandBuilder::new("quotetest")
        .arg_completion(|_ctx, prefix| {
            let items = vec![
                r#"file"with"quotes.txt"#,
                r"file'with'quotes.txt",
                "normal-file.txt",
            ];
            Ok(CompletionResult::new().extend(
                items
                    .into_iter()
                    .filter(|i| i.starts_with(prefix))
                    .map(String::from),
            ))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["quotetest".to_string()]);

    // Test that quotes are preserved in completions
    let result = app.get_completions(&ctx, "file", None).unwrap();
    assert_eq!(result.values.len(), 2);
    assert!(
        result
            .values
            .contains(&r#"file"with"quotes.txt"#.to_string())
    );
    assert!(result.values.contains(&r"file'with'quotes.txt".to_string()));
}

#[test]
fn test_completion_with_unicode() {
    let app = CommandBuilder::new("unicodetest")
        .arg_completion(|_ctx, prefix| {
            let items = vec!["café", "naïve", "résumé", "🚀deploy", "测试test"];
            Ok(CompletionResult::new().extend(
                items
                    .into_iter()
                    .filter(|i| i.starts_with(prefix))
                    .map(String::from),
            ))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["unicodetest".to_string()]);

    // Test Unicode prefix matching
    let result = app.get_completions(&ctx, "c", None).unwrap();
    assert_eq!(result.values, vec!["café"]);

    let result = app.get_completions(&ctx, "🚀", None).unwrap();
    assert_eq!(result.values, vec!["🚀deploy"]);

    let result = app.get_completions(&ctx, "测", None).unwrap();
    assert_eq!(result.values, vec!["测试test"]);
}

#[test]
fn test_completion_with_very_long_values() {
    let app = CommandBuilder::new("longtest")
        .arg_completion(|_ctx, _prefix| {
            let long_value = "a".repeat(1000);
            Ok(CompletionResult::new()
                .add(&long_value)
                .add_with_description("short", "b".repeat(500)))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["longtest".to_string()]);
    let result = app.get_completions(&ctx, "", None).unwrap();

    // Verify long values are handled
    assert_eq!(result.values.len(), 2);
    assert_eq!(result.values[0].len(), 1000);
    assert_eq!(result.descriptions[1].len(), 500);
}

#[test]
fn test_completion_with_empty_prefix_and_many_results() {
    let app = CommandBuilder::new("manytest")
        .arg_completion(|_ctx, prefix| {
            if prefix.is_empty() {
                // Return many results when no prefix
                let items: Vec<String> = (0..100).map(|i| format!("item{i:03}")).collect();
                Ok(CompletionResult::new().extend(items))
            } else {
                Ok(CompletionResult::new())
            }
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["manytest".to_string()]);
    let result = app.get_completions(&ctx, "", None).unwrap();

    assert_eq!(result.values.len(), 100);
    assert_eq!(result.values[0], "item000");
    assert_eq!(result.values[99], "item099");
}

#[test]
fn test_completion_with_active_help() {
    let app = CommandBuilder::new("helptest")
        .arg_completion(|_ctx, prefix| {
            let mut result = CompletionResult::new();

            if prefix.is_empty() {
                result = result
                    .add("option1")
                    .add("option2")
                    .add_help_text("💡 Tip: Start typing to filter options");
            } else if prefix.len() < 3 {
                result = result
                    .add_help_text("Keep typing for more specific results...")
                    .add_conditional_help("Press Ctrl+C to cancel", |_| true);
            }

            Ok(result)
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["helptest".to_string()]);

    // Empty prefix should include help
    let result = app.get_completions(&ctx, "", None).unwrap();
    assert_eq!(result.values.len(), 2);
    assert_eq!(result.active_help.len(), 1);
    assert!(result.active_help[0].message.contains("Start typing"));

    // Short prefix should have different help
    let result = app.get_completions(&ctx, "op", None).unwrap();
    assert_eq!(result.values.len(), 0);
    assert_eq!(result.active_help.len(), 2);
}

#[test]
fn test_completion_error_handling() {
    let app = CommandBuilder::new("errortest")
        .arg_completion(|_ctx, prefix| {
            if prefix == "error" {
                Err(flag_rs::Error::Completion("Simulated error".to_string()))
            } else {
                Ok(CompletionResult::new().add("success"))
            }
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["errortest".to_string()]);

    // Normal completion should work
    let result = app.get_completions(&ctx, "s", None);
    assert!(result.is_ok());

    // Error prefix should return error
    let result = app.get_completions(&ctx, "error", None);
    assert!(result.is_err());
    match result {
        Err(flag_rs::Error::Completion(msg)) => assert_eq!(msg, "Simulated error"),
        _ => panic!("Expected Completion error"),
    }
}

#[test]
fn test_recursive_command_completion() {
    let app = CommandBuilder::new("recursive")
        .subcommand(
            CommandBuilder::new("level1")
                .subcommand(
                    CommandBuilder::new("level2")
                        .subcommand(
                            CommandBuilder::new("level3")
                                .subcommand(
                                    CommandBuilder::new("level4")
                                        .arg_completion(|_ctx, _prefix| {
                                            Ok(CompletionResult::new().add("deep-completion"))
                                        })
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build();

    // Verify deep nesting works
    let cmd = app
        .find_subcommand("level1")
        .unwrap()
        .find_subcommand("level2")
        .unwrap()
        .find_subcommand("level3")
        .unwrap()
        .find_subcommand("level4")
        .unwrap();

    let ctx = flag_rs::Context::new(vec![
        "recursive".to_string(),
        "level1".to_string(),
        "level2".to_string(),
        "level3".to_string(),
        "level4".to_string(),
    ]);

    let result = cmd.get_completions(&ctx, "", None).unwrap();
    assert_eq!(result.values, vec!["deep-completion"]);
}

#[test]
fn test_completion_with_mixed_flag_types() {
    let app = CommandBuilder::new("mixedtest")
        .flag(Flag::new("bool").value_type(FlagType::Bool))
        .flag(Flag::new("string").value_type(FlagType::String))
        .flag(Flag::new("int").value_type(FlagType::Int))
        .flag(Flag::new("float").value_type(FlagType::Float))
        .flag(Flag::new("array").value_type(FlagType::StringArray))
        .flag(Flag::new("choice").value_type(FlagType::Choice(vec![
            "opt1".to_string(),
            "opt2".to_string(),
        ])))
        .flag(Flag::new("range").value_type(FlagType::Range(1, 10)))
        .flag(Flag::new("file").value_type(FlagType::File))
        .flag(Flag::new("dir").value_type(FlagType::Directory))
        .build();

    // Verify different flag types are properly stored
    assert!(app.flags().contains_key("bool"));
    assert!(app.flags().contains_key("string"));
    assert!(app.flags().contains_key("int"));
    assert!(app.flags().contains_key("float"));
    assert!(app.flags().contains_key("array"));
    assert!(app.flags().contains_key("choice"));
    assert!(app.flags().contains_key("range"));
    assert!(app.flags().contains_key("file"));
    assert!(app.flags().contains_key("dir"));

    // Verify choice flag has correct options
    match &app.flags().get("choice").unwrap().value_type {
        FlagType::Choice(choices) => {
            assert!(choices.contains(&"opt1".to_string()));
            assert!(choices.contains(&"opt2".to_string()));
        }
        _ => panic!("Expected Choice type"),
    }
}

#[test]
fn test_completion_state_isolation() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = Arc::clone(&call_count);

    let app = CommandBuilder::new("statetest")
        .arg_completion(move |_ctx, _prefix| {
            let count = count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(CompletionResult::new().add(format!("call{count}")))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["statetest".to_string()]);

    // Multiple calls should maintain independent state
    let result1 = app.get_completions(&ctx, "", None).unwrap();
    let result2 = app.get_completions(&ctx, "", None).unwrap();
    let result3 = app.get_completions(&ctx, "", None).unwrap();

    assert_eq!(result1.values, vec!["call0"]);
    assert_eq!(result2.values, vec!["call1"]);
    assert_eq!(result3.values, vec!["call2"]);
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}
