use flag_rs::{Command, CommandBuilder, CompletionResult, Context, Flag, FlagType};

#[test]
fn test_complex_cli_app() {
    // Simulate a git-like CLI
    let app = build_git_cli();

    // Test: git clone https://example.com/repo.git
    let result = app.execute(vec![
        "clone".to_string(),
        "https://example.com/repo.git".to_string(),
    ]);
    assert!(result.is_ok());

    // Test: git -C /path/to/repo status
    let result = app.execute(vec![
        "-C".to_string(),
        "/path/to/repo".to_string(),
        "status".to_string(),
    ]);
    assert!(result.is_ok());

    // Test: git commit -m "Initial commit"
    let result = app.execute(vec![
        "commit".to_string(),
        "-m".to_string(),
        "Initial commit".to_string(),
    ]);
    if let Err(e) = &result {
        eprintln!("Commit test failed with: {e}");
    }
    assert!(result.is_ok());
}

fn build_git_cli() -> Command {
    CommandBuilder::new("git")
        .short("The stupid content tracker")
        .flag(
            Flag::new("work-tree")
                .short('C')
                .usage("Change to directory before doing anything")
                .value_type(FlagType::String),
        )
        .subcommand(build_clone_command())
        .subcommand(build_commit_command())
        .subcommand(build_status_command())
        .build()
}

fn build_clone_command() -> Command {
    CommandBuilder::new("clone")
        .short("Clone a repository into a new directory")
        .run(|ctx| {
            if let Some(url) = ctx.args().first() {
                println!("Cloning from: {url}");
            }
            Ok(())
        })
        .build()
}

fn build_commit_command() -> Command {
    CommandBuilder::new("commit")
        .short("Record changes to the repository")
        .flag(
            Flag::new("message")
                .short('m')
                .usage("Use the given message as the commit message")
                .value_type(FlagType::String),
        )
        .run(|ctx| {
            if let Some(message) = ctx.flag("message") {
                println!("Committing with message: {message}");
            }
            Ok(())
        })
        .build()
}

fn build_status_command() -> Command {
    CommandBuilder::new("status")
        .short("Show the working tree status")
        .run(|ctx| {
            if let Some(work_tree) = ctx.flag("work-tree") {
                println!("Checking status in: {work_tree}");
            } else {
                println!("Checking status in current directory");
            }
            Ok(())
        })
        .build()
}

#[test]
fn test_dynamic_completion() {
    let mut cmd = CommandBuilder::new("kubectl")
        .subcommand(
            CommandBuilder::new("get")
                .subcommand(
                    CommandBuilder::new("pods")
                        .arg_completion(|_ctx, prefix| {
                            // Simulate dynamic pod lookup
                            let pods = vec![
                                "nginx-7fb96c846b-8xvnl",
                                "nginx-7fb96c846b-kxptv",
                                "redis-master-0",
                                "redis-slave-0",
                            ];

                            Ok(CompletionResult::new().extend(
                                pods.into_iter()
                                    .filter(|p| p.starts_with(prefix))
                                    .map(String::from),
                            ))
                        })
                        .build(),
                )
                .build(),
        )
        .build();

    // Navigate to the pods subcommand for testing
    let pods_cmd = cmd
        .find_subcommand_mut("get")
        .and_then(|get| get.find_subcommand_mut("pods"))
        .unwrap();

    let ctx = Context::new(vec![]);

    // Test completion
    let result = pods_cmd.get_completions(&ctx, "nginx", None).unwrap();
    assert_eq!(result.values.len(), 2);
    assert!(
        result
            .values
            .contains(&"nginx-7fb96c846b-8xvnl".to_string())
    );
    assert!(
        result
            .values
            .contains(&"nginx-7fb96c846b-kxptv".to_string())
    );

    let result = pods_cmd.get_completions(&ctx, "redis", None).unwrap();
    assert_eq!(result.values.len(), 2);
    assert!(result.values.contains(&"redis-master-0".to_string()));
    assert!(result.values.contains(&"redis-slave-0".to_string()));
}

#[test]
fn test_nested_subcommands() {
    let app = CommandBuilder::new("app")
        .subcommand(
            CommandBuilder::new("level1")
                .subcommand(
                    CommandBuilder::new("level2")
                        .subcommand(
                            CommandBuilder::new("level3")
                                .run(|_ctx| {
                                    println!("Reached level 3!");
                                    Ok(())
                                })
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build();

    let result = app.execute(vec![
        "level1".to_string(),
        "level2".to_string(),
        "level3".to_string(),
    ]);

    assert!(result.is_ok());
}
