//! Demonstrates `ActiveHelp` system for contextual hints during completion
//!
//! This example shows how to use `ActiveHelp` to provide contextual assistance
//! to users as they type commands, similar to Cobra's `ActiveHelp` feature.

use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType};

fn main() {
    let app = CommandBuilder::new("active-help-demo")
        .short("Demonstrates ActiveHelp system")
        .long("This example shows how ActiveHelp provides contextual hints during tab completion")
        .flag(
            Flag::new("namespace")
                .short('n')
                .usage("Kubernetes namespace")
                .value_type(FlagType::String),
        )
        .flag(
            Flag::new("output")
                .short('o')
                .usage("Output format")
                .value_type(FlagType::String),
        )
        .subcommand(
            CommandBuilder::new("get")
                .short("Get resources")
                .subcommand(
                    CommandBuilder::new("pods")
                        .short("Get pods")
                        .arg_completion(|ctx, prefix| {
                            let mut result = CompletionResult::new();

                            // Add contextual help based on current state
                            if ctx.flag("namespace").is_none() {
                                result = result.add_help_text(
                                    "Tip: Use -n <namespace> to list pods from a specific namespace",
                                );
                            }

                            if ctx.flag("output").is_none() {
                                result = result.add_conditional_help(
                                    "Use -o json for machine-readable output",
                                    |_| true, // Always show this tip
                                );
                            }

                            // Simulate pod completions
                            let namespace = ctx
                                .flag("namespace")
                                .map(String::as_str)
                                .unwrap_or("default");

                            let pods = match namespace {
                                "default" => vec![
                                    ("nginx-abc123", "Running"),
                                    ("redis-def456", "Running"),
                                    ("postgres-ghi789", "CrashLoopBackOff"),
                                ],
                                "kube-system" => vec![
                                    ("coredns-xyz789", "Running"),
                                    ("kube-proxy-abc123", "Running"),
                                    ("etcd-master", "Running"),
                                ],
                                _ => vec![],
                            };

                            // Add pods with status descriptions
                            for (pod, status) in &pods {
                                if pod.starts_with(prefix) {
                                    result = result.add_with_description(
                                        *pod,
                                        format!("Status: {status}"),
                                    );
                                }
                            }

                            // Add help if there's a problematic pod
                            if pods.iter().any(|(_, status)| status == &"CrashLoopBackOff") {
                                result = result.add_help_text(
                                    "Warning: Some pods are in CrashLoopBackOff state. Use 'describe' to investigate.",
                                );
                            }

                            Ok(result)
                        })
                        .run(|ctx| {
                            let namespace = ctx
                                .flag("namespace")
                                .map(String::as_str)
                                .unwrap_or("default");
                            let output = ctx.flag("output").map(String::as_str).unwrap_or("table");

                            println!("Getting pods from namespace: {namespace}");
                            println!("Output format: {output}");

                            if let Some(pod) = ctx.args().first() {
                                println!("Getting specific pod: {pod}");
                            } else {
                                println!("Listing all pods");
                            }
                            Ok(())
                        })
                        .build(),
                )
                .subcommand(
                    CommandBuilder::new("services")
                        .short("Get services")
                        .arg_completion(|_ctx, _prefix| {
                            let mut result = CompletionResult::new()
                                .add_with_description("nginx-service", "Type: LoadBalancer")
                                .add_with_description("redis-service", "Type: ClusterIP")
                                .add_with_description("postgres-service", "Type: NodePort");

                            // Add conditional help based on flags
                            result = result.add_conditional_help(
                                "Tip: Use -o wide to see more details about services",
                                |ctx| {
                                    ctx.flag("output")
                                        .map(|o| o != "wide")
                                        .unwrap_or(true)
                                },
                            );

                            Ok(result)
                        })
                        .build(),
                )
                .build(),
        )
        .subcommand(
            CommandBuilder::new("create")
                .short("Create resources")
                .flag(
                    Flag::new("file")
                        .short('f')
                        .usage("Filename to use to create the resource")
                        .value_type(FlagType::String)
                        .required(),
                )
                .flag_completion("file", |_ctx, prefix| {
                    let mut result = CompletionResult::new();

                    // Simulate file completions
                    let files = vec![
                        ("deployment.yaml", "Deployment configuration"),
                        ("service.yaml", "Service configuration"),
                        ("configmap.yaml", "ConfigMap configuration"),
                        ("secret.yaml", "Secret configuration"),
                        ("pod.yaml", "Pod configuration"),
                    ];

                    for (file, desc) in files {
                        if file.starts_with(prefix) {
                            result = result.add_with_description(file, desc);
                        }
                    }

                    // Add contextual help
                    result = result.add_help_text(
                        "Files should be valid Kubernetes manifests in YAML or JSON format",
                    );

                    if std::path::Path::new(prefix)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                    {
                        result = result.add_help_text(
                            "Note: JSON format is supported but YAML is more common in Kubernetes",
                        );
                    }

                    Ok(result)
                })
                .run(|ctx| {
                    let file = ctx.flag("file").expect("File is required");
                    println!("Creating resources from file: {file}");
                    Ok(())
                })
                .build(),
        )
        .subcommand(
            CommandBuilder::new("debug")
                .short("Debug and troubleshoot resources")
                .arg_completion(|_ctx, _prefix| {
                    Ok(CompletionResult::new()
                        .add_with_description("pod/nginx-abc123", "Debug a specific pod")
                        .add_with_description("node/worker-1", "Debug a node")
                        .add_with_description("deployment/nginx", "Debug a deployment")
                        .add_help_text("Debug creates an interactive debugging session")
                        .add_help_text("Common debugging commands: kubectl logs, kubectl exec, kubectl describe")
                        .add_conditional_help(
                            "Tip: Use 'kubectl logs -f' to follow log output in real-time",
                            |ctx| ctx.args().is_empty(),
                        ))
                })
                .build(),
        )
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
