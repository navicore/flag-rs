//! Integration tests for lifecycle hooks functionality

use flag_rs::{CommandBuilder, Error, Flag, FlagType};
use std::sync::{Arc, Mutex};

#[test]
fn test_basic_lifecycle_hooks() {
    // Track execution order
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let cmd = CommandBuilder::new("test")
        .pre_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("pre_run".to_string());
                Ok(())
            }
        })
        .run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("run".to_string());
                Ok(())
            }
        })
        .post_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("post_run".to_string());
                Ok(())
            }
        })
        .build();

    let result = cmd.execute(vec![]);
    assert!(result.is_ok());

    let expected = vec![
        "pre_run".to_string(),
        "run".to_string(),
        "post_run".to_string(),
    ];
    assert_eq!(*execution_log.lock().unwrap(), expected);
}

#[test]
fn test_persistent_hooks_with_subcommands() {
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let cmd = CommandBuilder::new("parent")
        .persistent_pre_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock()
                    .unwrap()
                    .push("parent_persistent_pre".to_string());
                Ok(())
            }
        })
        .persistent_post_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock()
                    .unwrap()
                    .push("parent_persistent_post".to_string());
                Ok(())
            }
        })
        .subcommand(
            CommandBuilder::new("child")
                .pre_run({
                    let log = execution_log.clone();
                    move |_ctx| {
                        log.lock().unwrap().push("child_pre".to_string());
                        Ok(())
                    }
                })
                .run({
                    let log = execution_log.clone();
                    move |_ctx| {
                        log.lock().unwrap().push("child_run".to_string());
                        Ok(())
                    }
                })
                .post_run({
                    let log = execution_log.clone();
                    move |_ctx| {
                        log.lock().unwrap().push("child_post".to_string());
                        Ok(())
                    }
                })
                .build(),
        )
        .build();

    let result = cmd.execute(vec!["child".to_string()]);
    assert!(result.is_ok());

    let expected = vec![
        "parent_persistent_pre".to_string(),
        "child_pre".to_string(),
        "child_run".to_string(),
        "child_post".to_string(),
        "parent_persistent_post".to_string(),
    ];
    assert_eq!(*execution_log.lock().unwrap(), expected);
}

#[test]
fn test_nested_persistent_hooks() {
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let cmd = CommandBuilder::new("root")
        .persistent_pre_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("root_persistent_pre".to_string());
                Ok(())
            }
        })
        .persistent_post_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("root_persistent_post".to_string());
                Ok(())
            }
        })
        .subcommand(
            CommandBuilder::new("level1")
                .persistent_pre_run({
                    let log = execution_log.clone();
                    move |_ctx| {
                        log.lock()
                            .unwrap()
                            .push("level1_persistent_pre".to_string());
                        Ok(())
                    }
                })
                .persistent_post_run({
                    let log = execution_log.clone();
                    move |_ctx| {
                        log.lock()
                            .unwrap()
                            .push("level1_persistent_post".to_string());
                        Ok(())
                    }
                })
                .subcommand(
                    CommandBuilder::new("level2")
                        .run({
                            let log = execution_log.clone();
                            move |_ctx| {
                                log.lock().unwrap().push("level2_run".to_string());
                                Ok(())
                            }
                        })
                        .build(),
                )
                .build(),
        )
        .build();

    let result = cmd.execute(vec!["level1".to_string(), "level2".to_string()]);
    assert!(result.is_ok());

    let expected = vec![
        "root_persistent_pre".to_string(),
        "level1_persistent_pre".to_string(),
        "level2_run".to_string(),
        "level1_persistent_post".to_string(),
        "root_persistent_post".to_string(),
    ];
    assert_eq!(*execution_log.lock().unwrap(), expected);
}

#[test]
fn test_hook_error_handling() {
    // Test pre-run error stops execution
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let cmd = CommandBuilder::new("test")
        .pre_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("pre_run".to_string());
                Err(Error::Validation("Pre-run failed".to_string()))
            }
        })
        .run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("run".to_string());
                Ok(())
            }
        })
        .post_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("post_run".to_string());
                Ok(())
            }
        })
        .build();

    let result = cmd.execute(vec![]);
    assert!(result.is_err());
    assert_eq!(*execution_log.lock().unwrap(), vec!["pre_run".to_string()]);

    // Test run error still executes post-run
    let execution_log2 = Arc::new(Mutex::new(Vec::new()));

    let cmd2 = CommandBuilder::new("test")
        .pre_run({
            let log = execution_log2.clone();
            move |_ctx| {
                log.lock().unwrap().push("pre_run".to_string());
                Ok(())
            }
        })
        .run({
            let log = execution_log2.clone();
            move |_ctx| {
                log.lock().unwrap().push("run".to_string());
                Err(Error::Validation("Run failed".to_string()))
            }
        })
        .post_run({
            let log = execution_log2.clone();
            move |_ctx| {
                log.lock().unwrap().push("post_run".to_string());
                Ok(())
            }
        })
        .build();

    let result = cmd2.execute(vec![]);
    assert!(result.is_err());
    assert_eq!(
        *execution_log2.lock().unwrap(),
        vec![
            "pre_run".to_string(),
            "run".to_string(),
            "post_run".to_string()
        ]
    );
}

#[test]
fn test_hooks_with_context_access() {
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let cmd = CommandBuilder::new("test")
        .flag(
            Flag::new("config")
                .short('c')
                .usage("Config file")
                .value_type(FlagType::String),
        )
        .persistent_pre_run({
            let log = execution_log.clone();
            move |ctx| {
                if let Some(config) = ctx.flag("config") {
                    log.lock()
                        .unwrap()
                        .push(format!("persistent_pre: config={config}"));
                } else {
                    log.lock()
                        .unwrap()
                        .push("persistent_pre: no config".to_string());
                }
                Ok(())
            }
        })
        .subcommand(
            CommandBuilder::new("process")
                .pre_run({
                    let log = execution_log.clone();
                    move |ctx| {
                        // Should have access to parent's flags
                        if let Some(config) = ctx.flag("config") {
                            log.lock().unwrap().push(format!("pre: config={config}"));
                        }
                        Ok(())
                    }
                })
                .run({
                    let log = execution_log.clone();
                    move |ctx| {
                        log.lock().unwrap().push("run".to_string());
                        // Set a value for post-run
                        ctx.set(String::from("success"));
                        Ok(())
                    }
                })
                .post_run({
                    let log = execution_log.clone();
                    move |ctx| {
                        if let Some(result) = ctx.get::<String>() {
                            log.lock().unwrap().push(format!("post: result={result}"));
                        }
                        Ok(())
                    }
                })
                .build(),
        )
        .build();

    let result = cmd.execute(vec![
        "--config".to_string(),
        "app.conf".to_string(),
        "process".to_string(),
    ]);
    assert!(result.is_ok());

    let expected = vec![
        "persistent_pre: config=app.conf",
        "pre: config=app.conf",
        "run",
        "post: result=success",
    ];
    assert_eq!(*execution_log.lock().unwrap(), expected);
}

#[test]
fn test_multiple_persistent_hooks_ordering() {
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let cmd = CommandBuilder::new("app")
        .persistent_pre_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("app_persistent_pre".to_string());
                Ok(())
            }
        })
        .persistent_post_run({
            let log = execution_log.clone();
            move |_ctx| {
                log.lock().unwrap().push("app_persistent_post".to_string());
                Ok(())
            }
        })
        .subcommand(
            CommandBuilder::new("service")
                .persistent_pre_run({
                    let log = execution_log.clone();
                    move |_ctx| {
                        log.lock()
                            .unwrap()
                            .push("service_persistent_pre".to_string());
                        Ok(())
                    }
                })
                .persistent_post_run({
                    let log = execution_log.clone();
                    move |_ctx| {
                        log.lock()
                            .unwrap()
                            .push("service_persistent_post".to_string());
                        Ok(())
                    }
                })
                .subcommand(
                    CommandBuilder::new("api")
                        .persistent_pre_run({
                            let log = execution_log.clone();
                            move |_ctx| {
                                log.lock().unwrap().push("api_persistent_pre".to_string());
                                Ok(())
                            }
                        })
                        .persistent_post_run({
                            let log = execution_log.clone();
                            move |_ctx| {
                                log.lock().unwrap().push("api_persistent_post".to_string());
                                Ok(())
                            }
                        })
                        .subcommand(
                            CommandBuilder::new("start")
                                .run({
                                    let log = execution_log.clone();
                                    move |_ctx| {
                                        log.lock().unwrap().push("start_run".to_string());
                                        Ok(())
                                    }
                                })
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build();

    let result = cmd.execute(vec![
        "service".to_string(),
        "api".to_string(),
        "start".to_string(),
    ]);
    assert!(result.is_ok());

    let expected = vec![
        "app_persistent_pre".to_string(),
        "service_persistent_pre".to_string(),
        "api_persistent_pre".to_string(),
        "start_run".to_string(),
        "api_persistent_post".to_string(),
        "service_persistent_post".to_string(),
        "app_persistent_post".to_string(),
    ];
    assert_eq!(*execution_log.lock().unwrap(), expected);
}
