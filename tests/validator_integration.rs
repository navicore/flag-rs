//! Integration tests for argument validators
use flag_rs::{ArgValidator, CommandBuilder, Error};

#[test]
fn test_exact_args_validator() {
    let cmd = CommandBuilder::new("test")
        .args(ArgValidator::ExactArgs(2))
        .run(|ctx| {
            assert_eq!(ctx.args().len(), 2);
            Ok(())
        })
        .build();

    // Should succeed with exactly 2 args
    assert!(
        cmd.execute(vec!["arg1".to_string(), "arg2".to_string()])
            .is_ok()
    );

    // Should fail with wrong number of args
    let result = cmd.execute(vec!["arg1".to_string()]);
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ArgumentValidation {
            message,
            expected,
            received,
        } => {
            assert!(message.contains("accepts 2 arg(s)"));
            assert_eq!(expected, "2");
            assert_eq!(received, 1);
        }
        _ => panic!("Expected ArgumentValidation error"),
    }
}

#[test]
fn test_minimum_args_validator() {
    let cmd = CommandBuilder::new("test")
        .args(ArgValidator::MinimumArgs(1))
        .run(|ctx| {
            assert!(!ctx.args().is_empty());
            Ok(())
        })
        .build();

    // Should succeed with 1 or more args
    assert!(cmd.execute(vec!["arg1".to_string()]).is_ok());
    assert!(
        cmd.execute(vec!["arg1".to_string(), "arg2".to_string()])
            .is_ok()
    );

    // Should fail with no args
    let result = cmd.execute(vec![]);
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ArgumentValidation { message, .. } => {
            assert!(message.contains("requires at least 1 arg(s)"));
        }
        _ => panic!("Expected ArgumentValidation error"),
    }
}

#[test]
fn test_maximum_args_validator() {
    let cmd = CommandBuilder::new("test")
        .args(ArgValidator::MaximumArgs(2))
        .run(|_| Ok(()))
        .build();

    // Should succeed with 0, 1, or 2 args
    assert!(cmd.execute(vec![]).is_ok());
    assert!(cmd.execute(vec!["arg1".to_string()]).is_ok());
    assert!(
        cmd.execute(vec!["arg1".to_string(), "arg2".to_string()])
            .is_ok()
    );

    // Should fail with more than 2 args
    let result = cmd.execute(vec!["1".to_string(), "2".to_string(), "3".to_string()]);
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ArgumentValidation { message, .. } => {
            assert!(message.contains("accepts at most 2 arg(s)"));
        }
        _ => panic!("Expected ArgumentValidation error"),
    }
}

#[test]
fn test_range_args_validator() {
    let cmd = CommandBuilder::new("test")
        .args(ArgValidator::RangeArgs(1, 3))
        .run(|_| Ok(()))
        .build();

    // Should succeed within range
    assert!(cmd.execute(vec!["1".to_string()]).is_ok());
    assert!(cmd.execute(vec!["1".to_string(), "2".to_string()]).is_ok());
    assert!(
        cmd.execute(vec!["1".to_string(), "2".to_string(), "3".to_string()])
            .is_ok()
    );

    // Should fail outside range
    assert!(cmd.execute(vec![]).is_err());
    assert!(
        cmd.execute(vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ])
        .is_err()
    );
}

#[test]
fn test_only_valid_args_validator() {
    let cmd = CommandBuilder::new("test")
        .args(ArgValidator::OnlyValidArgs(vec![
            "start".to_string(),
            "stop".to_string(),
            "restart".to_string(),
        ]))
        .run(|_| Ok(()))
        .build();

    // Should succeed with valid args
    assert!(cmd.execute(vec!["start".to_string()]).is_ok());
    assert!(cmd.execute(vec!["stop".to_string()]).is_ok());

    // Should fail with invalid arg
    let result = cmd.execute(vec!["invalid".to_string()]);
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ArgumentValidation {
            message, expected, ..
        } => {
            assert!(message.contains("invalid argument \"invalid\""));
            assert!(expected.contains("one of:"));
        }
        _ => panic!("Expected ArgumentValidation error"),
    }
}

#[test]
fn test_custom_validator() {
    let cmd = CommandBuilder::new("test")
        .args(ArgValidator::Custom(std::sync::Arc::new(|args| {
            // Custom validator: all args must be positive integers
            for arg in args {
                match arg.parse::<i32>() {
                    Ok(n) if n > 0 => {}
                    _ => {
                        return Err(Error::ArgumentValidation {
                            message: format!("'{arg}' must be a positive integer"),
                            expected: "positive integer".to_string(),
                            received: args.len(),
                        });
                    }
                }
            }
            Ok(())
        })))
        .run(|_| Ok(()))
        .build();

    // Should succeed with positive integers
    assert!(
        cmd.execute(vec!["1".to_string(), "2".to_string(), "3".to_string()])
            .is_ok()
    );

    // Should fail with non-positive integer
    assert!(cmd.execute(vec!["0".to_string()]).is_err());
    assert!(cmd.execute(vec!["-1".to_string()]).is_err());
    assert!(cmd.execute(vec!["abc".to_string()]).is_err());
}

#[test]
fn test_validator_with_subcommands() {
    let cmd = CommandBuilder::new("parent")
        .subcommand(
            CommandBuilder::new("child")
                .args(ArgValidator::ExactArgs(1))
                .run(|ctx| {
                    assert_eq!(ctx.args().len(), 1);
                    Ok(())
                })
                .build(),
        )
        .build();

    // Should validate args for subcommand
    assert!(
        cmd.execute(vec!["child".to_string(), "arg".to_string()])
            .is_ok()
    );

    // Should fail validation for subcommand
    let result = cmd.execute(vec!["child".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_validator_runs_before_command() {
    use std::sync::{Arc, Mutex};

    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    let cmd = CommandBuilder::new("test")
        .args(ArgValidator::ExactArgs(1))
        .run(move |_| {
            *executed_clone.lock().unwrap() = true;
            Ok(())
        })
        .build();

    // With wrong args, command should not execute
    let result = cmd.execute(vec![]);
    assert!(result.is_err());
    assert!(
        !*executed.lock().unwrap(),
        "Command should not execute when validation fails"
    );
}

#[test]
fn test_multiple_validators_scenarios() {
    // Validator that requires specific file extension
    let file_validator = ArgValidator::Custom(std::sync::Arc::new(|args| {
        for arg in args {
            if !std::path::Path::new(arg)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
            {
                return Err(Error::ArgumentValidation {
                    message: format!("'{arg}' must be a .txt file"),
                    expected: "*.txt file".to_string(),
                    received: args.len(),
                });
            }
        }
        Ok(())
    }));

    let cmd = CommandBuilder::new("process")
        .args(file_validator)
        .run(|ctx| {
            println!("Processing {} files", ctx.args().len());
            Ok(())
        })
        .build();

    // Should succeed with .txt files
    assert!(
        cmd.execute(vec!["file1.txt".to_string(), "file2.txt".to_string()])
            .is_ok()
    );

    // Should fail with non-.txt file
    assert!(cmd.execute(vec!["file1.pdf".to_string()]).is_err());
}
